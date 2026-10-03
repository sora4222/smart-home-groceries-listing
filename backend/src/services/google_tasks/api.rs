//! The contract every Google Tasks client satisfies.
//!
//! The rest of the backend sees only [`TasksApi`] — never a Google URL, token
//! or JSON body — so the real API and the in-memory fake are interchangeable
//! (`GOOGLE_TASKS_CLIENT`).

use crate::services::stores::client::BoxFuture;

/// One Google Tasks list, e.g. "Groceries".
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TaskList {
    pub id: String,
    pub title: String,
}

/// One open task on a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTask {
    /// Google's id for the task. Becomes the request's `external_id`.
    pub id: String,
    /// What the person typed or said, e.g. "2 oat milk".
    pub title: String,
}

/// Why a call to Google did not work. Shown to the household, so the message
/// names what went wrong without any token, header or response body.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GoogleError {
    /// Google refused the sign-in: it was revoked, expired, or the code was
    /// already used. The household has to connect again.
    #[error("Google said the sign-in is no longer valid — connect Google Tasks again")]
    SignInRevoked,
    /// The list or task does not exist any more.
    #[error("Google could not find that list or task")]
    NotFound,
    /// Google could not be reached, refused, or took too long.
    #[error("Google Tasks could not be reached ({0})")]
    Unavailable(String),
    /// Google answered, but not in the shape expected.
    #[error("Google Tasks gave an answer that could not be read")]
    BadReply,
}

/// Talks to one household's Google Tasks account. Every call that touches
/// the account takes the stored refresh token; the client turns it into a
/// short-lived access token itself.
pub trait TasksApi: Send + Sync {
    /// Google's sign-in page, carrying `state` so the reply can be checked.
    fn authorize_url(&self, state: &str) -> String;

    /// Trades the code Google sent back for a long-lived refresh token.
    fn exchange_code<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<String, GoogleError>>;

    /// Every Tasks list on the account.
    fn task_lists<'a>(
        &'a self,
        refresh_token: &'a str,
    ) -> BoxFuture<'a, Result<Vec<TaskList>, GoogleError>>;

    /// The open (not completed) tasks on one list, oldest first.
    fn open_tasks<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
    ) -> BoxFuture<'a, Result<Vec<RemoteTask>, GoogleError>>;

    /// Deletes one task. A task that is already gone counts as deleted.
    fn delete_task<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
        task_id: &'a str,
    ) -> BoxFuture<'a, Result<(), GoogleError>>;

    /// Adds a task to a list and returns it, with its new id.
    fn insert_task<'a>(
        &'a self,
        refresh_token: &'a str,
        list_id: &'a str,
        title: &'a str,
    ) -> BoxFuture<'a, Result<RemoteTask, GoogleError>>;
}

/// How a Google failure is answered over HTTP: a sign-in to redo is a 409 the
/// household can act on, a missing list a 404, anything else a 503 — Google
/// was unavailable, not this server broken.
impl From<GoogleError> for crate::error::ApiError {
    fn from(err: GoogleError) -> Self {
        match err {
            GoogleError::SignInRevoked => Self::Conflict(err.to_string()),
            GoogleError::NotFound => Self::NotFound(err.to_string()),
            GoogleError::Unavailable(_) | GoogleError::BadReply => {
                Self::ServiceUnavailable(err.to_string())
            }
        }
    }
}
