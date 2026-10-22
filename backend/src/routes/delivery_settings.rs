//! Settings › Delivery: each store's delivery fee rules and the order
//! planner's default mode and delivery spending cap.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::DeliverySettingsBody;
use crate::routes::extract::ValidatedJson;
use crate::services::delivery_settings::DeliverySettingsService;
use crate::state::AppState;

/// `GET` and `PUT /api/delivery-settings`.
pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/delivery-settings",
        get(read_settings).put(save_settings),
    )
}

/// `GET /api/delivery-settings` — every store's rules and the preferences.
async fn read_settings(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<DeliverySettingsBody>, ApiError> {
    let settings = DeliverySettingsService::new(&state.pool).read().await?;
    Ok(Json(settings.into()))
}

/// `PUT /api/delivery-settings` — saves them; 422 for an amount out of range.
/// Answers with the settings as they now stand, which the web app keeps for
/// its Undo.
async fn save_settings(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<DeliverySettingsBody>,
) -> Result<Json<DeliverySettingsBody>, ApiError> {
    let settings = DeliverySettingsService::new(&state.pool)
        .save(
            &body.fee_rules(),
            body.mode,
            body.max_delivery_spend,
            &user.id,
        )
        .await?;
    Ok(Json(settings.into()))
}
