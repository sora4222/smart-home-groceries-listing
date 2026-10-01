//! The order review: what the committed list will cost, checked now.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::OrderReviewResponse;
use crate::services::order_review::OrderReviewService;
use crate::state::AppState;

/// `/api/order-review`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/order-review", get(review_order))
}

/// `GET /api/order-review` — every committed item, its chosen product
/// re-priced at its store now, grouped by store with subtotals.
///
/// A store that cannot be asked does not fail the request: its lines carry
/// `status: "store_failed"` and a `problem` sentence instead of a price.
async fn review_order(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<OrderReviewResponse>, ApiError> {
    let review = OrderReviewService::new(&state.pool, &state.stores)
        .review()
        .await?;
    Ok(Json(review.into()))
}
