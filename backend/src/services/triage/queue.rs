//! Checking intake requests in the background, after they are recorded.
//!
//! An intake channel is answered as soon as its item is stored; the
//! classifier runs afterwards. The Alexa bridge waits only a few seconds for
//! the backend, and a local model can take longer than that to load, so
//! classifying first would lose items whenever the model is slow.
//! A request is `unchecked` until its check is stored; one left unchecked by
//! a restart is checked again at startup ([`TriageQueue::recheck_unchecked`]).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::assessor::Triage;
use super::log;
use crate::error::ApiError;
use crate::models::db::{TriageStatus, VoiceRequest};
use crate::services::voice::{counts, triage_repository};
use crate::services::ws_hub::WsHub;

/// Starts and stores triage checks. Cheap to clone.
#[derive(Clone)]
pub struct TriageQueue {
    triage: Arc<Triage>,
    pool: PgPool,
    hub: WsHub,
}

impl TriageQueue {
    /// A queue writing to `pool` and announcing on `hub`.
    pub fn new(triage: Triage, pool: PgPool, hub: WsHub) -> Self {
        Self {
            triage: Arc::new(triage),
            pool,
            hub,
        }
    }

    /// The status a newly recorded request starts in.
    pub fn initial_status(&self) -> TriageStatus {
        if self.triage.is_enabled() {
            TriageStatus::Unchecked
        } else {
            TriageStatus::Skipped
        }
    }

    /// Checks a just-recorded request in the background, if it is waiting
    /// for one. The task outlives the HTTP request that recorded the item.
    pub fn start(&self, request: &VoiceRequest) {
        if request.triage_status != TriageStatus::Unchecked {
            return;
        }
        let queue = self.clone();
        let (id, item) = (request.id, request.parsed_name.clone());
        tokio::spawn(async move {
            if let Err(err) = queue.check(id, &item).await {
                log::check_failed(id, &err);
            }
        });
    }

    /// Classifies one request, stores the outcome and updates the badges.
    pub async fn check(&self, request_id: Uuid, item_text: &str) -> Result<(), ApiError> {
        let outcome = self.triage.assess(item_text).await;
        let stored = triage_repository::record_outcome(
            &self.pool,
            request_id,
            outcome.status,
            outcome.reason.as_deref(),
            outcome.confidence,
        )
        .await?;
        if let Some(request) = stored {
            log::assessed(&request);
            counts::publish(&self.pool, &self.hub).await?;
        }
        Ok(())
    }

    /// Checks every request a restart left `unchecked`, one at a time.
    ///
    /// Only requests recorded before `started_at` (this process's start):
    /// a newer one already has its own check running from [`Self::start`].
    pub async fn recheck_unchecked(&self, started_at: DateTime<Utc>) -> Result<usize, ApiError> {
        let waiting = triage_repository::list_unchecked_before(&self.pool, started_at).await?;
        if !waiting.is_empty() {
            log::rechecking(waiting.len());
        }
        for request in &waiting {
            self.check(request.id, &request.parsed_name).await?;
        }
        Ok(waiting.len())
    }
}
