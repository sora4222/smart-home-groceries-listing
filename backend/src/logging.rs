//! Process-wide logging: the usual console output, plus the access-log file.
//!
//! The console follows `RUST_LOG`. Access-log lines (target
//! [`ACCESS_LOG_TARGET`]) go only to a daily file in `ACCESS_LOG_DIR`, kept
//! for [`ACCESS_LOG_FILES_KEPT`] days. The console already shows each request
//! through `tower_http`'s trace layer, so it does not repeat them.

use std::path::Path;

use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

use crate::services::access_log::file::ACCESS_LOG_TARGET;

/// How many daily access-log files are kept before the oldest is deleted.
pub const ACCESS_LOG_FILES_KEPT: usize = 30;

/// The console's filter when `RUST_LOG` is unset.
const DEFAULT_CONSOLE_FILTER: &str = "grocery_backend=info,tower_http=info,warn";

/// Starts logging. Keep the returned guard alive for the life of the process:
/// dropping it stops the file writer, and lines still queued are lost.
///
/// When the folder cannot be used, the console says so and the process runs
/// on without the file — the `access_logs` table still records every request.
pub fn init(access_log_dir: Option<&Path>) -> Option<WorkerGuard> {
    let console_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(DEFAULT_CONSOLE_FILTER));
    let console = fmt::layer().with_filter(console_filter);

    let (file, guard, problem) = match access_log_dir.map(access_log_writer) {
        Some(Ok((writer, guard))) => (Some(access_log_layer(writer)), Some(guard), None),
        Some(Err(err)) => (None, None, Some(err)),
        None => (None, None, None),
    };

    // `try_init` rather than `init`: a second call must not abort the process.
    let _ = tracing_subscriber::registry()
        .with(console)
        .with(file)
        .try_init();

    if let Some(err) = problem {
        tracing::warn!(%err, "access-log file unavailable; requests are still stored in the database");
    }
    guard
}

/// The layer that writes access-log lines, and nothing else, to `writer`.
fn access_log_layer<S, W>(writer: W) -> impl Layer<S>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
    W: for<'writer> fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    let only_access_lines = Targets::new().with_target(ACCESS_LOG_TARGET, LevelFilter::INFO);
    fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .with_filter(only_access_lines)
}

/// A non-blocking writer to the daily access-log file in `dir`.
fn access_log_writer(
    dir: &Path,
) -> Result<(tracing_appender::non_blocking::NonBlocking, WorkerGuard), String> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("access")
        .filename_suffix("log")
        .max_log_files(ACCESS_LOG_FILES_KEPT)
        .build(dir)
        .map_err(|err| format!("{}: {err}", dir.display()))?;
    Ok(tracing_appender::non_blocking(appender))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::access_log::{file::write_line, AccessLogEntry};

    fn entry() -> AccessLogEntry {
        AccessLogEntry {
            source_ip: Some("192.168.1.20".to_string()),
            forwarded_for: None,
            user_id: "user_abc".to_string(),
            method: "GET".to_string(),
            path: "/api/grocery-items".to_string(),
            status_code: 200,
            duration_ms: 4,
        }
    }

    /// A fresh, empty folder under the system temp folder.
    fn empty_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn file_contents(dir: &Path) -> String {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| std::fs::read_to_string(entry.unwrap().path()).unwrap())
            .collect()
    }

    #[test]
    fn access_lines_reach_the_daily_file_and_other_events_do_not() {
        let dir = empty_dir("access-log-test");
        let (writer, guard) = access_log_writer(&dir).unwrap();
        let subscriber = tracing_subscriber::registry().with(access_log_layer(writer));

        tracing::subscriber::with_default(subscriber, || {
            write_line(&entry());
            tracing::info!("an ordinary event");
        });
        drop(guard); // flushes the queued lines

        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(names.len(), 1);
        assert!(names[0].starts_with("access.") && names[0].ends_with(".log"));
        let contents = file_contents(&dir);
        assert!(contents.contains("user=user_abc"), "{contents}");
        assert!(contents.contains("path=/api/grocery-items"), "{contents}");
        assert!(contents.contains("ip=192.168.1.20"), "{contents}");
        assert!(!contents.contains("an ordinary event"), "{contents}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_unusable_folder_is_reported_not_fatal() {
        let dir = empty_dir("access-log-test");
        let not_a_folder = dir.join("a-file");
        std::fs::write(&not_a_folder, "").unwrap();

        assert!(access_log_writer(&not_a_folder).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
