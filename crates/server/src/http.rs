//! The HTTP server providing the public API.

use std::net::SocketAddr;

use axum::prelude::*;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tracing::instrument]
pub async fn listen(addr: &SocketAddr) -> eyre::Result<()> {
    let app = route("/", get(hello)).layer(TraceLayer::new_for_http());

    info!("Starting HTTP server at http://{}:{}...", addr.ip(), addr.port());
    axum::Server::bind(addr).serve(app.into_make_service()).await?;

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
