//! Settings for the Google Tasks intake channel
//! (`docs/features/FEATURE_GOOGLE_TASKS.md`).
//!
//! The OAuth client id and secret come from the household's own Google Cloud
//! project (`docs/human-setup.md`). Which list to watch, whether polling is on
//! and how often are chosen on `/settings/intake` and kept in the database.

use std::fmt;
use std::time::Duration;

use super::{optional, parse_bool, parse_or};
use crate::error::ConfigError;

/// Which Google Tasks client the backend uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleTasksMode {
    /// Google's real API.
    Live,
    /// An in-memory Tasks account; nothing leaves the machine. For
    /// development, tests and e2e.
    Fake,
}

/// How the backend reaches Google Tasks.
#[derive(Clone)]
pub struct GoogleTasksSettings {
    /// `GOOGLE_TASKS_CLIENT=live|fake`. Defaults to `live`.
    pub mode: GoogleTasksMode,
    /// `GOOGLE_CLIENT_ID` from the Google Cloud project.
    pub client_id: String,
    /// `GOOGLE_CLIENT_SECRET`. Never logged.
    pub client_secret: String,
    /// `GOOGLE_REDIRECT_URI`: the web app's `/settings/intake` page, exactly
    /// as registered on the OAuth client.
    pub redirect_uri: String,
    /// Google's sign-in page. Overridable only for tests.
    pub auth_url: String,
    /// Google's token endpoint. Overridable only for tests.
    pub token_url: String,
    /// The Tasks API root. Overridable only for tests.
    pub api_base: String,
    /// How long one call to Google may take.
    pub timeout: Duration,
    /// Whether this process polls on its own. Off in the integration tests,
    /// which poll by hand so nothing races them.
    pub poll_in_background: bool,
}

impl GoogleTasksSettings {
    /// Reads the Google Tasks settings from the environment.
    pub(super) fn from_env() -> Result<Self, ConfigError> {
        let mode = match optional("GOOGLE_TASKS_CLIENT").map(|v| v.trim().to_ascii_lowercase()) {
            None => GoogleTasksMode::Live,
            Some(value) if value == "live" => GoogleTasksMode::Live,
            Some(value) if value == "fake" => GoogleTasksMode::Fake,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    key: "GOOGLE_TASKS_CLIENT".to_string(),
                })
            }
        };
        Ok(Self {
            mode,
            client_id: optional("GOOGLE_CLIENT_ID").unwrap_or_default(),
            client_secret: optional("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            redirect_uri: optional("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|| "http://localhost:3000/settings/intake".to_string()),
            auth_url: optional("GOOGLE_OAUTH_AUTH_URL")
                .unwrap_or_else(|| "https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            token_url: optional("GOOGLE_OAUTH_TOKEN_URL")
                .unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string()),
            api_base: optional("GOOGLE_TASKS_API_BASE")
                .unwrap_or_else(|| "https://tasks.googleapis.com".to_string()),
            timeout: Duration::from_secs(parse_or("GOOGLE_TIMEOUT_SECONDS", 15)?),
            poll_in_background: !parse_bool("GOOGLE_TASKS_NO_BACKGROUND_POLL"),
        })
    }

    /// Whether the OAuth client is set up. The fake needs no client.
    pub fn is_configured(&self) -> bool {
        self.mode == GoogleTasksMode::Fake
            || (!self.client_id.is_empty() && !self.client_secret.is_empty())
    }
}

/// Hand-written so the client secret can never reach a log line.
impl fmt::Debug for GoogleTasksSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GoogleTasksSettings")
            .field("mode", &self.mode)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[redacted]")
            .field("redirect_uri", &self.redirect_uri)
            .field("api_base", &self.api_base)
            .field("timeout", &self.timeout)
            .field("poll_in_background", &self.poll_in_background)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(mode: GoogleTasksMode, id: &str, secret: &str) -> GoogleTasksSettings {
        GoogleTasksSettings {
            mode,
            client_id: id.to_string(),
            client_secret: secret.to_string(),
            redirect_uri: String::new(),
            auth_url: String::new(),
            token_url: String::new(),
            api_base: String::new(),
            timeout: Duration::from_secs(1),
            poll_in_background: false,
        }
    }

    #[test]
    fn live_needs_both_the_client_id_and_secret() {
        assert!(!settings(GoogleTasksMode::Live, "id", "").is_configured());
        assert!(!settings(GoogleTasksMode::Live, "", "secret").is_configured());
        assert!(settings(GoogleTasksMode::Live, "id", "secret").is_configured());
    }

    #[test]
    fn the_fake_needs_no_client() {
        assert!(settings(GoogleTasksMode::Fake, "", "").is_configured());
    }

    #[test]
    fn debug_never_shows_the_secret() {
        let text = format!("{:?}", settings(GoogleTasksMode::Live, "id", "s3cret"));
        assert!(!text.contains("s3cret"));
    }
}
