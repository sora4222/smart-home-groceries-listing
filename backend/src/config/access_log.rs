//! Where the access-log file goes (`docs/features/FEATURE_ACCESS_LOGS.md`).
//!
//! Read on its own, before [`Settings`](super::Settings), because logging is
//! set up first so a bad setting can itself be logged.

use std::path::PathBuf;

use super::optional;

/// `ACCESS_LOG_DIR`: the folder for the rotating access-log files. Unset
/// means no file — the `access_logs` table is still written.
pub fn access_log_dir() -> Option<PathBuf> {
    optional("ACCESS_LOG_DIR").map(|dir| PathBuf::from(dir.trim()))
}
