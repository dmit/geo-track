use thiserror::Error;

use crate::{http::HttpError, ingest::IngestError, storage::StorageError};

/// Parent of all server errors.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("HTTP server error")]
    Http(#[from] HttpError),
    #[error("ingest server error")]
    Ingest(#[from] IngestError),
    #[error("storage error")]
    Storage(#[from] StorageError),
}

pub type Result<T> = std::result::Result<T, ServerError>;
