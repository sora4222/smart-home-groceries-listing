//! Grocery list backend: voice intake, the confirmation queue, the list, and
//! store product search (ordering to come).
//!
//! [`build_app`] assembles the router and middleware; `main.rs` only reads
//! configuration, opens the pool and serves it. Keeping assembly in the
//! library is what lets the integration tests exercise the real application
//! rather than a stand-in.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::Router;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::config::Settings;
use crate::services::encryption::Encryptor;
use crate::services::ws_hub::WsHub;
use crate::state::AppState;

/// Largest body any endpoint accepts. Intake payloads are a few hundred bytes;
/// a cap this low turns a malicious or looping client into a cheap 413 instead
/// of memory pressure.
const MAX_BODY_BYTES: usize = 16 * 1024;

/// How long a request may run before it is abandoned.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Builds the application: state, routes and middleware.
pub fn build_app(pool: PgPool, settings: Settings) -> Router {
    let encryptor = match Encryptor::from_base64_key(&settings.credential_encryption_key) {
        Ok(encryptor) => Some(encryptor),
        Err(err) => {
            // Store credentials are not needed for voice intake, so this is a
            // warning rather than a refusal to start.
            tracing::warn!(%err, "credential encryption unavailable");
            None
        }
    };

    if settings.dev_auth_bypass {
        tracing::warn!(
            "DEV_AUTH_BYPASS is enabled — every API request is treated as signed in. \
             Never run this on a host reachable from the internet."
        );
    }

    let state = AppState {
        pool,
        auth: auth::build_provider(&settings),
        stores: services::stores::registry::build(&settings.stores),
        settings: Arc::new(settings),
        hub: WsHub::new(),
        encryptor,
    };

    let cors = cors_layer(&state.settings);

    routes::api_router(&state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    REQUEST_TIMEOUT,
                ))
                .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
                .layer(cors),
        )
        .with_state(state)
}

/// CORS restricted to the configured origins.
///
/// Never `Any`: credentials are allowed, and a wildcard origin with
/// credentials would let any site on the internet drive the household's API
/// using the user's own session. Every method a route serves must be listed,
/// or browsers block it while `curl` and the tests still work
/// (`tests/cors.rs` checks).
fn cors_layer(settings: &Settings) -> CorsLayer {
    let origins: Vec<HeaderValue> = settings
        .cors_origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::warn!(%origin, "ignoring unparseable CORS origin");
                None
            }
        })
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}
