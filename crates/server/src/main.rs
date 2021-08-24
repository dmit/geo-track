#![forbid(unsafe_code)]

mod http;
mod ingest;
mod storage;

use std::{io, net::SocketAddr};

use argh::FromArgs;
use eyre::{eyre, WrapErr};
use tokio::{net::lookup_host, sync::mpsc};
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

#[derive(Debug, FromArgs)]
#[argh(description = "Geo Tracker network service")]
struct Opts {
    /// storage to use for incoming events and computed data.
    /// supported values:
    /// "memory" (in-memory storage);
    /// "sled[:db_path]" (on-disk persistence using the embedded Sled database
    /// engine, with an optional path to the storage directory)
    #[argh(option, default = "storage::StorageConfig::InMemory")]
    storage: storage::StorageConfig,

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
    async fn lookup_first(host: &str, port: u16) -> io::Result<SocketAddr> {
        lookup_host((host, port)).await.and_then(|mut addrs| {
            addrs.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "host didn't resolve to any IP addresses")
            })
        })
    }

    setup_logging()?;

    let opts = argh::from_env::<Opts>();
    info!(?opts, "Starting server...");

    // Initializing storage.
    info!("Initializing storage...");
    let mut _storage = storage::init(&opts.storage).wrap_err("Failed to initialize storage")?;

    // Initializing network listeners.
    let http_addr = lookup_first(opts.host.as_str(), opts.port)
        .await
        .wrap_err_with(|| eyre!("Failed to resolve HTTP bind address: {}", &opts.host))?;
    let tcp_addr = lookup_first(opts.tcp_host.as_str(), opts.tcp_port)
        .await
        .wrap_err_with(|| eyre!("Failed to resolve TCP bind address: {}", &opts.tcp_host))?;
    let udp_addr = lookup_first(opts.udp_host.as_str(), opts.udp_port)
        .await
        .wrap_err_with(|| eyre!("Failed to resolve UDP bind address: {}", &opts.udp_host))?;

    let (status_tx, mut _status_rx) = mpsc::channel(1024);

    ingest::listen_tcp(&tcp_addr, opts.tcp_read_timeout.into(), status_tx.clone()).await?;
    ingest::listen_udp(&udp_addr, status_tx).await?;
    http::listen(&http_addr).await?;

    Ok(())
}

fn setup_logging() -> eyre::Result<()> {
    // Default log level for tracing.
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let filter = EnvFilter::try_from_default_env()?;
    let output = tracing_subscriber::fmt::layer()
        .with_timer(ChronoUtc::with_format("%Y-%m-%d %H:%M:%S.%3fZ".to_owned()));
    let errors = ErrorLayer::default();
    tracing_subscriber::registry().with(filter).with(output).with(errors).init();

    Ok(())
}
