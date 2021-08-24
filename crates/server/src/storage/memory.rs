use std::{
    collections::{BTreeMap, HashMap},
    ops::RangeBounds,
};

use shared::data::{SourceId, Status};
use time::OffsetDateTime;

use crate::storage::Storage;
use async_trait::async_trait;

#[derive(Default)]
pub struct MemoryStorage {
    statuses: HashMap<SourceId, BTreeMap<OffsetDateTime, Status>>,
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn persist_status(&mut self, status: Status) -> eyre::Result<()> {
        self.statuses.entry(status.source_id).or_default().insert(status.timestamp, status);
        Ok(())
    }

    async fn get_statuses<R>(&self, source_id: SourceId, timestamps: R) -> eyre::Result<Vec<Status>>
    where
        R: RangeBounds<OffsetDateTime> + Send,
    {
        let range = self
            .statuses
            .get(&source_id)
            .map(|m| m.range(timestamps).map(|(_, v)| v).copied().collect())
            .unwrap_or_default();
        Ok(range)
    }
}
