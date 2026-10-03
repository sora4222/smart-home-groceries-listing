//! Reading `access_logs` straight from the database in tests.
//!
//! The middleware writes each row on a background task after the response,
//! so a test waits for the rows it expects instead of reading at once. The
//! wait has a deadline, so a missing row fails the test rather than hanging.

use std::time::{Duration, Instant};

use sqlx::PgPool;

/// How long a test waits for background rows before failing.
const WAIT_LIMIT: Duration = Duration::from_secs(5);
/// How often the table is re-read while waiting.
const POLL_EVERY: Duration = Duration::from_millis(20);

/// One stored row, in the columns the tests check.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoggedRequest {
    pub source_ip: Option<String>,
    pub forwarded_for: Option<String>,
    pub user_id: String,
    pub method: String,
    pub path: String,
    pub status_code: i16,
}

/// Every stored row, oldest first.
pub async fn logged_requests(pool: &PgPool) -> Vec<LoggedRequest> {
    sqlx::query_as::<_, LoggedRequest>(
        "SELECT source_ip, forwarded_for, user_id, method, path, status_code
         FROM access_logs ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .expect("reading access_logs")
}

/// Waits until at least `count` rows are stored, then returns them all.
pub async fn wait_for_logged(pool: &PgPool, count: usize) -> Vec<LoggedRequest> {
    let deadline = Instant::now() + WAIT_LIMIT;
    loop {
        let rows = logged_requests(pool).await;
        if rows.len() >= count {
            return rows;
        }
        assert!(
            Instant::now() < deadline,
            "expected {count} access log rows, found {}: {rows:?}",
            rows.len()
        );
        tokio::time::sleep(POLL_EVERY).await;
    }
}

/// Waits for the one row logged for `path`.
pub async fn wait_for_path(pool: &PgPool, path: &str) -> LoggedRequest {
    let deadline = Instant::now() + WAIT_LIMIT;
    loop {
        if let Some(row) = logged_requests(pool)
            .await
            .into_iter()
            .find(|r| r.path == path)
        {
            return row;
        }
        assert!(Instant::now() < deadline, "no access log row for {path}");
        tokio::time::sleep(POLL_EVERY).await;
    }
}
