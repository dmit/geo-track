#![forbid(unsafe_code)]

use std::net::{IpAddr, SocketAddr};

use argh::FromArgs;
use tracing::info;
use tracing_subscriber::EnvFilter;
use warp::Filter;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup()?;

    run().await
}

fn setup() -> eyre::Result<()> {
    // color-eyre
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    // tracing
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    Ok(())
}

#[derive(Debug, FromArgs)]
#[argh(description = "Geo Tracker service")]
struct Opts {
    /// network host the server will bind to
    #[argh(option, short = 'h', default = "std::net::Ipv4Addr::LOCALHOST.into()")]
    host: IpAddr,

    /// network port the server will bind to
    #[argh(option, short = 'p', default = "8000")]
    port: u16,
}

async fn run() -> eyre::Result<()> {
    let opts: Opts = argh::from_env();
    info!(?opts);

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    info!(host = %opts.host, post = %opts.port, "Starting network server...");
    warp::serve(hello).run(SocketAddr::new(opts.host, opts.port)).await;

    Ok(())
}
