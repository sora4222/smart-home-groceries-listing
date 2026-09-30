//! The active grocery list.
//!
//! Read access is needed by the Pending Requests flow (duplicate detection
//! surfaces the existing active item) and by the grocery list view itself.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::GroceryItemResponse;
use crate::services::voice::repository;
use crate::state::AppState;

/// Routes under `/api/grocery-items`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/grocery-items", get(list_active_items))
}

/// `GET /api/grocery-items` — items on the list, newest first.
async fn list_active_items(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<GroceryItemResponse>>, ApiError> {
    let items = repository::list_active_items(&state.pool).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}
