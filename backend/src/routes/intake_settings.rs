//! `GET /api/intake/settings` — what `/settings/intake` shows about every
//! intake channel and the triage checker. Never returns a secret: only
//! whether each one is set.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{IntakeSettingsResponse, SecretChannelStatus, TriageStatusResponse};
use crate::routes::google_tasks::status_of;
use crate::services::google_tasks::TasksConnection;
use crate::state::AppState;

/// Routes under `/api/intake/settings`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/intake/settings", get(show))
}

/// `GET /api/intake/settings`.
async fn show(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<IntakeSettingsResponse>, ApiError> {
    let link = TasksConnection::new(&state.pool, &state.google_tasks, state.encryptor.as_ref())
        .link()
        .await?;
    let settings = &state.settings;
    Ok(Json(IntakeSettingsResponse {
        google_tasks: status_of(&state, link.as_ref()),
        alexa: SecretChannelStatus {
            configured: !settings.alexa_bridge_secret.is_empty(),
        },
        webhook: SecretChannelStatus {
            configured: !settings.voice_webhook_secret.is_empty(),
        },
        triage: TriageStatusResponse {
            provider: format!("{:?}", settings.triage.provider).to_ascii_lowercase(),
            model: settings.triage.model.clone(),
        },
    }))
}
