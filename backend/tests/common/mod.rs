//! Shared harness for the integration tests.
//!
//! Each test gets its own database: `#[sqlx::test]` creates one, runs the
//! migrations in `migrations/`, hands over a pool and drops the database
//! afterwards. That replaces the previous testcontainers fixture and needs no
//! Docker daemon — only a PostgreSQL server at `DATABASE_URL` that may create
//! databases.
//!
//! Requests go through the real router, middleware included, so a test
//! exercises the same stack a browser would.

#![allow(dead_code)]

pub mod purchases;
pub mod triage;
pub mod trolley;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use grocery_backend::build_app;
use grocery_backend::config::{Settings, StoreMode, StoreSettings, TriageProvider, TriageSettings};
use grocery_backend::routes::alexa::BRIDGE_SECRET_HEADER;
use grocery_backend::routes::voice::WEBHOOK_SECRET_HEADER;

/// The shared secrets the tests authenticate with. Test-only values.
pub const TEST_WEBHOOK_SECRET: &str = "test-webhook-secret";
pub const TEST_BRIDGE_SECRET: &str = "test-bridge-secret";
pub const TEST_STORE_TAB_SECRET: &str = "test-store-tab-secret";

/// The application under test.
pub struct TestApp {
    router: Router,
}

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

    /// Builds the real application with triage switched on, as `triage` says.
    pub fn with_triage(pool: PgPool, triage: TriageSettings) -> Self {
        Self {
            router: build_app(
                pool,
                Settings {
                    triage,
                    ..test_settings()
                },
            ),
        }
    }

    /// Builds the real application over a test database.
    ///
    /// `dev_auth_bypass` is on, matching how the previous Python suite ran:
    /// these tests cover the confirmation queue's rules, not Clerk's JWKS
    /// verification, which is unit-tested in `auth::clerk`.
    pub fn new(pool: PgPool) -> Self {
        Self {
            router: build_app(pool, test_settings()),
        }
    }

    /// Serves the application on an ephemeral port, returning its address.
    ///
    /// Used by the WebSocket tests, which need a real socket rather than the
    /// in-process `oneshot` the HTTP tests use. The server task is detached;
    /// the test process exiting is what stops it.
    pub async fn serve(&self) -> std::net::SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binding an ephemeral port");
        let address = listener.local_addr().expect("reading the bound address");
        let router = self.router.clone();
        tokio::spawn(async move {
            let _ = axum::serve(listener, router.into_make_service()).await;
        });
        address
    }

    /// Sends a request and returns the status and parsed JSON body.
    pub async fn send(&self, request: Request<Body>) -> (StatusCode, Value) {
        let (status, body, _) = self.send_with_headers(request).await;
        (status, body)
    }

    /// Sends a request and returns the status, parsed JSON body and headers.
    pub async fn send_with_headers(
        &self,
        request: Request<Body>,
    ) -> (StatusCode, Value, axum::http::HeaderMap) {
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router should not fail to respond");
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("response body should be readable")
            .to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        (status, body, headers)
    }

    /// `GET <uri>` as a signed-in household member.
    pub async fn get(&self, uri: &str) -> (StatusCode, Value) {
        self.send(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    /// `POST <uri>` with no body, as a signed-in household member.
    pub async fn post(&self, uri: &str) -> (StatusCode, Value) {
        self.send(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    /// `POST <uri>` with a JSON body, as a signed-in household member.
    pub async fn post_json(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        self.send(json_request(uri, body, &[])).await
    }

    /// `PATCH <uri>` with a JSON body, as a signed-in household member.
    pub async fn patch_json(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        self.send(
            Request::builder()
                .method(Method::PATCH)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
    }

    /// `PUT <uri>` with a JSON body, as a signed-in household member.
    pub async fn put_json(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        self.send(
            Request::builder()
                .method(Method::PUT)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
    }

    /// `DELETE <uri>` as a signed-in household member.
    pub async fn delete(&self, uri: &str) -> (StatusCode, Value) {
        self.send(
            Request::builder()
                .method(Method::DELETE)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    /// Adds an item through the web app's own endpoint and returns its id.
    pub async fn add_item(&self, body: &Value) -> String {
        let (status, created) = self.post_json("/api/grocery-items", body).await;
        assert_eq!(status, StatusCode::CREATED, "setup failed: {created}");
        created["id"]
            .as_str()
            .expect("id should be a string")
            .to_string()
    }

    /// `POST /api/voice-requests` carrying the webhook secret.
    pub async fn post_webhook(&self, body: &Value) -> (StatusCode, Value) {
        self.send(json_request(
            "/api/voice-requests",
            body,
            &[(WEBHOOK_SECRET_HEADER, TEST_WEBHOOK_SECRET)],
        ))
        .await
    }

    /// `POST /api/voice-requests` with no webhook secret.
    pub async fn post_webhook_unauthenticated(&self, body: &Value) -> (StatusCode, Value) {
        self.send(json_request("/api/voice-requests", body, &[]))
            .await
    }

    /// `POST /api/intake/alexa` carrying the bridge secret.
    pub async fn post_alexa(&self, body: &Value) -> (StatusCode, Value) {
        self.send(json_request(
            "/api/intake/alexa",
            body,
            &[(BRIDGE_SECRET_HEADER, TEST_BRIDGE_SECRET)],
        ))
        .await
    }

    /// `POST /api/intake/alexa` with an arbitrary bridge-secret header.
    pub async fn post_alexa_with_secret(
        &self,
        body: &Value,
        secret: Option<&str>,
    ) -> (StatusCode, Value) {
        let headers: Vec<(&str, &str)> = secret
            .map(|s| vec![(BRIDGE_SECRET_HEADER, s)])
            .unwrap_or_default();
        self.send(json_request("/api/intake/alexa", body, &headers))
            .await
    }

    /// Creates a pending webhook request and returns its id.
    pub async fn create_pending(&self, item: &str, quantity: i64) -> String {
        let (status, body) = self
            .post_webhook(&serde_json::json!({ "item": item, "quantity": quantity }))
            .await;
        assert_eq!(status, StatusCode::CREATED, "setup failed: {body}");
        body["id"]
            .as_str()
            .expect("id should be a string")
            .to_string()
    }
}

/// Builds a JSON `POST` request with optional extra headers.
fn json_request(uri: &str, body: &Value, headers: &[(&str, &str)]) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

/// Settings for a test run: auth bypassed, known secrets, fake stores, no
/// external calls.
fn test_settings() -> Settings {
    Settings {
        // The pool is supplied directly, so this is never dialled.
        database_url: String::new(),
        database_max_connections: 5,
        bind_address: "127.0.0.1:0".to_string(),
        clerk_jwks_url: String::new(),
        voice_webhook_secret: TEST_WEBHOOK_SECRET.to_string(),
        alexa_bridge_secret: TEST_BRIDGE_SECRET.to_string(),
        store_tab_secret: TEST_STORE_TAB_SECRET.to_string(),
        credential_encryption_key: String::new(),
        cors_origins: vec!["http://localhost:3000".to_string()],
        dev_auth_bypass: true,
        stores: fake_store_settings(),
        triage: triage_settings(TriageProvider::Off, ""),
    }
}

/// Triage settings for a test. Off by default, so a request is in Pending
/// Requests the moment it is recorded; the triage tests switch it on.
pub fn triage_settings(provider: TriageProvider, base_url: &str) -> TriageSettings {
    TriageSettings {
        provider,
        base_url: base_url.to_string(),
        model: "test-model".to_string(),
        api_key: "test-key".to_string(),
        timeout: std::time::Duration::from_secs(2),
        min_confidence: 0.7,
    }
}

/// The fake catalogue: an integration test must never reach a real store.
pub fn fake_store_settings() -> StoreSettings {
    StoreSettings {
        mode: StoreMode::Fake,
        woolworths_base_url: "http://127.0.0.1:9".to_string(),
        coles_base_url: "http://127.0.0.1:9".to_string(),
        timeout: std::time::Duration::from_secs(2),
        cache_ttl: std::time::Duration::from_secs(60),
    }
}
