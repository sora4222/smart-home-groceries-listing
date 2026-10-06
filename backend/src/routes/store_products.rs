//! One product at one store, by the store's own id.
//!
//! The route only reads. It answers with the product priced for one unit,
//! in the same shape as a list item's product search.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::ProductResponse;
use crate::services::product_lookup;
use crate::services::product_search::PricedProduct;
use crate::services::stores::Store;
use crate::state::AppState;

/// Routes under `/api/stores/{store}/products/{product_id}`.
pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/stores/{store}/products/{product_id}",
        get(store_product),
    )
}

/// `GET /api/stores/{store}/products/{product_id}` — `store` is
/// `woolworths` or `coles`; the id is the store's own (stockcode, Coles
/// product id). 404 when the store does not sell it, 503 when the store
/// could not be asked.
async fn store_product(
    State(state): State<AppState>,
    _user: AuthUser,
    Path((store, product_id)): Path<(Store, String)>,
) -> Result<Json<ProductResponse>, ApiError> {
    let product = product_lookup::find(&state.stores, store, &product_id).await?;
    Ok(Json(PricedProduct::new(product, 1).into()))
}
