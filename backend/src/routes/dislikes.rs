//! Product dislikes, and overriding them for one order.
//!
//! `/api/product-dislikes` is the household view and each member's own
//! dislikes; `/api/dislike-overrides` and
//! `/api/grocery-items/{id}/dislike-override` set a dislike aside for one
//! list item ("buy it this time anyway").

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    DislikeOverrideCreate, DislikeOverrideResponse, ProductDislikeCreate, ProductDislikeResponse,
};
use crate::routes::extract::ValidatedJson;
use crate::services::dislikes::{DislikeService, DislikedProduct, Member};
use crate::services::stores::Store;
use crate::state::AppState;

/// Every dislike route.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/product-dislikes",
            get(list_dislikes).put(dislike_product),
        )
        .route(
            "/api/product-dislikes/{store}/{product_id}",
            delete(remove_my_dislike),
        )
        .route("/api/dislike-overrides", get(list_overrides))
        .route(
            "/api/grocery-items/{item_id}/dislike-override",
            put(override_for_item),
        )
        .route(
            "/api/grocery-items/{item_id}/dislike-override/{store}/{product_id}",
            delete(clear_override),
        )
}

/// `GET /api/product-dislikes` — every member's dislikes, newest first.
async fn list_dislikes(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ProductDislikeResponse>>, ApiError> {
    let dislikes = DislikeService::new(&state.pool).list_household().await?;
    Ok(Json(
        dislikes
            .into_iter()
            .map(|row| ProductDislikeResponse::for_viewer(row, &user.id))
            .collect(),
    ))
}

/// `PUT /api/product-dislikes` — the caller dislikes the product.
async fn dislike_product(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<ProductDislikeCreate>,
) -> Result<Json<ProductDislikeResponse>, ApiError> {
    let member = Member {
        user_id: &user.id,
        email: user.email.as_deref(),
    };
    let product = DislikedProduct {
        store: body.store,
        product_id: &body.product_id,
        name: &body.name,
        brand: body.brand.as_deref(),
        package_size: body.package_size.as_deref(),
    };
    let saved = DislikeService::new(&state.pool)
        .dislike(member, product)
        .await?;
    Ok(Json(ProductDislikeResponse::for_viewer(saved, &user.id)))
}

/// `DELETE /api/product-dislikes/{store}/{product_id}` — removes the
/// caller's own dislike. 204 also when they had none.
async fn remove_my_dislike(
    State(state): State<AppState>,
    user: AuthUser,
    Path((store, product_id)): Path<(Store, String)>,
) -> Result<StatusCode, ApiError> {
    DislikeService::new(&state.pool)
        .remove_mine(&user.id, store, &product_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/dislike-overrides` — every "buy it this time anyway".
async fn list_overrides(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<DislikeOverrideResponse>>, ApiError> {
    let overrides = DislikeService::new(&state.pool).list_overrides().await?;
    Ok(Json(overrides.into_iter().map(Into::into).collect()))
}

/// `PUT /api/grocery-items/{id}/dislike-override` — buy the disliked product
/// for this item anyway. 404 unknown item; 409 not on the list any more;
/// 422 nobody dislikes it.
async fn override_for_item(
    State(state): State<AppState>,
    user: AuthUser,
    Path(item_id): Path<Uuid>,
    ValidatedJson(body): ValidatedJson<DislikeOverrideCreate>,
) -> Result<Json<DislikeOverrideResponse>, ApiError> {
    let saved = DislikeService::new(&state.pool)
        .override_for_item(item_id, body.store, &body.product_id, &user.id)
        .await?;
    Ok(Json(saved.into()))
}

/// `DELETE /api/grocery-items/{id}/dislike-override/{store}/{product_id}` —
/// the dislikes count again. 204 also when there was no override.
async fn clear_override(
    State(state): State<AppState>,
    _user: AuthUser,
    Path((item_id, store, product_id)): Path<(Uuid, Store, String)>,
) -> Result<StatusCode, ApiError> {
    DislikeService::new(&state.pool)
        .clear_override(item_id, store, &product_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
