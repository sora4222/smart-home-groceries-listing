//! Recording items polled from a source list (Google Tasks).
//!
//! Split from the parent module only for length; it is one more way into the
//! same confirmation queue, under the same rules.

use super::{log, repository, restore_repository, Delivery, VoiceService};
use crate::error::ApiError;
use crate::models::db::{IntakeSource, TriageStatus, VoiceRequest};
use crate::services::triage::TriageQueue;

/// One item read from a source list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolledItem {
    /// The source's id for the item. Makes a re-poll a no-op.
    pub external_id: String,
    /// What the person typed, e.g. "2 oat milk".
    pub raw_text: String,
    /// The item name read from it, e.g. "oat milk".
    pub name: String,
    pub quantity: i32,
}

impl VoiceService<'_> {
    /// Records an item polled from `source`.
    ///
    /// Idempotent on `external_id`: an item seen before (its delete after the
    /// last poll failed) returns the request recorded the first time. An item
    /// a person put back from the Triage view skips triage and goes straight
    /// to Pending Requests — still waiting for someone to accept it.
    ///
    /// The caller polls one source at a time, so two polls never race on the
    /// same `external_id`.
    pub async fn record_polled(
        &self,
        source: IntakeSource,
        item: &PolledItem,
        triage: &TriageQueue,
    ) -> Result<(VoiceRequest, Delivery), ApiError> {
        if let Some(request) =
            repository::find_by_external_id(self.pool, source, &item.external_id).await?
        {
            log::redelivered(&request, &item.external_id);
            return Ok((request, Delivery::AlreadySeen));
        }

        let mut tx = self.pool.begin().await?;
        let restored = restore_repository::take(&mut *tx, source, &item.external_id).await?;
        let triage_status = if restored {
            TriageStatus::Skipped
        } else {
            triage.initial_status()
        };
        let request = repository::insert_request(
            &mut *tx,
            source,
            Some(&item.external_id),
            &item.raw_text,
            &item.name,
            item.quantity,
            triage_status,
        )
        .await?;
        tx.commit().await?;

        self.after_recorded(&request, triage).await?;
        Ok((request, Delivery::Recorded))
    }
}
