//! Row type for `access_logs` (migration `0008`).

use chrono::{DateTime, Utc};

/// One answered request, as stored.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccessLogRow {
    pub id: i64,
    pub occurred_at: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub forwarded_for: Option<String>,
    pub user_id: String,
    pub method: String,
    pub path: String,
    pub status_code: i16,
    pub duration_ms: i32,
}
