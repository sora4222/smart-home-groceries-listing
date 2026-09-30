//! The household grocery list: review, modify, annotate, commit.
//!
//! Every route here needs a signed-in household member. Items arrive either
//! from `POST /api/grocery-items` (the web app's add-item form) or by
//! accepting an intake request in `routes/voice.rs`; from then on they are
//! edited and committed through this module.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{GroceryItemCreate, GroceryItemResponse, GroceryItemUpdate};
use crate::routes::extract::ValidatedJson;
use crate::services::grocery::GroceryService;
use crate::state::AppState;

/// Routes under `/api/grocery-items`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/grocery-items", get(list_items).post(add_item))
        .route("/api/grocery-items/commit", post(commit_list))
        .route("/api/grocery-items/release", post(release_list))
        .route(
            "/api/grocery-items/{item_id}",
            patch(update_item).delete(delete_item),
        )
}

/// `?merge=true` folds the quantity into the existing active item instead of
/// answering 409.
#[derive(Debug, Deserialize)]
pub struct AddQuery {
    #[serde(default)]
    pub merge: bool,
}

/// `GET /api/grocery-items` — the list, newest first.
///
/// Carries both items under review and items already committed for purchase;
/// the web app groups them by `status`.
async fn list_items(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<GroceryItemResponse>>, ApiError> {
    let items = GroceryService::new(&state.pool).list().await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

/// `POST /api/grocery-items` — add an item by hand.
///
/// 409 with the clashing item when the name is already on the active list,
/// unless `?merge=true` asks for the quantities to be combined.
async fn add_item(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<AddQuery>,
    ValidatedJson(body): ValidatedJson<GroceryItemCreate>,
) -> Result<(StatusCode, Json<GroceryItemResponse>), ApiError> {
    let item = GroceryService::new(&state.pool)
        .add(&body, &user.id, query.merge)
        .await?;
    Ok((StatusCode::CREATED, Json(item.into())))
}

/// `PATCH /api/grocery-items/{id}` — rename, re-quantify, annotate, re-chip.
///
/// Absent fields are left alone. 409 if the item is committed for purchase.
async fn update_item(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<GroceryItemUpdate>,
) -> Result<Json<GroceryItemResponse>, ApiError> {
    let item = GroceryService::new(&state.pool)
        .update(item_id, &body)
        .await?;
    Ok(Json(item.into()))
}

/// `DELETE /api/grocery-items/{id}` — take an item off the list.
async fn delete_item(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    GroceryService::new(&state.pool).delete(item_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/grocery-items/commit` — lock the reviewed list in for purchase.
async fn commit_list(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<GroceryItemResponse>>, ApiError> {
    let items = GroceryService::new(&state.pool).commit_list().await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

/// `POST /api/grocery-items/release` — reopen a committed list for editing.
async fn release_list(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<GroceryItemResponse>>, ApiError> {
    let items = GroceryService::new(&state.pool).release_list().await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}
