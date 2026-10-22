//! Pushing the queue counts the web app's nav badges show.

use sqlx::PgPool;

use super::{repository, triage_repository};
use crate::error::ApiError;
use crate::services::ws_hub::{QueueCounts, ServerEvent, WsHub};

/// Reads the counts used by both connection snapshots and live broadcasts.
pub async fn current(pool: &PgPool) -> Result<QueueCounts, ApiError> {
    Ok(QueueCounts {
        voice_request_added: repository::pending_count(pool).await?,
        triage_held: triage_repository::held_count(pool).await?,
    })
}

/// Sends every open browser session the Pending Requests count and the
/// held-for-review count. Called after anything that can change either.
pub async fn publish(pool: &PgPool, hub: &WsHub) -> Result<(), ApiError> {
    let counts = current(pool).await?;
    hub.broadcast(ServerEvent::VoiceRequestAdded {
        count: counts.voice_request_added,
    });
    hub.broadcast(ServerEvent::TriageHeld {
        count: counts.triage_held,
    });
    Ok(())
}
