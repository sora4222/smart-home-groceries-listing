//! Every SQL statement on `google_tasks_link`, the household's one link to
//! Google Tasks. Each write is an upsert on the `singleton` column, so the
//! row is created by whichever write comes first.
//!
//! Every statement is a literal with bind parameters.

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::GoogleTasksLink;

/// The link, if anything was ever saved.
pub async fn get<'e, E>(executor: E) -> Result<Option<GoogleTasksLink>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, GoogleTasksLink>(
        "SELECT id, refresh_token_encrypted, connected_at, task_list_id, task_list_title,
                enabled, poll_seconds, last_polled_at, last_poll_error
         FROM google_tasks_link",
    )
    .fetch_optional(executor)
    .await?)
}

/// Stores a new sign-in (already encrypted). The chosen list and polling
/// settings are kept, so reconnecting after a revoked sign-in picks up where
/// it left off.
pub async fn save_sign_in<'e, E>(executor: E, encrypted: &str) -> Result<GoogleTasksLink, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, GoogleTasksLink>(
        "INSERT INTO google_tasks_link (id, refresh_token_encrypted, connected_at)
         VALUES ($1, $2, now())
         ON CONFLICT (singleton) DO UPDATE
         SET refresh_token_encrypted = EXCLUDED.refresh_token_encrypted,
             connected_at = now(), last_poll_error = NULL, updated_at = now()
         RETURNING id, refresh_token_encrypted, connected_at, task_list_id, task_list_title,
                   enabled, poll_seconds, last_polled_at, last_poll_error",
    )
    .bind(Uuid::new_v4())
    .bind(encrypted)
    .fetch_one(executor)
    .await?)
}

/// Forgets the sign-in and stops polling. The chosen list is kept.
pub async fn clear_sign_in<'e, E>(executor: E) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE google_tasks_link
         SET refresh_token_encrypted = NULL, connected_at = NULL, enabled = false,
             updated_at = now()",
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// Saves the household's choices from `/settings/intake`.
pub async fn save_choices<'e, E>(
    executor: E,
    list: Option<(&str, &str)>,
    enabled: bool,
    poll_seconds: i32,
) -> Result<GoogleTasksLink, ApiError>
where
    E: PgExecutor<'e>,
{
    let (list_id, list_title) = match list {
        Some((id, title)) => (Some(id), Some(title)),
        None => (None, None),
    };
    Ok(sqlx::query_as::<_, GoogleTasksLink>(
        "INSERT INTO google_tasks_link (id, task_list_id, task_list_title, enabled, poll_seconds)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (singleton) DO UPDATE
         SET task_list_id = EXCLUDED.task_list_id,
             task_list_title = EXCLUDED.task_list_title,
             enabled = EXCLUDED.enabled,
             poll_seconds = EXCLUDED.poll_seconds,
             updated_at = now()
         RETURNING id, refresh_token_encrypted, connected_at, task_list_id, task_list_title,
                   enabled, poll_seconds, last_polled_at, last_poll_error",
    )
    .bind(Uuid::new_v4())
    .bind(list_id)
    .bind(list_title)
    .bind(enabled)
    .bind(poll_seconds)
    .fetch_one(executor)
    .await?)
}

/// Notes that a poll ran, and why it failed if it did.
pub async fn record_poll<'e, E>(executor: E, error: Option<&str>) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE google_tasks_link
         SET last_polled_at = now(), last_poll_error = $1, updated_at = now()",
    )
    .bind(error)
    .execute(executor)
    .await?;
    Ok(())
}
