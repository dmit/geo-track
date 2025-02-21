//! The HTTP server providing the public API.

use std::net::SocketAddr;

use axum::{
    Extension, Json,
    extract::{self, Query},
    http::StatusCode,
    routing::{Router, get},
};
use serde::Deserialize;
use shared::data::{SourceId, Status};
use thiserror::Error;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::storage::{StorageCommand, StorageHandler};

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("internal HTTP server error")]
    Internal(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HttpError>;

/// Bind to the specified network address and start serving HTTP requests.
#[tracing::instrument(skip(handler))]
pub async fn listen(addr: &SocketAddr, handler: StorageHandler) -> Result<()> {
    // Routes are listed from least specific to most specific.
    let app = Router::new()
        .route("/", get(hello))
        .route("/status", get(latest_status).post(submit_status))
        .layer(Extension(handler))
        .layer(TraceLayer::new_for_http());

    info!("Starting HTTP server at http://{}:{}...", addr.ip(), addr.port());

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct HelloQuery {
    name: String,
}

#[tracing::instrument]
async fn hello(Query(query): Query<Option<HelloQuery>>) -> String {
    match query {
        Some(HelloQuery { name }) => format!("Hello, {}!", name),
        None => "Hello!".to_owned(),
    }
}

#[tracing::instrument(skip(handler))]
async fn submit_status(
    extract::Extension(handler): extract::Extension<StorageHandler>,
    extract::Json(status): extract::Json<Status>,
) -> StatusCode {
    match handler.command(StorageCommand::PersistStatus(status)).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!(%err, "Failed to write status update");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Debug, Deserialize)]
struct LatestStatusQuery {
    source_id: SourceId,
}

#[tracing::instrument]
async fn latest_status(
    Query(query): Query<LatestStatusQuery>,
) -> std::result::Result<Json<Status>, StatusCode> {
    // query.source_id
    todo!()
}
