//! HTTP routing.
//!
//! Add a domain by writing `routes/<domain>.rs` exposing a `router()`, then
//! merging it in [`api_router`] — this module stays a thin wiring layer, the
//! way `app/main.py` did.

pub mod alexa;
pub mod dislikes;
pub mod extract;
pub mod grocery;
pub mod health;
pub mod item_rules;
pub mod order_review;
pub mod products;
pub mod purchases;
pub mod selections;
pub mod spending;
pub mod store_tab;
pub mod triage;
pub mod trolley_handoffs;
pub mod voice;
pub mod ws;

use axum::Router;

use crate::state::AppState;

/// Every route the backend serves, without the app-wide middleware.
///
/// Takes the state because the store-tab routes carry a layer of their own
/// that reads the stores' configured websites.
pub fn api_router(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(voice::router())
        .merge(alexa::router())
        .merge(dislikes::router())
        .merge(grocery::router())
        .merge(item_rules::router())
        .merge(order_review::router())
        .merge(products::router())
        .merge(purchases::router())
        .merge(selections::router())
        .merge(spending::router())
        .merge(triage::router())
        .merge(trolley_handoffs::router())
        .merge(store_tab::router(state.clone()))
        .merge(ws::router())
}
