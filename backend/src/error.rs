//! The API's error type and its HTTP representation.
//!
//! Responses keep the `{"detail": ...}` envelope the web app already parses,
//! so the frontend's error handling is unchanged by the move to Rust.
//! Internal failures are logged with their cause but answered with a generic
//! message — a database or upstream error never leaks into a response body.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;

/// A configuration value was present but unusable.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("environment variable {key} is not a valid value")]
    Invalid { key: String },
}

/// Everything a route can fail with.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// No credential was supplied, or the one supplied did not verify.
    #[error("{0}")]
    Unauthorized(String),

    /// The request was well-formed but the resource does not exist.
    #[error("{0}")]
    NotFound(String),

    /// The request conflicts with the resource's current state.
    #[error("{0}")]
    Conflict(String),

    /// The request body failed validation.
    #[error("{0}")]
    UnprocessableEntity(String),

    /// A structured 409 carrying the item that already exists, so the web app
    /// can offer "add another or update the existing quantity?".
    #[error("duplicate active item")]
    DuplicateActiveItem { detail: serde_json::Value },

    /// Something the request depends on — a store — could not be reached just
    /// now. Worth trying again later; the message says what was unavailable.
    #[error("{0}")]
    ServiceUnavailable(String),

    /// A server-side misconfiguration, such as a secret that was never set.
    #[error("{0}")]
    Misconfigured(String),

    /// An unexpected internal failure. The cause is logged, not returned.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.into())
    }
}

/// The body shape the frontend parses: `{"detail": <string | object>}`.
#[derive(Serialize)]
struct ErrorBody {
    detail: serde_json::Value,
}

impl ApiError {
    /// The status code this error is answered with.
    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) | Self::DuplicateActiveItem { .. } => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Misconfigured(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();

        let detail = match &self {
            Self::DuplicateActiveItem { detail } => detail.clone(),
            // Both of these are server-side faults: log the cause, return a
            // generic message so internals are never exposed to a caller.
            Self::Internal(cause) => {
                tracing::error!(error = ?cause, "unhandled internal error");
                json!("Internal server error")
            }
            Self::Misconfigured(message) => {
                tracing::error!(%message, "server is misconfigured");
                json!("Server configuration error")
            }
            other => json!(other.to_string()),
        };

        (status, Json(ErrorBody { detail })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_match_the_python_implementation() {
        assert_eq!(
            ApiError::Unauthorized("x".into()).status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::NotFound("x".into()).status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::Conflict("x".into()).status(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            ApiError::DuplicateActiveItem { detail: json!({}) }.status(),
            StatusCode::CONFLICT
        );
    }

    #[test]
    fn an_unreachable_dependency_is_a_503() {
        assert_eq!(
            ApiError::ServiceUnavailable("Coles could not be searched".into()).status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn internal_errors_do_not_leak_their_cause() {
        let err = ApiError::Internal(anyhow::anyhow!("connection string user=admin"));
        let body = match &err {
            ApiError::Internal(_) => json!("Internal server error"),
            _ => unreachable!(),
        };
        assert_eq!(body, json!("Internal server error"));
        assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
