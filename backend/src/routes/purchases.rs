//! The purchase history: saved shops, Undo, and a product's past purchases.
//!
//! Shops are saved by the trolley fill itself (`routes/store_tab.rs`); these
//! routes read them and undo one saved by mistake.

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    ProductHistoryLookup, ProductHistoryResponse, PurchaseOrderResponse, RecategoriseResponse,
    UndoPurchaseResponse,
};
use crate::routes::extract::ValidatedJson;
use crate::services::purchases::PurchaseService;
use crate::state::AppState;

/// `/api/purchase-orders` and `/api/purchase-history/*`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/purchase-orders", get(list_orders))
        .route("/api/purchase-orders/{order_id}", delete(undo_order))
        .route("/api/purchase-history/products", post(product_history))
        .route("/api/purchase-history/recategorise", post(recategorise))
}

/// `GET /api/purchase-orders` — the newest saved shops, newest first.
async fn list_orders(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<PurchaseOrderResponse>>, ApiError> {
    let orders = PurchaseService::new(&state.pool).recent_orders().await?;
    Ok(Json(orders.into_iter().map(Into::into).collect()))
}

/// `DELETE /api/purchase-orders/{id}` — Undo: forget a shop saved by
/// mistake and put its items back on the list. 404 when it is already gone.
async fn undo_order(
    State(state): State<AppState>,
    user: AuthUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<UndoPurchaseResponse>, ApiError> {
    let undone = PurchaseService::new(&state.pool)
        .undo(order_id, &user.id)
        .await?;
    Ok(Json(UndoPurchaseResponse {
        items_restored: undone.items_restored,
    }))
}

/// `POST /api/purchase-history/products` — how many times each product was
/// bought and what it cost each time. A read with a body, because a
/// comparison asks about many products at once. Products never bought are
/// left out.
async fn product_history(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedJson(body): ValidatedJson<ProductHistoryLookup>,
) -> Result<Json<Vec<ProductHistoryResponse>>, ApiError> {
    let products: Vec<_> = body
        .products
        .into_iter()
        .map(|product| (product.store, product.product_id))
        .collect();
    let history = PurchaseService::new(&state.pool)
        .product_history(&products)
        .await?;
    Ok(Json(history.into_iter().map(Into::into).collect()))
}

/// `POST /api/purchase-history/recategorise` — work out every purchase's
/// category again with today's keyword list.
async fn recategorise(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<RecategoriseResponse>, ApiError> {
    let changed = PurchaseService::new(&state.pool).recategorise().await?;
    Ok(Json(RecategoriseResponse { changed }))
}
