//! Every SQL statement on `intake_restores`: items a person put back in
//! their source list from the Triage view. The next poll that sees the new
//! item records it with triage skipped (the spec's `force_accept`).
//!
//! Every statement is a literal with bind parameters.

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::IntakeSource;

/// Notes that `external_id` on `source` was put back from `restored_from`.
pub async fn insert<'e, E>(
    executor: E,
    source: IntakeSource,
    external_id: &str,
    restored_from: Uuid,
    restored_by: &str,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO intake_restores (id, source, external_id, restored_from, restored_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(source)
    .bind(external_id)
    .bind(restored_from)
    .bind(restored_by)
    .execute(executor)
    .await?;
    Ok(())
}

/// Uses up the restore mark on an item. True when the item was put back by a
/// person and this is the first time it is seen since.
pub async fn take<'e, E>(
    executor: E,
    source: IntakeSource,
    external_id: &str,
) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let taken: Option<Uuid> = sqlx::query_scalar(
        "UPDATE intake_restores SET used_at = now()
         WHERE source = $1 AND external_id = $2 AND used_at IS NULL
         RETURNING id",
    )
    .bind(source)
    .bind(external_id)
    .fetch_optional(executor)
    .await?;
    Ok(taken.is_some())
}
