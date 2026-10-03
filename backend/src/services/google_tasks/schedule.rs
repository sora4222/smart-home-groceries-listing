//! The background poll: every `poll_seconds` (read afresh each round, so a
//! change on `/settings/intake` takes effect without a restart), poll once.
//!
//! A failed poll is logged and stored for the settings page; the loop never
//! retries early, so a Google outage costs one call per interval.

use std::time::Duration;

use super::link_repository;
use super::poller::TasksPoller;
use crate::models::schemas::MIN_POLL_SECONDS;
use crate::state::AppState;

/// How long to wait when nothing is stored yet.
const DEFAULT_WAIT: Duration = Duration::from_secs(60);

/// Starts the poll loop. It runs for the life of the process.
pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(next_wait(&state).await).await;
            let poller = TasksPoller::new(
                &state.pool,
                &state.hub,
                &state.triage,
                &state.google_tasks,
                state.encryptor.as_ref(),
            );
            // Failures are already logged and stored by the poller.
            let _ = poller.poll_once().await;
        }
    });
}

/// The household's chosen interval, never under the minimum.
async fn next_wait(state: &AppState) -> Duration {
    match link_repository::get(&state.pool).await {
        Ok(Some(link)) => Duration::from_secs(link.poll_seconds.max(MIN_POLL_SECONDS) as u64),
        Ok(None) => DEFAULT_WAIT,
        Err(err) => {
            tracing::warn!(%err, "could not read google tasks settings; waiting the default");
            DEFAULT_WAIT
        }
    }
}
