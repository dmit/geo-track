use std::{fmt::Debug, ops::RangeBounds, path::PathBuf};

use async_trait::async_trait;
use shared::data::{SourceId, Status};
use sled::Db;
use time::OffsetDateTime;

use crate::storage::{self, DupeStrategy, Storage};

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
    dupe_strategy: DupeStrategy,
}

impl SledStorage {
    pub fn new(cfg: &SledConfig, dupe_strategy: DupeStrategy) -> storage::Result<Self> {
        let db = sled::open(&cfg.db_dir)?;
        Ok(Self { db, dupe_strategy })
    }
}

#[async_trait]
impl Storage for SledStorage {
    #[tracing::instrument(skip(self))]
    async fn persist_status(&mut self, status: Status) -> storage::Result<()> {
        todo!();
    }

    #[tracing::instrument(skip(self))]
    async fn get_statuses<R>(
        &self,
        _source_id: SourceId,
        _timestamps: R,
    ) -> storage::Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send + Debug,
    {
        todo!();
    }
}
