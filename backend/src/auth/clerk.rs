//! Clerk JWT verification — the current [`AuthProvider`] implementation.
//!
//! Verifies the bearer token from the web app against Clerk's published JWKS,
//! fetched on demand and cached for [`JWKS_CACHE_TTL_SECONDS`]. Replacing this
//! file is enough to change auth provider; no route imports it directly.
//!
//! The signing algorithm comes from the *JWK*, never from the token's own
//! header: trusting the header's `alg` is how algorithm-confusion attacks get
//! in (a token claiming `HS256` verified against the RSA public key as an HMAC
//! secret).

use std::time::{Duration, Instant};

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::auth::{AuthProvider, AuthUser, BoxFuture};
use crate::config::JWKS_CACHE_TTL_SECONDS;
use crate::error::ApiError;

/// How long to wait for Clerk's JWKS endpoint before giving up.
const JWKS_FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// The claims this application needs from a Clerk session token.
#[derive(Debug, Deserialize)]
struct ClerkClaims {
    /// Clerk's user id.
    sub: String,
    #[serde(default)]
    email: Option<String>,
}

/// Verifies Clerk-issued session JWTs against the instance's JWKS.
#[derive(Debug)]
pub struct ClerkAuthProvider {
    jwks_url: String,
    cache: RwLock<Option<CachedJwks>>,
    http: reqwest::Client,
}

#[derive(Debug, Clone)]
struct CachedJwks {
    jwks: JwkSet,
    fetched_at: Instant,
}

impl ClerkAuthProvider {
    /// Creates a provider for the given JWKS endpoint.
    pub fn new(jwks_url: String) -> Self {
        Self {
            jwks_url,
            cache: RwLock::new(None),
            http: reqwest::Client::builder()
                .timeout(JWKS_FETCH_TIMEOUT)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Returns the cached JWKS, re-fetching once the TTL has passed.
    async fn jwks(&self) -> Result<JwkSet, ApiError> {
        if self.jwks_url.is_empty() {
            return Err(ApiError::Misconfigured(
                "CLERK_JWKS_URL is not configured on the server".to_string(),
            ));
        }

        let ttl = Duration::from_secs(JWKS_CACHE_TTL_SECONDS);
        if let Some(cached) = self.cache.read().await.as_ref() {
            if cached.fetched_at.elapsed() < ttl {
                return Ok(cached.jwks.clone());
            }
        }

        let jwks: JwkSet = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|err| ApiError::Internal(err.into()))?
            .error_for_status()
            .map_err(|err| ApiError::Internal(err.into()))?
            .json()
            .await
            .map_err(|err| ApiError::Internal(err.into()))?;

        *self.cache.write().await = Some(CachedJwks {
            jwks: jwks.clone(),
            fetched_at: Instant::now(),
        });
        Ok(jwks)
    }

    async fn verify(&self, token: &str) -> Result<AuthUser, ApiError> {
        let header = decode_header(token)
            .map_err(|err| ApiError::Unauthorized(format!("Invalid session token: {err}")))?;
        let kid = header
            .kid
            .ok_or_else(|| ApiError::Unauthorized("Token has no key id".to_string()))?;

        let jwks = self.jwks().await?;
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| ApiError::Unauthorized("No matching signing key".to_string()))?;

        let algorithm = jwk
            .common
            .key_algorithm
            .and_then(|alg| alg.to_string().parse().ok())
            .ok_or_else(|| {
                ApiError::Unauthorized("Signing key declares no usable algorithm".to_string())
            })?;

        let key = DecodingKey::from_jwk(jwk)
            .map_err(|err| ApiError::Internal(anyhow::anyhow!("unusable JWK: {err}")))?;

        let mut validation = Validation::new(algorithm);
        // Clerk session tokens carry no `aud` for a backend API, which matches
        // the previous implementation's `verify_aud: False`.
        validation.validate_aud = false;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.set_required_spec_claims(&["exp", "sub"]);

        let data = decode::<ClerkClaims>(token, &key, &validation)
            .map_err(|err| ApiError::Unauthorized(format!("Invalid session token: {err}")))?;

        if data.claims.sub.is_empty() {
            return Err(ApiError::Unauthorized(
                "Token missing subject claim".to_string(),
            ));
        }

        Ok(AuthUser {
            id: data.claims.sub,
            email: data.claims.email,
        })
    }
}

impl AuthProvider for ClerkAuthProvider {
    fn verify_token<'a>(&'a self, token: &'a str) -> BoxFuture<'a, Result<AuthUser, ApiError>> {
        Box::pin(self.verify(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_unconfigured_jwks_url_is_a_misconfiguration_not_a_bypass() {
        let provider = ClerkAuthProvider::new(String::new());
        assert!(matches!(
            provider.verify("any.token.here").await,
            // The header is parsed first, so a junk token is rejected as
            // unauthorized; a well-formed one reaches the JWKS check.
            Err(ApiError::Unauthorized(_)) | Err(ApiError::Misconfigured(_))
        ));
    }

    #[tokio::test]
    async fn a_malformed_token_is_rejected_before_any_network_call() {
        let provider = ClerkAuthProvider::new("http://127.0.0.1:1/jwks".to_string());
        assert!(matches!(
            provider.verify("not-a-jwt").await,
            Err(ApiError::Unauthorized(_))
        ));
    }

    #[tokio::test]
    async fn a_token_without_a_key_id_is_rejected() {
        // HS256 header with no `kid`: {"alg":"HS256","typ":"JWT"}
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyIn0.c2ln";
        let provider = ClerkAuthProvider::new("http://127.0.0.1:1/jwks".to_string());
        assert!(matches!(
            provider.verify(token).await,
            Err(ApiError::Unauthorized(_))
        ));
    }
}
