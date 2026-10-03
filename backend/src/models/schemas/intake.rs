//! Bodies for `/settings/intake` and the Google Tasks channel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::db::GoogleTasksLink;

/// Shortest and longest gap between two polls, matching the CHECK constraint.
pub const MIN_POLL_SECONDS: i32 = 30;
pub const MAX_POLL_SECONDS: i32 = 3600;
/// The gap before anyone chooses one, matching the column default.
pub const DEFAULT_POLL_SECONDS: i32 = 60;

/// What Google sent back to `/settings/intake` after the household signed in.
#[derive(Debug, Deserialize, Validate)]
pub struct GoogleSignInFinish {
    #[validate(length(min = 1, max = 2048))]
    pub code: String,
    #[validate(length(min = 1, max = 256))]
    pub state: String,
}

/// The household's Google Tasks choices.
#[derive(Debug, Deserialize, Validate)]
pub struct GoogleTasksChoices {
    /// The list to watch; `None` clears the choice.
    #[validate(length(min = 1, max = 255))]
    pub task_list_id: Option<String>,
    /// Whether the backend polls the list.
    pub enabled: bool,
    #[validate(range(min = MIN_POLL_SECONDS, max = MAX_POLL_SECONDS))]
    pub poll_seconds: i32,
}

/// A task added to the fake Tasks account (development and e2e only).
#[derive(Debug, Deserialize, Validate)]
pub struct FakeTaskCreate {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
}

/// Where to send the browser to sign in with Google.
#[derive(Debug, Serialize)]
pub struct GoogleSignInStart {
    pub authorize_url: String,
}

/// One Google Tasks list, as the settings page shows it.
#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub id: String,
    pub title: String,
}

/// The Google Tasks channel as `/settings/intake` shows it. Never carries the
/// token itself.
#[derive(Debug, Serialize)]
pub struct GoogleTasksStatus {
    /// `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` are set.
    pub configured: bool,
    /// `CREDENTIAL_ENCRYPTION_KEY` is set, so a sign-in can be stored.
    pub encryption_ready: bool,
    /// A Google sign-in is stored.
    pub connected: bool,
    pub connected_at: Option<DateTime<Utc>>,
    pub task_list: Option<TaskListResponse>,
    pub enabled: bool,
    pub poll_seconds: i32,
    /// True when polling will actually run: on, connected and a list chosen.
    pub polling: bool,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub last_poll_error: Option<String>,
}

/// A channel that only needs a secret in `.env`.
#[derive(Debug, Serialize)]
pub struct SecretChannelStatus {
    /// Its shared secret is set.
    pub configured: bool,
}

/// The triage checker in use, read from the environment.
#[derive(Debug, Serialize)]
pub struct TriageStatusResponse {
    /// `INTAKE_LLM_PROVIDER`, e.g. `ollama`.
    pub provider: String,
    pub model: String,
}

/// Everything `/settings/intake` shows.
#[derive(Debug, Serialize)]
pub struct IntakeSettingsResponse {
    pub google_tasks: GoogleTasksStatus,
    pub alexa: SecretChannelStatus,
    pub webhook: SecretChannelStatus,
    pub triage: TriageStatusResponse,
}

/// What one Google Tasks poll did.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct PollReport {
    /// New items, now in Pending Requests or on their way through triage.
    pub recorded: u32,
    /// Items seen before, whose delete is being tried again.
    pub already_seen: u32,
    /// Blank tasks, left on the list.
    pub blank: u32,
    /// Tasks that could not be deleted from Google; tried again next poll.
    pub not_deleted: u32,
}

impl GoogleTasksStatus {
    /// The status as stored, plus what the server's settings allow.
    pub fn from_link(
        link: Option<&GoogleTasksLink>,
        configured: bool,
        encryption_ready: bool,
    ) -> Self {
        let Some(link) = link else {
            return Self {
                configured,
                encryption_ready,
                connected: false,
                connected_at: None,
                task_list: None,
                enabled: false,
                poll_seconds: DEFAULT_POLL_SECONDS,
                polling: false,
                last_polled_at: None,
                last_poll_error: None,
            };
        };
        Self {
            configured,
            encryption_ready,
            connected: link.refresh_token_encrypted.is_some(),
            connected_at: link.connected_at,
            task_list: match (&link.task_list_id, &link.task_list_title) {
                (Some(id), title) => Some(TaskListResponse {
                    id: id.clone(),
                    title: title.clone().unwrap_or_default(),
                }),
                _ => None,
            },
            enabled: link.enabled,
            poll_seconds: link.poll_seconds,
            polling: link.pollable().is_some(),
            last_polled_at: link.last_polled_at,
            last_poll_error: link.last_poll_error.clone(),
        }
    }
}
