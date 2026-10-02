//! The Triage view: intake requests the classifier held or rejected.
//!
//! Every route needs a signed-in household member. Triage never accepts an
//! item: "Accept" here only moves a request on to Pending Requests, where a
//! person still accepts it onto the list (`docs/features/FEATURE_TRIAGE.md`).

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::VoiceRequestResponse;
use crate::services::triage::{TriageReview, TriageTab};
use crate::services::voice::VoiceService;
use crate::state::AppState;

/// Routes under `/api/triage`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/triage", get(list_tab))
        .route("/api/triage/{request_id}/accept", post(move_to_pending))
        .route("/api/triage/{request_id}/reject", post(confirm_rejection))
}

/// `?tab=held|rejected` — which tab to list.
#[derive(Debug, Deserialize)]
struct TabQuery {
    tab: TriageTab,
}

/// `GET /api/triage?tab=held|rejected` — undecided requests in one tab.
async fn list_tab(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(query): Query<TabQuery>,
) -> Result<Json<Vec<VoiceRequestResponse>>, ApiError> {
    let requests = TriageReview::new(&state.pool, &state.hub)
        .list(query.tab)
        .await?;
    Ok(Json(requests.into_iter().map(Into::into).collect()))
}

/// `POST /api/triage/{id}/accept` — override triage; the request moves on to
/// Pending Requests.
async fn move_to_pending(
    State(state): State<AppState>,
    user: AuthUser,
    Path(request_id): Path<Uuid>,
) -> Result<Json<VoiceRequestResponse>, ApiError> {
    let request = TriageReview::new(&state.pool, &state.hub)
        .move_to_pending(request_id, &user.id)
        .await?;
    Ok(Json(request.into()))
}

/// `POST /api/triage/{id}/reject` — confirm the rejection; the request is
/// recorded as rejected and leaves the Triage view.
async fn confirm_rejection(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(request_id): Path<Uuid>,
) -> Result<Json<VoiceRequestResponse>, ApiError> {
    let request = VoiceService::new(&state.pool, &state.hub)
        .reject(request_id)
        .await?;
    Ok(Json(request.into()))
}
