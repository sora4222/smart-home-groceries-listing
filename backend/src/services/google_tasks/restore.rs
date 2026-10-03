//! Triage's "Restore to source": put an item the checker held or rejected
//! back on the household's Google Tasks list, so the request is not lost.
//!
//! The new task is marked (`intake_restores`) so the next poll records it with
//! triage skipped — it lands in Pending Requests, where a person still
//! accepts it. The original request is closed as rejected.

use sqlx::PgPool;
use uuid::Uuid;

use super::registry::GoogleTasks;
use super::{link_repository, log, sign_in};
use crate::error::ApiError;
use crate::models::db::{IntakeSource, TriageStatus, VoiceRequest, VoiceRequestStatus};
use crate::services::encryption::Encryptor;
use crate::services::voice::{counts, repository, restore_repository, triage_repository};
use crate::services::ws_hub::WsHub;

/// Puts triaged Google Tasks items back on their list.
pub struct TasksRestore<'a> {
    pool: &'a PgPool,
    hub: &'a WsHub,
    google: &'a GoogleTasks,
    encryptor: Option<&'a Encryptor>,
}

impl<'a> TasksRestore<'a> {
    /// Borrows what one request needs.
    pub fn new(
        pool: &'a PgPool,
        hub: &'a WsHub,
        google: &'a GoogleTasks,
        encryptor: Option<&'a Encryptor>,
    ) -> Self {
        Self {
            pool,
            hub,
            google,
            encryptor,
        }
    }

    /// Restores one request to Google Tasks and returns it, now closed.
    ///
    /// 404 for an unknown id. 409 when the request is not held or rejected by
    /// triage, was already decided, did not come from Google Tasks, or Google
    /// Tasks is not connected to a list. When Google refuses, nothing changes.
    ///
    /// Holds the poll lock throughout, so a poll cannot pick up the new task
    /// before it is marked.
    pub async fn restore(&self, request_id: Uuid, user_id: &str) -> Result<VoiceRequest, ApiError> {
        let _no_poll_meanwhile = self.google.poll_lock.lock().await;
        let mut tx = self.pool.begin().await?;

        let request = repository::lock_request(&mut tx, request_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(request_id.to_string()))?;
        check_restorable(&request)?;

        let link = link_repository::get(&mut *tx)
            .await?
            .ok_or_else(sign_in::not_connected)?;
        let list_id = link
            .task_list_id
            .clone()
            .ok_or_else(|| ApiError::Conflict("Pick a Google Tasks list first".into()))?;
        let token = sign_in::open(self.encryptor, &link)?;

        let task = self
            .google
            .api
            .insert_task(&token, &list_id, &request.raw_text)
            .await?;
        restore_repository::insert(&mut *tx, IntakeSource::Tasks, &task.id, request.id, user_id)
            .await?;
        let closed = triage_repository::close_restored(&mut *tx, request.id).await?;
        tx.commit().await?;

        log::restored(&closed, &task.id, user_id);
        counts::publish(self.pool, self.hub).await?;
        Ok(closed)
    }
}

/// Only an undecided Google Tasks request in a Triage tab can go back.
fn check_restorable(request: &VoiceRequest) -> Result<(), ApiError> {
    let in_triage = matches!(
        request.triage_status,
        TriageStatus::Held | TriageStatus::Rejected
    );
    if request.status != VoiceRequestStatus::Pending || !in_triage {
        return Err(ApiError::Conflict(request.id.to_string()));
    }
    if request.source != IntakeSource::Tasks {
        return Err(ApiError::Conflict(
            "Only Google Tasks items can be put back where they came from".into(),
        ));
    }
    Ok(())
}
