//! The intake confirmation queue.
//!
//! `POST /api/voice-requests` is the generic intake webhook (shared-secret
//! auth, reachable through the Cloudflare Tunnel — see docs/human-setup.md).
//! Every other route here requires a signed-in household member and drives
//! the "Pending Requests" screen in the web app.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::secret::verify_shared_secret;
use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{
    VoiceRequestAcceptResult, VoiceRequestCreate, VoiceRequestDecision, VoiceRequestResponse,
};
use crate::routes::extract::{OptionalValidatedJson, ValidatedJson};
use crate::services::voice::VoiceService;
use crate::state::AppState;

/// Header the generic webhook authenticates with.
pub const WEBHOOK_SECRET_HEADER: &str = "x-webhook-secret";

/// Routes under `/api/voice-requests`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/voice-requests",
            post(create_voice_request).get(list_pending_voice_requests),
        )
        .route("/api/voice-requests/{request_id}/accept", post(accept))
        .route("/api/voice-requests/{request_id}/reject", post(reject))
}

/// `?merge=true` folds the quantity into an existing active item.
#[derive(Debug, Deserialize)]
pub struct AcceptQuery {
    #[serde(default)]
    pub merge: bool,
}

/// `POST /api/voice-requests` — the generic intake webhook.
///
/// Authenticated by a shared secret rather than a session JWT, because the
/// callers (Home Assistant, IFTTT, `curl`) cannot hold a Clerk session.
async fn create_voice_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<VoiceRequestCreate>,
) -> Result<(StatusCode, Json<VoiceRequestResponse>), ApiError> {
    verify_shared_secret(
        &state.settings.voice_webhook_secret,
        headers
            .get(WEBHOOK_SECRET_HEADER)
            .and_then(|v| v.to_str().ok()),
        "VOICE_WEBHOOK_SECRET",
    )?;

    let request = VoiceService::new(&state.pool, &state.hub)
        .create_webhook_request(&body, &state.triage)
        .await?;
    Ok((StatusCode::CREATED, Json(request.into())))
}

/// `GET /api/voice-requests` — requests awaiting a decision.
async fn list_pending_voice_requests(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<VoiceRequestResponse>>, ApiError> {
    let requests = VoiceService::new(&state.pool, &state.hub)
        .list_pending()
        .await?;
    Ok(Json(requests.into_iter().map(Into::into).collect()))
}

/// `POST /api/voice-requests/{id}/accept` — confirm, with optional corrections.
async fn accept(
    State(state): State<AppState>,
    user: AuthUser,
    Path(request_id): Path<Uuid>,
    Query(query): Query<AcceptQuery>,
    OptionalValidatedJson(decision): OptionalValidatedJson<VoiceRequestDecision>,
) -> Result<Json<VoiceRequestAcceptResult>, ApiError> {
    let (request, item) = VoiceService::new(&state.pool, &state.hub)
        .accept(request_id, &decision, &user.id, query.merge)
        .await?;

    Ok(Json(VoiceRequestAcceptResult {
        voice_request: request.into(),
        grocery_item: item.into(),
    }))
}

/// `POST /api/voice-requests/{id}/reject` — mark rejected; the web app keeps
/// the card visible so accepting again is a one-click undo.
async fn reject(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(request_id): Path<Uuid>,
) -> Result<Json<VoiceRequestResponse>, ApiError> {
    let request = VoiceService::new(&state.pool, &state.hub)
        .reject(request_id)
        .await?;
    Ok(Json(request.into()))
}
