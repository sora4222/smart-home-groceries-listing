//! Liveness endpoint, used by Docker Compose and the Cloudflare Tunnel.

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// `GET /api/health`
pub fn router() -> Router<AppState> {
    Router::new().route("/api/health", get(health))
}

/// Reports that the process is up. Deliberately does not touch the database:
/// this answers "is the container alive", not "is every dependency healthy".
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
