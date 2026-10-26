//! Business rules for the intake confirmation queue.
//!
//! Routes stay thin (HTTP concerns only); this module owns the rules from the
//! spec's "Voice Intake" and "Duplicate handling" sections. SQL lives in
//! [`repository`]. Every method that changes more than one row does so in a
//! single transaction and locks the request row first, so two browser tabs
//! racing to accept the same card cannot both succeed.

mod accept_undo_repository;
pub mod counts;
mod log;
mod polled;
pub mod repository;
pub mod restore_repository;
pub mod triage_repository;

pub use polled::PolledItem;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{
    GroceryItem, GroceryItemSource, IntakeSource, VoiceRequest, VoiceRequestStatus,
};
use crate::models::schemas::{
    AlexaIntakeCreate, VoiceRequestCreate, VoiceRequestDecision, MAX_QUANTITY,
};
use crate::services::grocery::{duplicate_error, repository as grocery_repository, OnDuplicate};
use crate::services::item_rules::{self, AddedVia};
use crate::services::triage::TriageQueue;
use crate::services::ws_hub::WsHub;

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

    /// Records a request from the generic webhook, starts its triage check and
    /// notifies open sessions.
    pub async fn create_webhook_request(
        &self,
        payload: &VoiceRequestCreate,
        triage: &TriageQueue,
    ) -> Result<VoiceRequest, ApiError> {
        let request = repository::insert_request(
            self.pool,
            IntakeSource::Webhook,
            None,
            &payload.item,
            payload.item.trim(),
            payload.quantity,
            triage.initial_status(),
        )
        .await?;
        self.after_recorded(&request, triage).await?;
        Ok(request)
    }

    /// Records a request forwarded by the Alexa bridge sidecar.
    ///
    /// Idempotent on `external_id`: a retried delivery returns the request
    /// created the first time rather than a second pending card. Alexa retries
    /// a skill endpoint it believes timed out, so this is the normal path, not
    /// an edge case. The returned [`Delivery`] says which happened, so the
    /// route can answer 201 for a new item and 200 for a retry.
    ///
    /// A retry is never classified again: the first delivery's check stands.
    pub async fn create_alexa_request(
        &self,
        payload: &AlexaIntakeCreate,
        triage: &TriageQueue,
    ) -> Result<(VoiceRequest, Delivery), ApiError> {
        if let Some(external_id) = payload.external_id.as_deref() {
            let existing =
                repository::find_by_external_id(self.pool, IntakeSource::Alexa, external_id)
                    .await?;
            if let Some(request) = existing {
                log::redelivered(&request, external_id);
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
            triage.initial_status(),
        )
        .await?;
        self.after_recorded(&request, triage).await?;
        Ok((request, Delivery::Recorded))
    }

    /// Requests waiting in Pending Requests, newest first. Requests triage
    /// held or rejected wait in the Triage view instead.
    pub async fn list_pending(&self) -> Result<Vec<VoiceRequest>, ApiError> {
        repository::list_pending(self.pool).await
    }

    /// Accepts a request, applying any corrections, onto the active list.
    ///
    /// When an active item with the same normalised name exists,
    /// `on_duplicate` decides: [`OnDuplicate::Ask`] returns
    /// [`ApiError::DuplicateActiveItem`] so the caller can ask the user whether
    /// to add another or update the existing quantity; [`OnDuplicate::Merge`]
    /// adds to the existing item's quantity, capped at [`MAX_QUANTITY`] to stay
    /// inside the column's constraint; [`OnDuplicate::Separate`] adds a second
    /// entry. A new list entry gets the chips of every item rule matching its
    /// (corrected) name; a merge leaves the existing item's chips alone.
    ///
    /// The returned flag is true when the accept merged into an existing item.
    ///
    /// Accepting is allowed from `pending` *and* `rejected` — the latter backs
    /// the web app's "dulled, undo-on-hover" reject. Only an already-accepted
    /// request is a closed decision.
    pub async fn accept(
        &self,
        request_id: Uuid,
        decision: &VoiceRequestDecision,
        user_id: &str,
        on_duplicate: OnDuplicate,
    ) -> Result<(VoiceRequest, GroceryItem, bool), ApiError> {
        let mut tx = self.pool.begin().await?;

        let request = repository::lock_request(&mut tx, request_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(request_id.to_string()))?;
        if request.status == VoiceRequestStatus::Accepted {
            return Err(ApiError::Conflict(request_id.to_string()));
        }

        let name = corrected_name(decision, &request);
        let quantity = decision.quantity.unwrap_or(request.parsed_quantity);

        let clash = grocery_repository::lock_active_duplicate(&mut tx, &name).await?;
        let (grocery_item, merged_quantity) = match (clash, on_duplicate) {
            (Some(existing), OnDuplicate::Ask) => {
                // Nothing has been written yet; rolling back just releases
                // the locks before the 409 goes out.
                tx.rollback().await?;
                log::duplicate_asked(&request, &existing);
                return Err(duplicate_error(&existing));
            }
            (Some(existing), OnDuplicate::Merge) => {
                let merged = existing.quantity.saturating_add(quantity).min(MAX_QUANTITY);
                let item =
                    grocery_repository::set_item_quantity(&mut tx, existing.id, merged).await?;
                (item, Some(merged - existing.quantity))
            }
            // No clash, or the user asked for both entries to exist.
            (_, _) => {
                // A voice item carries no note or chips of its own; its chips
                // come from the item rules matching its name, and the user
                // annotates it further on the list.
                let rule_terms =
                    item_rules::filter_terms_for(&mut *tx, &name, AddedVia::Voice).await?;
                let item = grocery_repository::insert_item(
                    &mut tx,
                    &name,
                    quantity,
                    GroceryItemSource::Voice,
                    None,
                    &rule_terms,
                    user_id,
                )
                .await?;
                (item, None)
            }
        };

        let accepted =
            repository::mark_accepted(&mut tx, request.id, &name, quantity, grocery_item.id)
                .await?;
        accept_undo_repository::set_merged_quantity(&mut tx, request.id, merged_quantity).await?;
        tx.commit().await?;
        log::accepted(&accepted, &grocery_item, merged_quantity.is_some(), user_id);

        self.publish_counts().await?;
        Ok((accepted, grocery_item, merged_quantity.is_some()))
    }

    /// Undoes an accept: takes back what it added to an existing item, or
    /// deletes the item it created, and returns the request to Pending
    /// Requests. An item already removed from the list is left removed.
    ///
    /// Only an accepted request can be undone; anything else is a conflict.
    pub async fn undo_accept(&self, request_id: Uuid) -> Result<VoiceRequest, ApiError> {
        let mut tx = self.pool.begin().await?;
        let request = repository::lock_request(&mut tx, request_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(request_id.to_string()))?;
        if request.status != VoiceRequestStatus::Accepted {
            return Err(ApiError::Conflict(request_id.to_string()));
        }

        let merged = accept_undo_repository::merged_quantity(&mut tx, request.id).await?;
        if let Some(item_id) = request.grocery_item_id {
            if let Some(item) = grocery_repository::lock_item(&mut tx, item_id).await? {
                match merged {
                    Some(added) => {
                        let back = (item.quantity - added).max(1);
                        grocery_repository::set_item_quantity(&mut tx, item.id, back).await?;
                    }
                    None => {
                        grocery_repository::delete_item(&mut tx, item.id).await?;
                    }
                }
            }
        }

        let restored = accept_undo_repository::mark_pending(&mut tx, request.id).await?;
        tx.commit().await?;
        log::accept_undone(&restored);

        self.publish_counts().await?;
        Ok(restored)
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
        log::rejected(&request);

        self.publish_counts().await?;
        Ok(request)
    }

    /// Logs a new request, starts its triage check, and updates the badges
    /// (a request triage skips is in Pending Requests already).
    async fn after_recorded(
        &self,
        request: &VoiceRequest,
        triage: &TriageQueue,
    ) -> Result<(), ApiError> {
        log::recorded(request);
        triage.start(request);
        self.publish_counts().await
    }

    /// Pushes the badge counts to every open browser session.
    async fn publish_counts(&self) -> Result<(), ApiError> {
        counts::publish(self.pool, self.hub).await
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
