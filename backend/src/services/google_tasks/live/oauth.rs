//! Google's OAuth 2.0 endpoints: the sign-in URL, trading a code for a
//! refresh token, and trading the refresh token for short-lived access
//! tokens, which are reused until shortly before they expire.
//!
//! Scope is `tasks` only. `access_type=offline` and `prompt=consent` make
//! Google return a refresh token every time someone connects.

use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::Mutex;

use super::status_error;
use crate::config::GoogleTasksSettings;
use crate::services::google_tasks::api::GoogleError;

/// The one permission asked for: read, add and delete tasks.
pub const TASKS_SCOPE: &str = "https://www.googleapis.com/auth/tasks";

/// An access token is renewed this long before Google says it expires.
const EXPIRY_MARGIN: Duration = Duration::from_secs(60);

/// What the token endpoint answers.
#[derive(Debug, Deserialize)]
struct TokenReply {
    access_token: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

/// A reusable access token, for the refresh token it came from.
struct CachedToken {
    refresh_token: String,
    access_token: String,
    expires_at: Instant,
}

/// The OAuth half of the live client.
pub struct OAuth {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    auth_url: String,
    token_url: String,
    cache: Mutex<Option<CachedToken>>,
}

impl OAuth {
    /// An OAuth client for the household's Google Cloud project.
    pub fn new(http: reqwest::Client, settings: &GoogleTasksSettings) -> Self {
        Self {
            http,
            client_id: settings.client_id.clone(),
            client_secret: settings.client_secret.clone(),
            redirect_uri: settings.redirect_uri.clone(),
            auth_url: settings.auth_url.clone(),
            token_url: settings.token_url.clone(),
            cache: Mutex::new(None),
        }
    }

    /// Google's sign-in page for this client, carrying `state`.
    pub fn authorize_url(&self, state: &str) -> String {
        authorize_url(&self.auth_url, &self.client_id, &self.redirect_uri, state)
    }

    /// Trades a sign-in code for a refresh token.
    pub async fn exchange_code(&self, code: &str) -> Result<String, GoogleError> {
        let reply = self
            .token_request(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
            ])
            .await?;
        reply.refresh_token.ok_or(GoogleError::BadReply)
    }

    /// A current access token for `refresh_token`, from the cache when it is
    /// still good.
    pub async fn access_token(&self, refresh_token: &str) -> Result<String, GoogleError> {
        let mut cache = self.cache.lock().await;
        if let Some(cached) = cache.as_ref() {
            if cached.refresh_token == refresh_token && cached.expires_at > Instant::now() {
                return Ok(cached.access_token.clone());
            }
        }
        let reply = self
            .token_request(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .await?;
        let lifetime = Duration::from_secs(reply.expires_in.unwrap_or(3600));
        *cache = Some(CachedToken {
            refresh_token: refresh_token.to_string(),
            access_token: reply.access_token.clone(),
            expires_at: Instant::now() + lifetime.saturating_sub(EXPIRY_MARGIN),
        });
        Ok(reply.access_token)
    }

    /// Forgets the cached access token, after Google refused it.
    pub async fn forget(&self) {
        *self.cache.lock().await = None;
    }

    /// One call to the token endpoint. A 400 or 401 there means the code or
    /// refresh token is no good (`invalid_grant`): the household must
    /// connect again.
    async fn token_request(&self, fields: &[(&str, &str)]) -> Result<TokenReply, GoogleError> {
        let mut form: Vec<(&str, &str)> = vec![
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];
        form.extend_from_slice(fields);
        let response = self
            .http
            .post(&self.token_url)
            .form(&form)
            .send()
            .await
            .map_err(super::unreachable)?;
        let status = response.status();
        if status.as_u16() == 400 || status.as_u16() == 401 {
            return Err(GoogleError::SignInRevoked);
        }
        if !status.is_success() {
            return Err(status_error(status));
        }
        response
            .json::<TokenReply>()
            .await
            .map_err(|_| GoogleError::BadReply)
    }
}

/// Builds Google's sign-in URL (pure, so it is unit-tested).
pub fn authorize_url(base: &str, client_id: &str, redirect_uri: &str, state: &str) -> String {
    let mut url = match url::Url::parse(base) {
        Ok(url) => url,
        Err(_) => return base.to_string(),
    };
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", TASKS_SCOPE)
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", state);
    url.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sign_in_url_asks_for_offline_tasks_access_with_the_state() {
        let url = authorize_url(
            "https://accounts.google.com/o/oauth2/v2/auth",
            "client-1",
            "http://localhost:3000/settings/intake",
            "abc",
        );
        let parsed = url::Url::parse(&url).unwrap();
        let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(pairs["client_id"], "client-1");
        assert_eq!(
            pairs["redirect_uri"],
            "http://localhost:3000/settings/intake"
        );
        assert_eq!(pairs["scope"], TASKS_SCOPE);
        assert_eq!(pairs["access_type"], "offline");
        assert_eq!(pairs["prompt"], "consent");
        assert_eq!(pairs["state"], "abc");
        assert_eq!(pairs["response_type"], "code");
    }
}
