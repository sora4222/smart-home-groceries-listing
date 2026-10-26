//! SQL for undoing an accepted request: how much an accept merged into an
//! existing item, and putting the request back in the queue.
//!
//! Every statement is a literal with bind parameters.

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::VoiceRequest;

/// Records how much an accept added to an existing item (`None` = it created one).
pub async fn set_merged_quantity(
    tx: &mut Transaction<'_, Postgres>,
    request_id: Uuid,
    merged_quantity: Option<i32>,
) -> Result<(), ApiError> {
    sqlx::query("UPDATE voice_requests SET merged_quantity = $2 WHERE id = $1")
        .bind(request_id)
        .bind(merged_quantity)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// How much the request's accept added to an existing item, if it merged.
pub async fn merged_quantity(
    tx: &mut Transaction<'_, Postgres>,
    request_id: Uuid,
) -> Result<Option<i32>, ApiError> {
    Ok(
        sqlx::query_scalar("SELECT merged_quantity FROM voice_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(&mut **tx)
            .await?,
    )
}

/// Puts an accepted request back in the queue, forgetting its item.
pub async fn mark_pending(
    tx: &mut Transaction<'_, Postgres>,
    request_id: Uuid,
) -> Result<VoiceRequest, ApiError> {
    Ok(sqlx::query_as::<_, VoiceRequest>(
        "UPDATE voice_requests
         SET status = 'pending', grocery_item_id = NULL, merged_quantity = NULL
         WHERE id = $1
         RETURNING id, source, external_id, raw_text, parsed_name, parsed_quantity, status,
                   grocery_item_id, created_at, triage_status, triage_reason,
                   triage_confidence",
    )
    .bind(request_id)
    .fetch_one(&mut **tx)
    .await?)
}
