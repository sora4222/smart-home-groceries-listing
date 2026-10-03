//! Removing, reducing and undoing list items by Alexa, forwarded by the
//! bridge sidecar.
//!
//! Authenticated exactly like `POST /api/intake/alexa` (see
//! [`super::alexa`]): the sidecar verified Amazon's signature and sends the
//! `X-Bridge-Secret` header. The rules live in
//! [`crate::services::voice_changes`].

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};

use super::alexa::verify_bridge;
use crate::error::ApiError;
use crate::models::db::IntakeSource;
use crate::models::schemas::{AlexaRemoveCreate, AlexaUndoCreate, VoiceListChangeResponse};
use crate::routes::extract::{OptionalValidatedJson, ValidatedJson};
use crate::services::voice::Delivery;
use crate::services::voice_changes;
use crate::state::AppState;

/// Routes under `/api/intake/alexa/`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/intake/alexa/remove", post(remove_item))
        .route("/api/intake/alexa/undo", post(undo_change))
}

/// `POST /api/intake/alexa/remove` — take an item off the list, or lower its
/// quantity when `quantity` is given.
///
/// 201 with the change made; 200 when `external_id` was already handled. 404
/// when no item on the list matches; 409 when it is committed for purchase.
async fn remove_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<AlexaRemoveCreate>,
) -> Result<(StatusCode, Json<VoiceListChangeResponse>), ApiError> {
    verify_bridge(&state, &headers)?;
    let (change, delivery) = voice_changes::remove(&state.pool, IntakeSource::Alexa, &body).await?;
    Ok((status_for(delivery), Json(change.into())))
}

/// `POST /api/intake/alexa/undo` — reverse the newest voice change.
///
/// 201 with the change reversed; 200 when `external_id` was already handled.
/// 404 when nothing recent is left to undo; 409 when it cannot be reversed.
async fn undo_change(
    State(state): State<AppState>,
    headers: HeaderMap,
    OptionalValidatedJson(body): OptionalValidatedJson<AlexaUndoCreate>,
) -> Result<(StatusCode, Json<VoiceListChangeResponse>), ApiError> {
    verify_bridge(&state, &headers)?;
    let (change, delivery) = voice_changes::undo_latest(
        &state.pool,
        IntakeSource::Alexa,
        body.external_id.as_deref(),
    )
    .await?;
    Ok((status_for(delivery), Json(change.into())))
}

/// 201 for a change made now, 200 for a retry of one already made.
fn status_for(delivery: Delivery) -> StatusCode {
    match delivery {
        Delivery::Recorded => StatusCode::CREATED,
        Delivery::AlreadySeen => StatusCode::OK,
    }
}
