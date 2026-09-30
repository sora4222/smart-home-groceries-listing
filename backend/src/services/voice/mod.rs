//! Business rules for the intake confirmation queue.
//!
//! Routes stay thin (HTTP concerns only); this module owns the rules from the
//! spec's "Voice Intake" and "Duplicate handling" sections. SQL lives in
//! [`repository`]. Every method that changes more than one row does so in a
//! single transaction and locks the request row first, so two browser tabs
//! racing to accept the same card cannot both succeed.

pub mod repository;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{
    GroceryItem, GroceryItemSource, IntakeSource, VoiceRequest, VoiceRequestStatus,
};
use crate::models::schemas::{
    AlexaIntakeCreate, VoiceRequestCreate, VoiceRequestDecision, MAX_QUANTITY,
};
use crate::services::grocery::{duplicate_error, repository as grocery_repository};
use crate::services::ws_hub::{ServerEvent, WsHub};

/// Whether an intake delivery produced a new request or matched one already
/// recorded under the same `external_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// A new pending request was created.
    Recorded,
    /// The channel re-delivered an item already in the queue; nothing changed.
    AlreadySeen,
}

/// Reads and writes the confirmation queue.
pub struct VoiceService<'a> {
    pool: &'a PgPool,
    hub: &'a WsHub,
}

impl<'a> VoiceService<'a> {
    /// Borrows the pool and event hub for the duration of one request.
    pub fn new(pool: &'a PgPool, hub: &'a WsHub) -> Self {
        Self { pool, hub }
    }

    /// Records a request from the generic webhook and notifies open sessions.
    pub async fn create_webhook_request(
        &self,
        payload: &VoiceRequestCreate,
    ) -> Result<VoiceRequest, ApiError> {
        let request = repository::insert_request(
            self.pool,
            IntakeSource::Webhook,
            None,
            &payload.item,
            payload.item.trim(),
            payload.quantity,
        )
        .await?;
        self.publish_pending_count().await?;
        Ok(request)
    }

    /// Records a request forwarded by the Alexa bridge sidecar.
    ///
    /// Idempotent on `external_id`: a retried delivery returns the request
    /// created the first time rather than a second pending card. Alexa retries
    /// a skill endpoint it believes timed out, so this is the normal path, not
    /// an edge case. The returned [`Delivery`] says which happened, so the
    /// route can answer 201 for a new item and 200 for a retry.
    pub async fn create_alexa_request(
        &self,
        payload: &AlexaIntakeCreate,
    ) -> Result<(VoiceRequest, Delivery), ApiError> {
        if let Some(external_id) = payload.external_id.as_deref() {
            let existing =
                repository::find_by_external_id(self.pool, IntakeSource::Alexa, external_id)
                    .await?;
            if let Some(request) = existing {
                tracing::info!(
                    external_id,
                    request_id = %request.id,
                    "ignoring re-delivered Alexa item"
                );
                return Ok((request, Delivery::AlreadySeen));
            }
        }

        let request = repository::insert_request(
            self.pool,
            IntakeSource::Alexa,
            payload.external_id.as_deref(),
            payload.raw_text.as_deref().unwrap_or(&payload.item),
            payload.item.trim(),
            payload.quantity,
        )
        .await?;
        self.publish_pending_count().await?;
        Ok((request, Delivery::Recorded))
    }

    /// Requests still awaiting a decision, newest first.
    pub async fn list_pending(&self) -> Result<Vec<VoiceRequest>, ApiError> {
        repository::list_pending(self.pool).await
    }

    /// Accepts a request, applying any corrections, onto the active list.
    ///
    /// Returns [`ApiError::DuplicateActiveItem`] when an active item with the
    /// same normalised name exists and `merge` is false, so the caller can ask
    /// the user whether to add another or update the existing quantity.
    /// `merge = true` adds to the existing item's quantity instead, capped at
    /// [`MAX_QUANTITY`] to stay inside the column's constraint.
    ///
    /// Accepting is allowed from `pending` *and* `rejected` — the latter backs
    /// the web app's "dulled, undo-on-hover" reject. Only an already-accepted
    /// request is a closed decision.
    pub async fn accept(
        &self,
        request_id: Uuid,
        decision: &VoiceRequestDecision,
        user_id: &str,
        merge: bool,
    ) -> Result<(VoiceRequest, GroceryItem), ApiError> {
        let mut tx = self.pool.begin().await?;

        let request = repository::lock_request(&mut tx, request_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(request_id.to_string()))?;
        if request.status == VoiceRequestStatus::Accepted {
            return Err(ApiError::Conflict(request_id.to_string()));
        }

        let name = corrected_name(decision, &request);
        let quantity = decision.quantity.unwrap_or(request.parsed_quantity);

        let grocery_item = match grocery_repository::lock_active_duplicate(&mut tx, &name).await? {
            Some(existing) if !merge => {
                // Nothing has been written yet; rolling back just releases
                // the locks before the 409 goes out.
                tx.rollback().await?;
                return Err(duplicate_error(&existing));
            }
            Some(existing) => {
                let merged = existing.quantity.saturating_add(quantity).min(MAX_QUANTITY);
                grocery_repository::set_item_quantity(&mut tx, existing.id, merged).await?
            }
            None => {
                // A voice item carries no note or chips of its own; the user
                // annotates it on the list once it is there.
                grocery_repository::insert_item(
                    &mut tx,
                    &name,
                    quantity,
                    GroceryItemSource::Voice,
                    None,
                    &[],
                    user_id,
                )
                .await?
            }
        };

        let accepted =
            repository::mark_accepted(&mut tx, request.id, &name, quantity, grocery_item.id)
                .await?;
        tx.commit().await?;

        self.publish_pending_count().await?;
        Ok((accepted, grocery_item))
    }

    /// Rejects a pending request. Rejections are recorded, never deleted, so
    /// the web app can offer the undo path back through [`Self::accept`].
    pub async fn reject(&self, request_id: Uuid) -> Result<VoiceRequest, ApiError> {
        let Some(request) = repository::reject_if_pending(self.pool, request_id).await? else {
            // Distinguish "no such request" from "already decided": the web
            // app shows different messages for a stale card and a missing one.
            return Err(
                if repository::request_exists(self.pool, request_id).await? {
                    ApiError::Conflict(request_id.to_string())
                } else {
                    ApiError::NotFound(request_id.to_string())
                },
            );
        };

        self.publish_pending_count().await?;
        Ok(request)
    }

    /// Counts requests still awaiting a decision.
    pub async fn pending_count(&self) -> Result<i64, ApiError> {
        repository::pending_count(self.pool).await
    }

    /// Pushes the current pending count to every open browser session.
    async fn publish_pending_count(&self) -> Result<(), ApiError> {
        let count = self.pending_count().await?;
        self.hub.broadcast(ServerEvent::VoiceRequestAdded { count });
        Ok(())
    }
}

/// The name to file the item under: the user's correction when they made one,
/// otherwise what the intake channel heard. A blank correction is ignored
/// rather than treated as a deliberate empty name.
fn corrected_name(decision: &VoiceRequestDecision, request: &VoiceRequest) -> String {
    decision
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(&request.parsed_name)
        .to_string()
}
