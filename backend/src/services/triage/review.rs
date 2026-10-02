//! The Triage view's rules: what is in each tab, and moving a request on.
//!
//! Confirming a rejection is the queue's ordinary reject
//! ([`crate::services::voice::VoiceService::reject`]) — it already records
//! the request as rejected and updates both badges.

use sqlx::PgPool;
use uuid::Uuid;

use super::log;
use crate::error::ApiError;
use crate::models::db::{TriageStatus, VoiceRequest};
use crate::services::voice::{counts, repository, triage_repository};
use crate::services::ws_hub::WsHub;

/// The two tabs of the Triage view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriageTab {
    /// Low confidence or a classifier failure; needs a person.
    Held,
    /// The classifier said "not a supermarket item".
    Rejected,
}

impl TriageTab {
    fn status(self) -> TriageStatus {
        match self {
            Self::Held => TriageStatus::Held,
            Self::Rejected => TriageStatus::Rejected,
        }
    }
}

/// Reads and acts on the Triage view.
pub struct TriageReview<'a> {
    pool: &'a PgPool,
    hub: &'a WsHub,
}

impl<'a> TriageReview<'a> {
    /// Borrows the pool and event hub for the duration of one request.
    pub fn new(pool: &'a PgPool, hub: &'a WsHub) -> Self {
        Self { pool, hub }
    }

    /// Undecided requests in one tab, newest first.
    pub async fn list(&self, tab: TriageTab) -> Result<Vec<VoiceRequest>, ApiError> {
        triage_repository::list_in_tab(self.pool, tab.status()).await
    }

    /// A person overrides triage: the request moves on to Pending Requests,
    /// where it still waits for someone to accept it onto the list.
    ///
    /// 404 for an unknown id; 409 for a request that is not held or rejected
    /// by triage, or that someone already decided.
    pub async fn move_to_pending(
        &self,
        request_id: Uuid,
        user_id: &str,
    ) -> Result<VoiceRequest, ApiError> {
        let Some(request) = triage_repository::move_to_pending(self.pool, request_id).await? else {
            return Err(
                if repository::request_exists(self.pool, request_id).await? {
                    ApiError::Conflict(request_id.to_string())
                } else {
                    ApiError::NotFound(request_id.to_string())
                },
            );
        };
        log::moved_to_pending(&request, user_id);
        counts::publish(self.pool, self.hub).await?;
        Ok(request)
    }
}
