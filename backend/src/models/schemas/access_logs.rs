//! Bodies for the `/logs` page: `GET /api/access-logs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::access_log_rows::AccessLogRow;
use crate::services::access_log::AccessLogPage;

/// `?before=&limit=&hide_health_checks=` — which page to list.
#[derive(Debug, Default, Deserialize)]
pub struct AccessLogQuery {
    /// Only rows older than this id: the previous page's `next_before`.
    pub before: Option<i64>,
    /// Rows wanted; the service clamps it.
    pub limit: Option<i64>,
    /// Leave out the container's health probe. Defaults to true.
    pub hide_health_checks: Option<bool>,
}

/// One answered request as the web app sees it.
#[derive(Debug, Serialize)]
pub struct AccessLogResponse {
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

impl From<AccessLogRow> for AccessLogResponse {
    fn from(row: AccessLogRow) -> Self {
        Self {
            id: row.id,
            occurred_at: row.occurred_at,
            source_ip: row.source_ip,
            forwarded_for: row.forwarded_for,
            user_id: row.user_id,
            method: row.method,
            path: row.path,
            status_code: row.status_code,
            duration_ms: row.duration_ms,
        }
    }
}

/// One page of the access log, newest first.
#[derive(Debug, Serialize)]
pub struct AccessLogPageResponse {
    pub entries: Vec<AccessLogResponse>,
    /// Pass as `before` to get the next, older page. `null` when this page
    /// was not full, so there is nothing older.
    pub next_before: Option<i64>,
}

impl From<AccessLogPage> for AccessLogPageResponse {
    fn from(page: AccessLogPage) -> Self {
        Self {
            entries: page.rows.into_iter().map(Into::into).collect(),
            next_before: page.next_before,
        }
    }
}
