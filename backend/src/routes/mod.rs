//! HTTP routing.
//!
//! Add a domain by writing `routes/<domain>.rs` exposing a `router()`, then
//! merging it in [`api_router`] — this module stays a thin wiring layer, the
//! way `app/main.py` did.

pub mod alexa;
pub mod extract;
pub mod grocery;
pub mod health;
pub mod voice;
pub mod ws;

use axum::Router;

use crate::state::AppState;

/// Every route the backend serves, without middleware.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(voice::router())
        .merge(alexa::router())
        .merge(grocery::router())
        .merge(ws::router())
}
