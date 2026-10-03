//! Every SQL statement the access log needs. Only `access_logs` is touched,
//! and every statement is a literal with bind parameters.

use sqlx::PgPool;

use super::AccessLogEntry;
use crate::error::ApiError;
use crate::models::access_log_rows::AccessLogRow;

/// Stores one answered request.
pub async fn insert(pool: &PgPool, entry: &AccessLogEntry) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO access_logs
             (source_ip, forwarded_for, user_id, method, path, status_code, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(entry.source_ip.as_deref())
    .bind(entry.forwarded_for.as_deref())
    .bind(&entry.user_id)
    .bind(&entry.method)
    .bind(&entry.path)
    .bind(i16::try_from(entry.status_code).unwrap_or(i16::MAX))
    .bind(i32::try_from(entry.duration_ms).unwrap_or(i32::MAX))
    .execute(pool)
    .await?;
    Ok(())
}

/// Up to `limit` requests, newest first, older than `before` when given.
/// `skip_path`, when given, leaves out requests to that exact path.
pub async fn list_page(
    pool: &PgPool,
    before: Option<i64>,
    limit: i64,
    skip_path: Option<&str>,
) -> Result<Vec<AccessLogRow>, ApiError> {
    Ok(sqlx::query_as::<_, AccessLogRow>(
        "SELECT id, occurred_at, source_ip, forwarded_for, user_id, method, path,
                status_code, duration_ms
         FROM access_logs
         WHERE ($1::bigint IS NULL OR id < $1)
           AND ($3::text IS NULL OR path <> $3)
         ORDER BY id DESC
         LIMIT $2",
    )
    .bind(before)
    .bind(limit)
    .bind(skip_path)
    .fetch_all(pool)
    .await?)
}
