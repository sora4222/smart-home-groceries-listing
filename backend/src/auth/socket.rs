//! Authentication for the `/ws` upgrade.
//!
//! A browser cannot set headers on a WebSocket, so the web app sends its
//! session token as `?token=` instead. Clerk session tokens live about a
//! minute, which keeps a token that lands in an access log short-lived. Only
//! the upgrade accepts a token this way; every other route takes the
//! `Authorization` header alone.

use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use serde::Deserialize;

use crate::auth::{authenticate, bearer_token, AuthUser};
use crate::error::ApiError;
use crate::state::AppState;

/// The household member opening a WebSocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketUser(pub AuthUser);

/// The one query parameter the upgrade reads.
#[derive(Debug, Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

/// The token from `?token=`, if there is a non-empty one.
fn query_token(parts: &Parts) -> Option<String> {
    let Query(query) = Query::<TokenQuery>::try_from_uri(&parts.uri).ok()?;
    query.token.filter(|token| !token.trim().is_empty())
}

impl FromRequestParts<AppState> for SocketUser {
    type Rejection = ApiError;

    /// Takes the `Authorization` header when there is one (non-browser
    /// clients), and `?token=` otherwise.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let from_query = query_token(parts);
        let token = bearer_token(parts).or(from_query.as_deref());
        authenticate(state, token).await.map(SocketUser)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts(uri: &str) -> Parts {
        Request::builder().uri(uri).body(()).unwrap().into_parts().0
    }

    #[test]
    fn reads_the_token_query_parameter() {
        assert_eq!(
            query_token(&parts("/ws?token=abc.def")),
            Some("abc.def".into())
        );
    }

    #[test]
    fn ignores_a_missing_or_empty_token() {
        assert_eq!(query_token(&parts("/ws")), None);
        assert_eq!(query_token(&parts("/ws?token=")), None);
        assert_eq!(query_token(&parts("/ws?other=1")), None);
    }
}
