#![forbid(unsafe_code)]

use std::net::{IpAddr, SocketAddr};

use argh::FromArgs;
use axum::prelude::*;
use serde::Deserialize;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup_logging()?;
    start_server().await
}

fn setup_logging() -> eyre::Result<()> {
    // color-eyre
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }
    color_eyre::install()?;

    // tracing
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    Ok(())
}

#[derive(Debug, FromArgs)]
#[argh(description = "Geo Tracker network service")]
struct Opts {
    /// network host the server will bind to
    #[argh(option, short = 'h', default = "std::net::Ipv4Addr::LOCALHOST.into()")]
    host: IpAddr,

    /// network port the server will bind to
    #[argh(option, short = 'p', default = "8000")]
    port: u16,
}

async fn start_server() -> eyre::Result<()> {
    let opts: Opts = argh::from_env();
    info!(?opts);

    let app = route("/", get(hello));

    info!(host = %opts.host, post = %opts.port, "Starting network server...");
    axum::Server::bind(&SocketAddr::new(opts.host, opts.port))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Deserialize)]
struct HelloQuery {
    name: String,
}

async fn hello(query: Option<extract::Query<HelloQuery>>) -> String {
    match query {
        Some(extract::Query(HelloQuery { name })) => format!("Hello, {}!", name),
        None => "Hello!".to_owned(),
    }
}
