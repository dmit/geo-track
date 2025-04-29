//! This module houses the [`Storage`] trait that describes the interface of
//! supported persistence engines, as well as its implementations.

mod memory;
#[cfg(feature = "sled")]
mod sled;

use std::{
    fmt::Debug,
    ops::{Bound, RangeBounds},
    str::FromStr,
};

use shared::data::{SourceId, Status};
use thiserror::Error;
use time::OffsetDateTime;

use crate::{
    cq::{Address, Request},
    storage::memory::MemoryStorage,
};

/// Storage errors.
#[derive(Debug, Error)]
pub enum StorageError {
    #[cfg(feature = "sled")]
    #[error("Sled error")]
    Sled(#[from] ::sled::Error),
    #[error("storage type not compiled: {name}; recompile with corresponding --features flag")]
    StorageNotCompiled { name: String },
    #[error("unknown duplicate strategy: {name}")]
    UnknownDupeStrategy { name: String },
    #[error("unknown storage type: {name}")]
    UnknownStorageType { name: String },
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// This trait describes the operations that all supported storage engines must
/// support in order to be used in this project.
trait Storage {
    /// Save a single [`Status`] packet.
    async fn persist_status(&mut self, status: Status) -> Result<()>;

    /// Get a range of [`Status`] packets for a given [`SourceId`] in a given
    /// time range.
    async fn get_statuses<R>(&self, source_id: SourceId, timestamps: R) -> Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send + Debug;
}

/// Lists all supported storage backends along with their corresponding
/// configuration options.
#[derive(Debug)]
pub enum StorageConfig {
    /// In-memory storage. Not persisted between service restarts.
    InMemory,
    /// Persistent storage backed by the Sled database engine.
    #[cfg(feature = "sled")]
    Sled {
        /// Path to the directory where Sled stores its data.
        config: sled::SledConfig,
    },
}

impl FromStr for StorageConfig {
    type Err = StorageError;

    fn from_str(s: &str) -> Result<Self> {
        let storage = match s {
            "memory" => Self::InMemory,
            #[cfg(feature = "sled")]
            _ if s == "sled" || s.starts_with("sled:") => {
                let config = if let Some(db_dir) =
                    s.split_once(':').map(|(_, path)| std::path::PathBuf::from(path))
                {
                    sled::SledConfig { db_dir }
                } else {
                    sled::SledConfig::default()
                };
                Self::Sled { config }
            }
            #[cfg(not(feature = "sled"))]
            _ if s == "sled" || s.starts_with("sled:") => {
                return Err(StorageError::StorageNotCompiled { name: s.to_owned() });
            }
            _ => return Err(StorageError::UnknownStorageType { name: s.to_owned() }),
        };
        Ok(storage)
    }
}

/// Strategy to use when multiple [`Status`] packets arrive with the same pair
/// of `source_id` + `timestamp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DupeStrategy {
    /// Discard newly received packets, keeping the original one.
    Drop,
    /// Add all fields of the newly received packets to the current entry,
    /// potentially overwriting existing data.
    Merge,
    /// Replace existing [`Status`] entry with the newly received one.
    Overwrite,
}

impl FromStr for DupeStrategy {
    type Err = StorageError;

    fn from_str(s: &str) -> Result<Self> {
        let strategy = match s {
            "drop" => Self::Drop,
            "merge" => Self::Merge,
            "overwrite" => Self::Overwrite,
            _ => return Err(StorageError::UnknownDupeStrategy { name: s.to_owned() }),
        };
        Ok(strategy)
    }
}

/// A concrete instance of one of the supported storage engines.
pub enum StorageEngine {
    #[doc(hidden)]
    InMemory(MemoryStorage),
    #[doc(hidden)]
    #[cfg(feature = "sled")]
    Sled(sled::SledStorage),
}

impl Storage for StorageEngine {
    async fn persist_status(&mut self, status: Status) -> Result<()> {
        match self {
            Self::InMemory(s) => s.persist_status(status).await,
            #[cfg(feature = "sled")]
            Self::Sled(s) => s.persist_status(status).await,
        }
    }

    async fn get_statuses<R>(&self, source_id: SourceId, timestamps: R) -> Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send + Debug,
    {
        match self {
            Self::InMemory(s) => s.get_statuses(source_id, timestamps).await,
            #[cfg(feature = "sled")]
            Self::Sled(s) => s.get_statuses(source_id, timestamps).await,
        }
    }
}

/// Initialize an instance of a storage engine based on the provided
/// [`StorageConfig`] and return it.
#[tracing::instrument]
pub fn init(cfg: &StorageConfig, dupe_strategy: DupeStrategy) -> Result<StorageEngine> {
    match cfg {
        StorageConfig::InMemory => Ok(StorageEngine::InMemory(MemoryStorage::new(dupe_strategy))),
        #[cfg(feature = "sled")]
        StorageConfig::Sled { config } => {
            sled::SledStorage::new(config, dupe_strategy).map(StorageEngine::Sled)
        }
    }
}

pub type StorageHandler = Address<StorageCommand, StorageQuery>;

pub enum StorageCommand {
    PersistStatus(Status),
}

impl Request for StorageCommand {
    type Result = Result<()>;
}

pub enum StorageQuery {
    GetStatuses(GetStatuses),
}

impl Request for StorageQuery {
    //TODO
    type Result = Result<()>;
}

#[derive(Debug, Clone)]
pub struct GetStatuses {
    source_id: SourceId,
    timestamps: (Bound<OffsetDateTime>, Bound<OffsetDateTime>),
}
