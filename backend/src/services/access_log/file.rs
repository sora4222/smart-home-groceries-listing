//! The access log's line in the log file.
//!
//! Lines are `tracing` events under [`ACCESS_LOG_TARGET`]. `logging.rs`
//! sends that target, and only that target, to the rotating access-log file.

use super::AccessLogEntry;

/// The `tracing` target every access-log line is written under.
pub const ACCESS_LOG_TARGET: &str = "access_log";

/// Writes one request to the access-log file.
pub fn write_line(entry: &AccessLogEntry) {
    tracing::info!(
        target: ACCESS_LOG_TARGET,
        ip = %entry.source_ip.as_deref().unwrap_or("-"),
        forwarded_for = %entry.forwarded_for.as_deref().unwrap_or("-"),
        user = %entry.user_id,
        method = %entry.method,
        path = %entry.path,
        status = entry.status_code,
        duration_ms = entry.duration_ms,
        "request"
    );
}
