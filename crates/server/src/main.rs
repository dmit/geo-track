#![forbid(unsafe_code)]

mod http;
mod ingest;

use std::net::{IpAddr, SocketAddr};

use argh::FromArgs;
use tokio::sync::mpsc;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

#[derive(Debug, FromArgs)]
#[argh(description = "Geo Tracker network service")]
struct Opts {
    /// network host the HTTP server will bind to
    #[argh(option, short = 'h', default = "std::net::Ipv4Addr::LOCALHOST.into()")]
    host: IpAddr,

    /// network port the HTTP server will bind to
    #[argh(option, short = 'p', default = "8000")]
    port: u16,

    /// network host the TCP listener will bind to
    #[argh(option, default = "std::net::Ipv4Addr::LOCALHOST.into()")]
    tcp_host: IpAddr,

    /// network port the TCP listener will bind to
    #[argh(option, default = "8001")]
    tcp_port: u16,

    /// read timeout for the TCP listener
    #[argh(option, default = "std::time::Duration::from_secs(30).into()")]
    tcp_read_timeout: humantime::Duration,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup_logging()?;
    let opts = argh::from_env::<Opts>();
    let (status_tx, mut _status_rx) = mpsc::channel(1024);

    ingest::listen(
        &SocketAddr::new(opts.tcp_host, opts.tcp_port),
        opts.tcp_read_timeout.into(),
        status_tx,
    )
    .await?;
    http::listen(&SocketAddr::new(opts.host, opts.port)).await?;

    Ok(())
}

fn setup_logging() -> eyre::Result<()> {
    // tracing
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
