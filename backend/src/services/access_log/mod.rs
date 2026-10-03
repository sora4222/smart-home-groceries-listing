//! Network access logging (`docs/features/FEATURE_ACCESS_LOGS.md`).
//!
//! Every answered request is written twice: a line in the rotating log file
//! ([`file`]) and a row in `access_logs` ([`repository`]) for the `/logs`
//! page. The middleware that gathers each [`AccessLogEntry`] lives in
//! `crate::middleware::access_log`.

mod entry;
pub mod file;
mod repository;

use sqlx::PgPool;

pub use entry::{user_label, AccessLogEntry, UNAUTHENTICATED};

use crate::error::ApiError;
use crate::models::access_log_rows::AccessLogRow;

/// The container health probe's path. It is asked every few seconds, so the
/// `/logs` page can hide it.
pub const HEALTH_CHECK_PATH: &str = "/api/health";

/// Rows the `/logs` page gets when it does not ask for a number.
pub const DEFAULT_PAGE_SIZE: i64 = 100;
/// The most rows one page may hold.
pub const MAX_PAGE_SIZE: i64 = 500;

/// Records one answered request: the file line now, the table row in the
/// background.
///
/// The row is written on its own task so a slow or missing database never
/// holds up a response. A failed write is logged and dropped — the file line
/// is still there.
pub fn record(pool: &PgPool, entry: AccessLogEntry) {
    file::write_line(&entry);
    let pool = pool.clone();
    tokio::spawn(async move {
        if let Err(err) = repository::insert(&pool, &entry).await {
            tracing::warn!(%err, path = %entry.path, "could not store an access log row");
        }
    });
}

/// One page of the access log, newest first.
#[derive(Debug)]
pub struct AccessLogPage {
    pub rows: Vec<AccessLogRow>,
    /// The id to ask for older rows with, or `None` when there are none.
    pub next_before: Option<i64>,
}

/// One page of the access log, newest first.
///
/// `limit` is clamped to `1..=MAX_PAGE_SIZE`; `hide_health_checks` leaves out
/// the container's health probe.
pub async fn recent(
    pool: &PgPool,
    before: Option<i64>,
    limit: Option<i64>,
    hide_health_checks: bool,
) -> Result<AccessLogPage, ApiError> {
    let limit = page_size(limit);
    let skip_path = hide_health_checks.then_some(HEALTH_CHECK_PATH);
    let rows = repository::list_page(pool, before, limit, skip_path).await?;
    Ok(AccessLogPage {
        next_before: next_before(&rows, limit),
        rows,
    })
}

/// The cursor for the next, older page: the last row's id when the page came
/// back full. A short page means nothing older is left.
fn next_before(rows: &[AccessLogRow], limit: i64) -> Option<i64> {
    let full = i64::try_from(rows.len()).is_ok_and(|len| len >= limit);
    full.then(|| rows.last().map(|row| row.id)).flatten()
}

/// The number of rows to fetch for a requested page size.
fn page_size(requested: Option<i64>) -> i64 {
    requested
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_size_defaults_when_not_asked() {
        assert_eq!(page_size(None), DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn page_size_is_clamped_at_both_ends() {
        assert_eq!(page_size(Some(0)), 1);
        assert_eq!(page_size(Some(-5)), 1);
        assert_eq!(page_size(Some(10_000)), MAX_PAGE_SIZE);
        assert_eq!(page_size(Some(25)), 25);
    }

    fn rows_with_ids(ids: &[i64]) -> Vec<AccessLogRow> {
        ids.iter()
            .map(|&id| AccessLogRow {
                id,
                occurred_at: chrono::Utc::now(),
                source_ip: None,
                forwarded_for: None,
                user_id: UNAUTHENTICATED.to_string(),
                method: "GET".to_string(),
                path: "/api/health".to_string(),
                status_code: 200,
                duration_ms: 1,
            })
            .collect()
    }

    #[test]
    fn a_full_page_points_at_its_oldest_row() {
        assert_eq!(next_before(&rows_with_ids(&[9, 8, 7]), 3), Some(7));
    }

    #[test]
    fn a_short_or_empty_page_has_nothing_older() {
        assert_eq!(next_before(&rows_with_ids(&[9, 8]), 3), None);
        assert_eq!(next_before(&[], 3), None);
    }
}
