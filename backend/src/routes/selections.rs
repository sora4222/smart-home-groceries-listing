//! Choosing products for each list item.
//!
//! The Select item page saves the household's picks here, and the order
//! screen reads every pick back to know what to buy.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    ItemChoicesResponse, ItemSelectionChoose, ItemSelectionClear, ItemSelectionQuantity,
    ItemSelectionResponse, OrderStoresSet, SelectionRestoreQuery, SelectionSnapshotResponse,
};
use crate::routes::extract::{ValidatedJson, ValidatedQuery};
use crate::services::selections::SelectionService;
use crate::state::AppState;

/// `/api/item-selections`, `/api/grocery-items/{id}/selection`,
/// `/api/selection-snapshots` and `/api/order-stores`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/item-selections", get(list_selections))
        .route("/api/order-stores", put(set_order_stores))
        .route(
            "/api/grocery-items/{item_id}/selection",
            put(choose_product).delete(clear_selection),
        )
        .route(
            "/api/grocery-items/{item_id}/selection/quantity",
            put(set_quantity),
        )
        .route(
            "/api/grocery-items/{item_id}/selection/snapshot",
            post(save_snapshot),
        )
        .route(
            "/api/selection-snapshots/{snapshot_id}/restore",
            post(restore_snapshot),
        )
}

/// `GET /api/item-selections` — every item's product to buy, with its
/// choices at the other stores.
async fn list_selections(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<ItemChoicesResponse>>, ApiError> {
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

/// `DELETE /api/grocery-items/{id}/selection[?store=[&product_id=]]` —
/// forget one product, every product at a store, or every choice.
async fn clear_selection(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<ItemSelectionClear>,
) -> Result<StatusCode, ApiError> {
    SelectionService::new(&state.pool, &state.stores)
        .clear(item_id, query.store, query.product_id.as_deref())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PUT /api/grocery-items/{id}/selection/quantity` — how many of one of the
/// item's products to buy, when it has several at that store.
///
/// 404 unknown item or product; 422 only one product there, or a count that
/// leaves no room for the others; 409 the item has been ordered.
async fn set_quantity(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<ItemSelectionQuantity>,
) -> Result<StatusCode, ApiError> {
    SelectionService::new(&state.pool, &state.stores)
        .set_quantity(item_id, body.store, &body.product_id, body.quantity)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PUT /api/order-stores` — which store each item's order buys from.
///
/// Used by the order screen's options, and by their Undo. 422, changing
/// nothing, when an item has no product chosen at its store.
async fn set_order_stores(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedJson(body): ValidatedJson<OrderStoresSet>,
) -> Result<StatusCode, ApiError> {
    let picks: Vec<_> = body
        .picks
        .iter()
        .map(|pick| (pick.grocery_item_id, pick.store))
        .collect();
    SelectionService::new(&state.pool, &state.stores)
        .buy_at(&picks)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/grocery-items/{id}/selection/snapshot` — save the item's
/// products before a change, for Undo. 404 unknown item.
async fn save_snapshot(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
) -> Result<(StatusCode, Json<SelectionSnapshotResponse>), ApiError> {
    let id = SelectionService::new(&state.pool, &state.stores)
        .snapshot(item_id)
        .await?;
    Ok((StatusCode::CREATED, Json(SelectionSnapshotResponse { id })))
}

/// `POST /api/selection-snapshots/{id}/restore[?grocery_item_id=]` — Undo:
/// put the saved products back in one step. 404 unknown snapshot or item;
/// 409 the item is no longer on the list.
async fn restore_snapshot(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(snapshot_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<SelectionRestoreQuery>,
) -> Result<StatusCode, ApiError> {
    SelectionService::new(&state.pool, &state.stores)
        .restore(snapshot_id, query.grocery_item_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
