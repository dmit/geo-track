use std::fmt::Debug;
use std::{ops::RangeBounds, path::PathBuf};

use crate::storage::Storage;
use async_trait::async_trait;
use shared::data::{SourceId, Status};
use sled::Db;
use time::OffsetDateTime;

#[derive(Debug)]
pub struct SledConfig {
    pub db_dir: PathBuf,
}

impl Default for SledConfig {
    fn default() -> Self {
        Self { db_dir: PathBuf::from("./geo_track_sled_db") }
    }
}

pub struct SledStorage {
    db: Db,
}

impl SledStorage {
    pub fn new(cfg: &SledConfig) -> eyre::Result<Self> {
        let db = sled::open(&cfg.db_dir)?;
        Ok(Self { db })
    }
}

#[async_trait]
impl Storage for SledStorage {
    #[tracing::instrument(skip(self))]
    async fn persist_status(&mut self, status: Status) -> eyre::Result<()> {
        todo!();
    }

    #[tracing::instrument(skip(self))]
    async fn get_statuses<R>(&self, source_id: SourceId, timestamps: R) -> eyre::Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send + Debug,
    {
        todo!();
    }
}
