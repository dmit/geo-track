mod memory;
#[cfg(feature = "sled")]
mod sled;

use std::{ops::RangeBounds, str::FromStr};

use async_trait::async_trait;
use eyre::bail;
use shared::data::{SourceId, Status};
use time::OffsetDateTime;

use crate::storage::memory::MemoryStorage;
#[cfg(feature = "sled")]
use crate::storage::sled::{SledConfig, SledStorage};

#[derive(Debug)]
pub enum StorageConfig {
    InMemory,
    #[cfg(feature = "sled")]
    Sled {
        config: SledConfig,
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
                    SledConfig { db_dir }
                } else {
                    SledConfig::default()
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

#[async_trait]
pub trait Storage {
    async fn persist_status(&mut self, status: Status) -> eyre::Result<()>;

    async fn get_statuses<R>(
        &self,
        source_id: SourceId,
        timestamps: R,
    ) -> eyre::Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send;
}

pub enum StorageEngine {
    InMemory(MemoryStorage),
    #[cfg(feature = "sled")]
    Sled(SledStorage),
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
        R: RangeBounds<OffsetDateTime> + Send,
    {
        match self {
            Self::InMemory(s) => s.get_statuses(source_id, timestamps).await,
            #[cfg(feature = "sled")]
            Self::Sled(s) => s.get_statuses(source_id, timestamps).await,
        }
    }
}

pub fn init(cfg: &StorageConfig) -> eyre::Result<StorageEngine> {
    match cfg {
        StorageConfig::InMemory => Ok(StorageEngine::InMemory(MemoryStorage::default())),
        #[cfg(feature = "sled")]
        StorageConfig::Sled { config } => SledStorage::new(config).map(StorageEngine::Sled),
    }
}
