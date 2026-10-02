//! Pushing the queue counts the web app's nav badges show.

use sqlx::PgPool;

use super::{repository, triage_repository};
use crate::error::ApiError;
use crate::services::ws_hub::{ServerEvent, WsHub};

/// Sends every open browser session the Pending Requests count and the
/// held-for-review count. Called after anything that can change either.
pub async fn publish(pool: &PgPool, hub: &WsHub) -> Result<(), ApiError> {
    let pending = repository::pending_count(pool).await?;
    let held = triage_repository::held_count(pool).await?;
    hub.broadcast(ServerEvent::VoiceRequestAdded { count: pending });
    hub.broadcast(ServerEvent::TriageHeld { count: held });
    Ok(())
}
