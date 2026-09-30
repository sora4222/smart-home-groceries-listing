//! Alexa intake, forwarded by the bridge sidecar.
//!
//! Alexa's own request signing is verified by `sidecars/alexa-bridge`, which
//! runs Amazon's `ask-sdk` (there is no maintained Rust equivalent). The
//! sidecar then forwards the extracted item here over loopback.
//!
//! This endpoint therefore trusts the sidecar, and the deployment must make
//! that trust sound:
//!
//! * `ALEXA_BRIDGE_SECRET` authenticates the sidecar, compared in constant
//!   time. It is a different secret from `VOICE_WEBHOOK_SECRET` so that a
//!   leaked webhook secret does not also grant the Alexa path.
//! * Nothing but the sidecar should be able to reach this port. In the
//!   Compose deployment the backend is not published to the host, and only
//!   the sidecar is exposed through the Cloudflare Tunnel.
//!
//! Alexa retries an endpoint it believes timed out, so delivery is idempotent
//! on `external_id`.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};

use crate::auth::secret::verify_shared_secret;
use crate::error::ApiError;
use crate::models::schemas::{AlexaIntakeCreate, VoiceRequestResponse};
use crate::routes::extract::ValidatedJson;
use crate::services::voice::{Delivery, VoiceService};
use crate::state::AppState;

/// Header the bridge sidecar authenticates with.
pub const BRIDGE_SECRET_HEADER: &str = "x-bridge-secret";

/// Routes under `/api/intake`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/intake/alexa", post(create_alexa_request))
}

/// `POST /api/intake/alexa` — record a spoken item Alexa passed on.
///
/// Returns 201 for a newly recorded item and 200 when an `external_id` has
/// already been seen, so the sidecar can tell a fresh item from a retry.
async fn create_alexa_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<AlexaIntakeCreate>,
) -> Result<(StatusCode, Json<VoiceRequestResponse>), ApiError> {
    verify_shared_secret(
        &state.settings.alexa_bridge_secret,
        headers
            .get(BRIDGE_SECRET_HEADER)
            .and_then(|v| v.to_str().ok()),
        "ALEXA_BRIDGE_SECRET",
    )?;

    let (request, delivery) = VoiceService::new(&state.pool, &state.hub)
        .create_alexa_request(&body)
        .await?;

    let status = match delivery {
        Delivery::Recorded => StatusCode::CREATED,
        Delivery::AlreadySeen => StatusCode::OK,
    };
    Ok((status, Json(request.into())))
}
