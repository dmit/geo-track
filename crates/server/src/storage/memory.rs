use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    ops::RangeBounds,
};

use async_trait::async_trait;
use shared::data::{SourceId, Status};
use time::OffsetDateTime;

use crate::storage::{DupeStrategy, Storage};

pub struct MemoryStorage {
    statuses: HashMap<SourceId, BTreeMap<OffsetDateTime, Status>>,
    dupe_strategy: DupeStrategy,
}

impl MemoryStorage {
    pub fn new(dupe_strategy: DupeStrategy) -> Self {
        Self { statuses: Default::default(), dupe_strategy }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn persist_status(&mut self, status: Status) -> eyre::Result<()> {
        match self.dupe_strategy {
            DupeStrategy::Drop => {
                self.statuses
                    .entry(status.source_id)
                    .or_default()
                    .entry(status.timestamp)
                    .or_insert(status);
            }
            DupeStrategy::Merge => {
                self.statuses
                    .entry(status.source_id)
                    .or_default()
                    .entry(status.timestamp)
                    .and_modify(|s| *s = s.merge(&status))
                    .or_insert(status);
            }
            DupeStrategy::Overwrite => {
                self.statuses.entry(status.source_id).or_default().insert(status.timestamp, status);
            }
        }
        Ok(())
    }

    async fn get_statuses<R>(&self, source_id: SourceId, timestamps: R) -> eyre::Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send + Debug,
    {
        let range = self
            .statuses
            .get(&source_id)
            .map(|m| m.range(timestamps).map(|(_, v)| v).copied().collect())
            .unwrap_or_default();
        Ok(range)
    }
}
