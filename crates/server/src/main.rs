#![forbid(unsafe_code)]

mod http;
mod ingest;

use argh::FromArgs;
use eyre::eyre;
use tokio::{net::lookup_host, sync::mpsc};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

#[derive(Debug, FromArgs)]
#[argh(description = "Geo Tracker network service")]
struct Opts {
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

    /// read timeout for the TCP listener
    #[argh(option, default = "std::time::Duration::from_secs(30).into()")]
    tcp_read_timeout: humantime::Duration,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup_logging()?;

    let opts = argh::from_env::<Opts>();
    let http_addr = lookup_host((opts.host.as_str(), opts.port))
        .await?
        .next()
        .ok_or_else(|| eyre!("Unable to resolve HTTP host: {}", &opts.host))?;
    let tcp_addr = lookup_host((opts.tcp_host.as_str(), opts.tcp_port))
        .await?
        .next()
        .ok_or_else(|| eyre!("Unable to resolve TCP host: {}", &opts.tcp_host))?;

    let (status_tx, mut _status_rx) = mpsc::channel(1024);

    ingest::listen(&tcp_addr, opts.tcp_read_timeout.into(), status_tx).await?;
    http::listen(&http_addr).await?;

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
