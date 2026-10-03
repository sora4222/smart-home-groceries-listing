//! The SQL behind the triage step: the `triage_*` columns of `voice_requests`.
//!
//! Split from [`super::repository`] only to keep each file short; both own the
//! same table and nothing outside `services/voice/` and `services/triage/`
//! writes it. Every statement is a literal with bind parameters.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{TriageStatus, VoiceRequest};

/// Stores the classifier's outcome, but only on a request still `unchecked`.
/// `None` means another check got there first (the startup re-check and the
/// check started on arrival can race), or the request is gone.
pub async fn record_outcome<'e, E>(
    executor: E,
    request_id: Uuid,
    status: TriageStatus,
    reason: Option<&str>,
    confidence: Option<f32>,
) -> Result<Option<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "UPDATE voice_requests
         SET triage_status = $2, triage_reason = $3, triage_confidence = $4
         WHERE id = $1 AND triage_status = 'unchecked'
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at, triage_status, triage_reason,
                   triage_confidence",
    )
    .bind(request_id)
    .bind(status)
    .bind(reason)
    .bind(confidence)
    .fetch_optional(executor)
    .await?)
}

/// Undecided requests recorded before `before` that the classifier has not
/// answered for yet, oldest first.
pub async fn list_unchecked_before<'e, E>(
    executor: E,
    before: DateTime<Utc>,
) -> Result<Vec<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "SELECT id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                grocery_item_id, created_at, triage_status, triage_reason, triage_confidence
         FROM voice_requests
         WHERE status = 'pending' AND triage_status = 'unchecked' AND created_at < $1
         ORDER BY created_at",
    )
    .bind(before)
    .fetch_all(executor)
    .await?)
}

/// Undecided requests triage left in one Triage-view tab (`held` or
/// `rejected`), newest first.
pub async fn list_in_tab<'e, E>(
    executor: E,
    tab: TriageStatus,
) -> Result<Vec<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "SELECT id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                grocery_item_id, created_at, triage_status, triage_reason, triage_confidence
         FROM voice_requests
         WHERE status = 'pending' AND triage_status = $1
         ORDER BY created_at DESC",
    )
    .bind(tab)
    .fetch_all(executor)
    .await?)
}

/// Counts undecided requests held for review — the Triage nav badge.
pub async fn held_count<'e, E>(executor: E) -> Result<i64, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_scalar(
        "SELECT count(*) FROM voice_requests
         WHERE status = 'pending' AND triage_status = 'held'",
    )
    .fetch_one(executor)
    .await?)
}

/// A person overrides triage: a held or rejected request moves on to Pending
/// Requests as `skipped`. The classifier's reason is kept. `None` means no
/// undecided held or rejected request has this id.
pub async fn move_to_pending<'e, E>(
    executor: E,
    request_id: Uuid,
) -> Result<Option<VoiceRequest>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "UPDATE voice_requests
         SET triage_status = 'skipped'
         WHERE id = $1 AND status = 'pending' AND triage_status IN ('held', 'rejected')
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at, triage_status, triage_reason,
                   triage_confidence",
    )
    .bind(request_id)
    .fetch_optional(executor)
    .await?)
}

/// Closes a request a person put back in its source list: it leaves the
/// Triage view as rejected. The new item in the source is a new request.
pub async fn close_restored<'e, E>(executor: E, request_id: Uuid) -> Result<VoiceRequest, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "UPDATE voice_requests SET status = 'rejected'
         WHERE id = $1
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at, triage_status, triage_reason,
                   triage_confidence",
    )
    .bind(request_id)
    .fetch_one(executor)
    .await?)
}
