//! Connecting Google Tasks and choosing what to watch — the rules behind
//! `/settings/intake`.
//!
//! Signing in is a round trip through Google: [`TasksConnection::start_sign_in`]
//! issues a one-use `state` and Google's URL; Google sends the browser back to
//! the settings page with a code; [`TasksConnection::finish_sign_in`] checks the
//! state, trades the code for a refresh token and stores it encrypted.

use sqlx::PgPool;

use super::api::TaskList;
use super::registry::GoogleTasks;
use super::{link_repository, log, oauth_states, sign_in};
use crate::error::ApiError;
use crate::models::db::GoogleTasksLink;
use crate::models::schemas::GoogleTasksChoices;
use crate::services::encryption::Encryptor;

/// The household's link to Google Tasks.
pub struct TasksConnection<'a> {
    pool: &'a PgPool,
    google: &'a GoogleTasks,
    encryptor: Option<&'a Encryptor>,
}

impl<'a> TasksConnection<'a> {
    /// Borrows what one request needs.
    pub fn new(
        pool: &'a PgPool,
        google: &'a GoogleTasks,
        encryptor: Option<&'a Encryptor>,
    ) -> Self {
        Self {
            pool,
            google,
            encryptor,
        }
    }

    /// The stored link, if anything was ever saved.
    pub async fn link(&self) -> Result<Option<GoogleTasksLink>, ApiError> {
        link_repository::get(self.pool).await
    }

    /// Starts a sign-in: Google's URL, carrying a new one-use state.
    ///
    /// Refused while the OAuth client or the encryption key is missing, so
    /// nobody signs in to Google only to find the token cannot be kept.
    pub async fn start_sign_in(&self) -> Result<String, ApiError> {
        if !self.google.configured {
            return Err(ApiError::Misconfigured(
                "GOOGLE_CLIENT_ID or GOOGLE_CLIENT_SECRET is not set".into(),
            ));
        }
        sign_in::require(self.encryptor)?;
        let state = oauth_states::new_state();
        oauth_states::remember(self.pool, &state).await?;
        Ok(self.google.api.authorize_url(&state))
    }

    /// Finishes a sign-in Google sent back. The state must be one this app
    /// issued, in date and unused; otherwise nothing is stored (409).
    pub async fn finish_sign_in(
        &self,
        code: &str,
        state: &str,
        user_id: &str,
    ) -> Result<GoogleTasksLink, ApiError> {
        sign_in::require(self.encryptor)?;
        if !oauth_states::take(self.pool, state).await? {
            log::sign_in_refused(user_id);
            return Err(ApiError::Conflict(
                "That Google sign-in has expired or was already used — try again".into(),
            ));
        }
        let refresh_token = self.google.api.exchange_code(code).await?;
        let sealed = sign_in::seal(self.encryptor, &refresh_token)?;
        let link = link_repository::save_sign_in(self.pool, &sealed).await?;
        log::connected(user_id);
        Ok(link)
    }

    /// Forgets the sign-in and stops polling.
    pub async fn disconnect(&self, user_id: &str) -> Result<(), ApiError> {
        link_repository::clear_sign_in(self.pool).await?;
        log::disconnected(user_id);
        Ok(())
    }

    /// Every list on the connected account.
    pub async fn task_lists(&self) -> Result<Vec<TaskList>, ApiError> {
        let link = self.link().await?.ok_or_else(sign_in::not_connected)?;
        let token = sign_in::open(self.encryptor, &link)?;
        Ok(self.google.api.task_lists(&token).await?)
    }

    /// Saves which list to watch, whether to poll and how often.
    ///
    /// A list must be on the connected account (its title is stored for the
    /// settings page). Polling cannot be switched on without a sign-in and a
    /// list.
    pub async fn save_choices(
        &self,
        choices: &GoogleTasksChoices,
        user_id: &str,
    ) -> Result<GoogleTasksLink, ApiError> {
        let list = match choices.task_list_id.as_deref() {
            Some(id) => Some(self.find_list(id).await?),
            None => None,
        };
        if choices.enabled && list.is_none() {
            return Err(ApiError::Conflict("Pick a Google Tasks list first".into()));
        }
        let link = link_repository::save_choices(
            self.pool,
            list.as_ref().map(|l| (l.id.as_str(), l.title.as_str())),
            choices.enabled,
            choices.poll_seconds,
        )
        .await?;
        log::choices_saved(&link, user_id);
        Ok(link)
    }

    /// The list with this id on the connected account; 404 when it is not
    /// there.
    async fn find_list(&self, id: &str) -> Result<TaskList, ApiError> {
        self.task_lists()
            .await?
            .into_iter()
            .find(|list| list.id == id)
            .ok_or_else(|| ApiError::NotFound(format!("No Google Tasks list {id}")))
    }
}
