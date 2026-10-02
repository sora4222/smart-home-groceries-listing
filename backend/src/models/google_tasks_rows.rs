//! Row types for the Google Tasks intake channel (`migrations/0010_*`).

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// The household's one link to Google Tasks: the stored sign-in, the list
/// that is the grocery inbox, and how polling is set.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GoogleTasksLink {
    pub id: Uuid,
    /// The refresh token, AES-256-GCM encrypted. `None` when not connected.
    pub refresh_token_encrypted: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    /// The Tasks list polled for items.
    pub task_list_id: Option<String>,
    pub task_list_title: Option<String>,
    /// Polling runs only when this is on, a sign-in is stored and a list is
    /// chosen.
    pub enabled: bool,
    pub poll_seconds: i32,
    pub last_polled_at: Option<DateTime<Utc>>,
    /// Why the last poll failed, for a person. `None` after a good poll.
    pub last_poll_error: Option<String>,
}

impl GoogleTasksLink {
    /// The stored sign-in and chosen list, when polling should run.
    pub fn pollable(&self) -> Option<(&str, &str)> {
        if !self.enabled {
            return None;
        }
        Some((
            self.refresh_token_encrypted.as_deref()?,
            self.task_list_id.as_deref()?,
        ))
    }
}
