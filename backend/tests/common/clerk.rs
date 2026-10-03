//! Clerk stand-in for the auth tests: a mock JWKS endpoint and session tokens
//! signed with the test-only keys in `tests/fixtures/clerk/`.
//!
//! The backend verifies these exactly as it would a real Clerk token, so the
//! tests cover the real verification path without a Clerk account.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use grocery_backend::build_app;
use grocery_backend::config::Settings;
use sqlx::PgPool;

use super::{test_settings, TestApp};

/// The key id `jwks.json` publishes the signing key under.
pub const TEST_KEY_ID: &str = "test-key-1";

const SIGNING_KEY: &[u8] = include_bytes!("../fixtures/clerk/signing_key.pem");
const OTHER_KEY: &[u8] = include_bytes!("../fixtures/clerk/other_key.pem");
const JWKS: &str = include_str!("../fixtures/clerk/jwks.json");

/// Starts a mock Clerk JWKS endpoint and returns its URL.
pub async fn serve_jwks() -> (MockServer, String) {
    let server = MockServer::start().await;
    let jwks: Value = serde_json::from_str(JWKS).expect("jwks.json is JSON");
    Mock::given(method("GET"))
        .and(path("/.well-known/jwks.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
        .mount(&server)
        .await;
    let url = format!("{}/.well-known/jwks.json", server.uri());
    (server, url)
}

/// Seconds since the Unix epoch.
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Claims shaped like a Clerk session token for `user_id`, valid for a minute.
pub fn session_claims(user_id: &str) -> Value {
    json!({ "sub": user_id, "iat": now(), "nbf": now() - 5, "exp": now() + 60 })
}

/// Signs `claims` with the key the mock JWKS publishes.
pub fn sign(claims: &Value) -> String {
    sign_with(claims, SIGNING_KEY, TEST_KEY_ID)
}

/// Signs `claims` with a key the JWKS does not know, under the known key id.
pub fn sign_forged(claims: &Value) -> String {
    sign_with(claims, OTHER_KEY, TEST_KEY_ID)
}

/// Signs `claims` with the right key but a key id the JWKS does not list.
pub fn sign_unknown_key_id(claims: &Value) -> String {
    sign_with(claims, SIGNING_KEY, "not-in-the-jwks")
}

fn sign_with(claims: &Value, pem: &[u8], kid: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    let key = EncodingKey::from_rsa_pem(pem).expect("test key is a valid RSA PEM");
    encode(&header, claims, &key).expect("signing a test token")
}

/// A token signed with HS256 using the JWKS document itself as the secret:
/// the algorithm-confusion attack the verifier must refuse.
pub fn sign_hs256_confusion(claims: &Value) -> String {
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(TEST_KEY_ID.to_string());
    let key = EncodingKey::from_secret(JWKS.as_bytes());
    encode(&header, claims, &key).expect("signing a test token")
}

/// `TestApp` constructors with the real sign-in check switched on.
impl TestApp {
    /// Builds the real application with Clerk verification left switched on,
    /// so a request with no credential is answered the way a browser without a
    /// session would be.
    pub fn with_auth(pool: PgPool) -> Self {
        Self {
            router: build_app(
                pool,
                Settings {
                    dev_auth_bypass: false,
                    ..test_settings()
                },
            ),
        }
    }

    /// Builds the real application verifying session tokens against the
    /// JWKS at `jwks_url` (see [`serve_jwks`]).
    pub fn with_clerk(pool: PgPool, jwks_url: &str) -> Self {
        Self {
            router: build_app(
                pool,
                Settings {
                    dev_auth_bypass: false,
                    clerk_jwks_url: jwks_url.to_string(),
                    ..test_settings()
                },
            ),
        }
    }
}
