//! Authentication abstraction.
//!
//! Every route depends on the [`AuthUser`] extractor or on
//! [`secret::verify_shared_secret`] — never on a specific provider's SDK. To
//! substitute a different provider (Auth0, Supabase Auth, ...), implement
//! [`AuthProvider`] and choose it in [`build_provider`]; nothing else in the
//! application changes.

pub mod clerk;
pub mod secret;
pub mod socket;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::config::Settings;
use crate::error::ApiError;
use crate::state::AppState;

/// A boxed future, so [`AuthProvider`] stays usable behind `dyn`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The authenticated household member making the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
}

/// The identity used when `DEV_AUTH_BYPASS` is on.
pub fn dev_user() -> AuthUser {
    AuthUser {
        id: "dev-user".to_string(),
        email: Some("dev@example.local".to_string()),
    }
}

/// The contract any authentication backend must satisfy.
pub trait AuthProvider: Send + Sync + std::fmt::Debug {
    /// Validates a bearer token, returning the household member it belongs to.
    fn verify_token<'a>(&'a self, token: &'a str) -> BoxFuture<'a, Result<AuthUser, ApiError>>;
}

/// Chooses the provider for this process. The one place a provider is named.
pub fn build_provider(settings: &Settings) -> Arc<dyn AuthProvider> {
    Arc::new(clerk::ClerkAuthProvider::new(
        settings.clerk_jwks_url.clone(),
    ))
}

/// Pulls the bearer token out of an `Authorization` header.
///
/// The scheme is matched case-insensitively because RFC 7235 defines it that
/// way, and some clients send `bearer`.
pub(crate) fn bearer_token(parts: &Parts) -> Option<&str> {
    let raw = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    let token = token.trim();
    (!token.is_empty()).then_some(token)
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    /// Resolves the authenticated household member.
    ///
    /// Every route except the intake endpoints depends on this. With
    /// `DEV_AUTH_BYPASS` set the check is skipped entirely — which is why that
    /// flag must never be set on a host reachable from the internet.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        authenticate(state, bearer_token(parts)).await
    }
}

/// Turns a presented token (or none) into the household member it belongs
/// to. Shared by [`AuthUser`] and [`socket::SocketUser`], so the bypass and
/// the verification are decided in one place.
pub(crate) async fn authenticate(
    state: &AppState,
    token: Option<&str>,
) -> Result<AuthUser, ApiError> {
    if state.settings.dev_auth_bypass {
        return Ok(dev_user());
    }
    let token = token.ok_or_else(|| ApiError::Unauthorized("Missing bearer token".to_string()))?;
    state.auth.verify_token(token).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts_with_auth(value: &str) -> Parts {
        let request = Request::builder()
            .header(AUTHORIZATION, value)
            .body(())
            .unwrap();
        request.into_parts().0
    }

    #[test]
    fn extracts_a_bearer_token() {
        assert_eq!(
            bearer_token(&parts_with_auth("Bearer abc.def")),
            Some("abc.def")
        );
    }

    #[test]
    fn accepts_a_lowercase_scheme() {
        assert_eq!(bearer_token(&parts_with_auth("bearer abc")), Some("abc"));
    }

    #[test]
    fn rejects_other_schemes_and_empty_tokens() {
        assert_eq!(bearer_token(&parts_with_auth("Basic abc")), None);
        assert_eq!(bearer_token(&parts_with_auth("Bearer   ")), None);
        assert_eq!(bearer_token(&parts_with_auth("abc")), None);
    }

    #[test]
    fn rejects_a_missing_header() {
        let parts = Request::builder().body(()).unwrap().into_parts().0;
        assert_eq!(bearer_token(&parts), None);
    }
}
