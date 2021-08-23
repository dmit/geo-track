#![forbid(unsafe_code)]

use std::net::{IpAddr, SocketAddr};

use argh::FromArgs;
use axum::prelude::*;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::time::ChronoUtc, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup_logging()?;
    let opts = argh::from_env();
    start_server(opts).await
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
    let filter = EnvFilter::try_from_default_env()?;
    let output = tracing_subscriber::fmt::layer()
        .with_timer(ChronoUtc::with_format("%Y-%m-%d %H:%M:%S.%3fZ".to_owned()));
    let errors = ErrorLayer::default();
    tracing_subscriber::registry().with(filter).with(output).with(errors).init();

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

#[tracing::instrument]
async fn start_server(opts: Opts) -> eyre::Result<()> {
    let app = route("/", get(hello)).layer(TraceLayer::new_for_http());

    info!("Starting network server at http://{}:{}...", opts.host, opts.port);
    axum::Server::bind(&SocketAddr::new(opts.host, opts.port))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct HelloQuery {
    name: String,
}

#[tracing::instrument]
async fn hello(query: Option<extract::Query<HelloQuery>>) -> String {
    match query {
        Some(extract::Query(HelloQuery { name })) => format!("Hello, {}!", name),
        None => "Hello!".to_owned(),
    }
}
