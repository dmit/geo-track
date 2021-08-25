//! This module houses the [`Storage`] trait that describes the interface of
//! supported persistence engines, as well as its implementations.

mod memory;
#[cfg(feature = "sled")]
mod sled;

use std::{fmt::Debug, ops::RangeBounds, str::FromStr};

use async_trait::async_trait;
use eyre::bail;
use shared::data::{SourceId, Status};
use time::OffsetDateTime;

use crate::storage::memory::MemoryStorage;

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
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
                bail!("recompile with --features=sled for Sled storage support")
            }
            _ => bail!("unsupported storage type: {}", s),
        };
        Ok(storage)
    }
}

/// This trait describes the operations that all supported storage engines must
/// support in order to be used in this project.
#[async_trait]
pub trait Storage {
    /// Save a single [`Status`] packet.
    async fn persist_status(&mut self, status: Status) -> eyre::Result<()>;

    /// Get a range of [`Status`] packets for a given [`SourceId`] in a given
    /// time range.
    async fn get_statuses<R>(
        &self,
        source_id: SourceId,
        timestamps: R,
    ) -> eyre::Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send + Debug;
}

/// A concrete instance of one of the supported storage engines.
pub enum StorageEngine {
    #[doc(hidden)]
    InMemory(MemoryStorage),
    #[doc(hidden)]
    #[cfg(feature = "sled")]
    Sled(sled::SledStorage),
}

#[async_trait]
impl Storage for StorageEngine {
    async fn persist_status(&mut self, status: Status) -> eyre::Result<()> {
        match self {
            Self::InMemory(s) => s.persist_status(status).await,
            #[cfg(feature = "sled")]
            Self::Sled(s) => s.persist_status(status).await,
        }
    }

    async fn get_statuses<R>(&self, source_id: SourceId, timestamps: R) -> eyre::Result<Vec<Status>>
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
pub fn init(cfg: &StorageConfig) -> eyre::Result<StorageEngine> {
    match cfg {
        StorageConfig::InMemory => Ok(StorageEngine::InMemory(MemoryStorage::default())),
        #[cfg(feature = "sled")]
        StorageConfig::Sled { config } => sled::SledStorage::new(config).map(StorageEngine::Sled),
    }
}
