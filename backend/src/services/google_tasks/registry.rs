//! Choosing the Google Tasks client for this process. The one place that
//! names a concrete [`TasksApi`].

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use super::api::TasksApi;
use super::fake::FakeTasksApi;
use super::live::LiveTasksApi;
use crate::config::{GoogleTasksMode, GoogleTasksSettings};

/// Everything the Google Tasks channel shares across requests. Cheap to
/// clone.
#[derive(Clone)]
pub struct GoogleTasks {
    /// The client `GOOGLE_TASKS_CLIENT` chose.
    pub api: Arc<dyn TasksApi>,
    /// The same client when it is the fake, so the development-only route
    /// can add tasks to it. `None` against the real Google.
    pub fake: Option<Arc<FakeTasksApi>>,
    /// Whether an OAuth client id and secret are set.
    pub configured: bool,
    /// Held for the whole of a poll, so a "Check now" and the background
    /// poll never record or delete the same task twice.
    pub poll_lock: Arc<Mutex<()>>,
}

/// Builds the client `settings` asks for.
pub fn build(settings: &GoogleTasksSettings) -> GoogleTasks {
    let (api, fake): (Arc<dyn TasksApi>, Option<Arc<FakeTasksApi>>) = match settings.mode {
        GoogleTasksMode::Fake => {
            let fake = Arc::new(FakeTasksApi::new(&settings.redirect_uri));
            (fake.clone(), Some(fake))
        }
        GoogleTasksMode::Live => (
            Arc::new(LiveTasksApi::new(http(settings.timeout), settings)),
            None,
        ),
    };
    tracing::info!(mode = ?settings.mode, configured = settings.is_configured(), "google tasks ready");
    GoogleTasks {
        api,
        fake,
        configured: settings.is_configured(),
        poll_lock: Arc::new(Mutex::new(())),
    }
}

/// An HTTP client with a time limit on every call to Google.
fn http(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|err| {
            tracing::error!(%err, "google tasks HTTP client could not be built; using defaults");
            reqwest::Client::new()
        })
}
