use std::{fmt::Debug, io, net::SocketAddr};

use argh::FromArgs;

use eyre::{WrapErr, eyre};
use server::{
    cq::{self, Request},
    http, ingest,
    storage::{self, StorageCommand, StorageQuery},
};
use time::{format_description, macros::format_description};
use tokio::net::lookup_host;
use tracing::{error, info};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt::time::UtcTime, prelude::*};

#[derive(Debug, FromArgs)]
#[argh(description = "Geo Tracker network service")]
struct Opts {
    /// storage to use for incoming events and computed data.
    /// supported values:
    /// "memory" (in-memory storage; default),
    /// "sled[:db_path]" (on-disk persistence using the embedded Sled database
    /// engine, with an optional path to the storage directory)
    #[argh(option, default = "storage::StorageConfig::InMemory")]
    storage: storage::StorageConfig,

    /// strategy to use when receiving multiple statuses for the same sensor
    /// and timestamp. supported values: "merge" (fields from duplicate packets
    /// are added to the existing status entry; default), "drop" (duplicates
    /// are discarded), "overwrite" (duplicate packets overwrite existing
    /// entries)
    #[argh(option, default = "storage::DupeStrategy::Merge")]
    duplicates: storage::DupeStrategy,

    /// network host the HTTP server will bind to
    #[argh(option, short = 'h', default = "\"127.0.0.1\".to_owned()")]
    host: String,

    /// network port the HTTP server will bind to
    #[argh(option, short = 'p', default = "8000")]
    port: u16,

    /// network host the TCP listener will bind to
    #[argh(option, default = "\"127.0.0.1\".to_owned()")]
    tcp_host: String,

    /// network port the TCP listener will bind to
    #[argh(option, default = "8001")]
    tcp_port: u16,

    /// network host the UDP listener will bind to
    #[argh(option, default = "\"127.0.0.1\".to_owned()")]
    udp_host: String,

    /// network port the UDP listener will bind to
    #[argh(option, default = "8002")]
    udp_port: u16,

    /// read timeout for the TCP listener
    #[argh(option, default = "std::time::Duration::from_secs(30).into()")]
    tcp_read_timeout: humantime::Duration,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    #[tracing::instrument]
    async fn lookup_first(host: &str, port: u16) -> eyre::Result<SocketAddr> {
        lookup_host((host, port))
            .await
            .and_then(|mut addrs| {
                addrs.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Host didn't resolve to any IP addresses")
                })
            })
            .wrap_err_with(|| eyre!("Failed to resolve hostname: {}", host))
    }

    set_up_logging()?;

    let opts = argh::from_env::<Opts>();
    info!(?opts, "Starting server...");

    // Initializing storage.
    info!("Initializing storage...");
    let _storage =
        storage::init(&opts.storage, opts.duplicates).wrap_err("Failed to initialize storage")?;

    let (status_tx, mut status_rx) = cq::bounded(1024, on_command, on_query);

    tokio::spawn(async move {
        loop {
            if let Err(err) = status_rx.next().await {
                error!(%err, "Failed to process status event");
            }
        }
    });

    // Initializing network listeners.
    let http_addr = lookup_first(opts.host.as_str(), opts.port).await?;
    let tcp_addr = lookup_first(opts.tcp_host.as_str(), opts.tcp_port).await?;
    let udp_addr = lookup_first(opts.udp_host.as_str(), opts.udp_port).await?;

    ingest::listen_tcp(&tcp_addr, opts.tcp_read_timeout.into(), status_tx.clone()).await?;
    ingest::listen_udp(&udp_addr, status_tx.clone()).await?;
    http::listen(&http_addr, status_tx.clone()).await?;

    Ok(())
}

async fn on_command(_cmd: StorageCommand) -> <StorageCommand as Request>::Result {
    todo!();
}

async fn on_query(_query: StorageQuery) -> <StorageQuery as Request>::Result {
    todo!();
}

fn set_up_logging() -> eyre::Result<()> {
    const TIMESTAMP_FORMAT: &[format_description::FormatItem] =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]Z");

    // Backtrace and spantrace capture.
    color_eyre::install()?;

    // Default log level.
    const DEFAULT_LOG_LEVEL: LevelFilter =
        if cfg!(debug_assertions) { LevelFilter::DEBUG } else { LevelFilter::INFO };

    let filter =
        EnvFilter::builder().with_default_directive(DEFAULT_LOG_LEVEL.into()).from_env()?;
    let output = tracing_subscriber::fmt::layer().with_timer(UtcTime::new(TIMESTAMP_FORMAT));
    let errors = ErrorLayer::default();
    tracing_subscriber::registry().with(filter).with(output).with(errors).init();

    Ok(())
}
