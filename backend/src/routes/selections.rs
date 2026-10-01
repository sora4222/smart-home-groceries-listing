//! Choosing one product for each list item.
//!
//! The price comparison saves the household's pick here, and the order screen
//! reads every pick back to know what to buy.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{ItemSelectionChoose, ItemSelectionResponse};
use crate::routes::extract::ValidatedJson;
use crate::services::selections::SelectionService;
use crate::state::AppState;

/// `/api/item-selections` and `/api/grocery-items/{id}/selection`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/item-selections", get(list_selections))
        .route(
            "/api/grocery-items/{item_id}/selection",
            put(choose_product).delete(clear_selection),
        )
}

/// `GET /api/item-selections` — every item's chosen product.
async fn list_selections(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<ItemSelectionResponse>>, ApiError> {
    let selections = SelectionService::new(&state.pool, &state.stores)
        .list()
        .await?;
    Ok(Json(selections.into_iter().map(Into::into).collect()))
}

/// `PUT /api/grocery-items/{id}/selection` — choose the item's product.
///
/// 404 unknown item; 422 the store does not offer that product for the item
/// now; 503 the store could not be searched; 409 the item changed meanwhile
/// or has already been ordered.
async fn choose_product(
    State(state): State<AppState>,
    user: AuthUser,
    Path(item_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ItemSelectionChoose>,
) -> Result<Json<ItemSelectionResponse>, ApiError> {
    let selection = SelectionService::new(&state.pool, &state.stores)
        .choose(item_id, body.store, &body.product_id, &user.id)
        .await?;
    Ok(Json(selection.into()))
}

/// `DELETE /api/grocery-items/{id}/selection` — forget the item's product.
async fn clear_selection(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    SelectionService::new(&state.pool, &state.stores)
        .clear(item_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
