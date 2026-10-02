//! Triage log events, one function per thing that happened. Item names are
//! logged; keys and classifier responses never are.

use crate::models::db::VoiceRequest;

/// The classifier answered (or failed) and the outcome was stored.
pub fn assessed(request: &VoiceRequest) {
    tracing::info!(
        request_id = %request.id,
        item = %request.parsed_name,
        triage_status = ?request.triage_status,
        confidence = ?request.triage_confidence,
        "intake request triaged"
    );
}

/// Requests left unchecked by a restart are being checked again.
pub fn rechecking(count: usize) {
    tracing::info!(count, "re-checking intake requests left unchecked");
}

/// A background check could not store its outcome.
pub fn check_failed(request_id: uuid::Uuid, err: &crate::error::ApiError) {
    tracing::error!(%request_id, %err, "intake triage could not be stored");
}

/// A person moved a held or rejected request on to Pending Requests.
pub fn moved_to_pending(request: &VoiceRequest, user_id: &str) {
    tracing::info!(
        request_id = %request.id,
        item = %request.parsed_name,
        user_id,
        "triage overridden: request moved to pending"
    );
}
