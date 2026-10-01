//! The web app's side of trolley handoffs: send the chosen products to a
//! store's trolley, and watch what happened.
//!
//! The store's side (the bookmarklet) is in [`super::store_tab`]. See
//! `services::trolley_handoffs` for the whole flow.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    StoreTabSecretResponse, TrolleyHandoffCreate, TrolleyHandoffResponse,
};
use crate::routes::extract::ValidatedJson;
use crate::services::trolley_handoffs::TrolleyHandoffService;
use crate::state::AppState;

/// `/api/trolley-handoffs/*`, all behind a Clerk session.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/trolley-handoffs", post(create_handoff))
        .route(
            "/api/trolley-handoffs/store-tab-secret",
            get(store_tab_secret),
        )
        .route("/api/trolley-handoffs/{handoff_id}", get(get_handoff))
}

/// `POST /api/trolley-handoffs` — hand every chosen product at a store to
/// the store tab, with the delivery time wanted (default: next day, any
/// time). 201 the new handoff; 422 nothing chosen at that store, or a
/// delivery date in the past or too far ahead.
async fn create_handoff(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<TrolleyHandoffCreate>,
) -> Result<(StatusCode, Json<TrolleyHandoffResponse>), ApiError> {
    let created = TrolleyHandoffService::new(&state.pool)
        .create_for_store(body.store, (&body.delivery).into(), &user.id)
        .await?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

/// `GET /api/trolley-handoffs/{id}` — the handoff and each line's outcome.
async fn get_handoff(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(handoff_id): Path<Uuid>,
) -> Result<Json<TrolleyHandoffResponse>, ApiError> {
    let found = TrolleyHandoffService::new(&state.pool)
        .find(handoff_id)
        .await?;
    Ok(Json(found.into()))
}

/// `GET /api/trolley-handoffs/store-tab-secret` — the secret the web app
/// builds the "Fill trolley" bookmarklet with.
///
/// Signed-in household members only. The secret can do nothing but claim and
/// report handoffs. 500 when `STORE_TAB_SECRET` is not set.
async fn store_tab_secret(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<StoreTabSecretResponse>, ApiError> {
    let secret = state.settings.store_tab_secret.clone();
    if secret.is_empty() {
        return Err(ApiError::Misconfigured(
            "STORE_TAB_SECRET is not configured on the server".to_string(),
        ));
    }
    Ok(Json(StoreTabSecretResponse { secret }))
}
