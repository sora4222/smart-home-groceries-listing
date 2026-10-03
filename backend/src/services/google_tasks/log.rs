//! Google Tasks log events, one function per thing that happened. Task ids
//! and counts are logged; tokens, codes and states never are.

use super::api::GoogleError;
use crate::error::ApiError;
use crate::models::db::{GoogleTasksLink, VoiceRequest};
use crate::models::schemas::PollReport;

/// A household member connected Google Tasks.
pub fn connected(user_id: &str) {
    tracing::info!(user_id, "google tasks connected");
}

/// A sign-in came back with a state this app did not issue, or reused one.
pub fn sign_in_refused(user_id: &str) {
    tracing::warn!(
        user_id,
        "google sign-in refused: unknown, expired or reused state"
    );
}

/// A household member disconnected Google Tasks.
pub fn disconnected(user_id: &str) {
    tracing::info!(user_id, "google tasks disconnected");
}

/// The list, polling switch or interval changed.
pub fn choices_saved(link: &GoogleTasksLink, user_id: &str) {
    tracing::info!(
        list = ?link.task_list_title,
        enabled = link.enabled,
        poll_seconds = link.poll_seconds,
        user_id,
        "google tasks settings saved"
    );
}

/// A poll finished. Quiet unless it found something.
pub fn polled(report: &PollReport) {
    if report == &PollReport::default() {
        tracing::debug!("google tasks poll: nothing new");
    } else {
        tracing::info!(
            recorded = report.recorded,
            already_seen = report.already_seen,
            blank = report.blank,
            not_deleted = report.not_deleted,
            "google tasks polled"
        );
    }
}

/// A poll could not run to the end.
pub fn poll_failed(err: &ApiError) {
    tracing::warn!(%err, "google tasks poll failed");
}

/// A recorded task could not be deleted; the next poll tries again.
pub fn delete_failed(task_id: &str, err: &GoogleError) {
    tracing::warn!(task_id, %err, "google task recorded but not deleted");
}

/// A person put a triaged item back on the Google Tasks list.
pub fn restored(request: &VoiceRequest, task_id: &str, user_id: &str) {
    tracing::info!(
        request_id = %request.id,
        item = %request.parsed_name,
        task_id,
        user_id,
        "intake request restored to google tasks"
    );
}
