//! Bodies for the intake channels and the confirmation queue.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::common::{default_quantity, MAX_NAME_LEN, MAX_QUANTITY};
use super::grocery::GroceryItemResponse;
use crate::models::db::{IntakeSource, TriageStatus, VoiceRequest, VoiceRequestStatus};

/// Payload the generic intake webhook accepts.
#[derive(Debug, Deserialize, Validate)]
pub struct VoiceRequestCreate {
    #[validate(length(min = 1, max = MAX_NAME_LEN))]
    pub item: String,
    #[serde(default = "default_quantity")]
    #[validate(range(min = 1, max = MAX_QUANTITY))]
    pub quantity: i32,
}

/// Payload the Alexa bridge sidecar forwards after verifying the request.
///
/// `external_id` is Alexa's own identifier for the utterance or list item; it
/// makes a retried delivery a no-op rather than a duplicate card.
#[derive(Debug, Deserialize, Validate)]
pub struct AlexaIntakeCreate {
    #[validate(length(min = 1, max = MAX_NAME_LEN))]
    pub item: String,
    #[serde(default = "default_quantity")]
    #[validate(range(min = 1, max = MAX_QUANTITY))]
    pub quantity: i32,
    #[validate(length(min = 1, max = 255))]
    pub external_id: Option<String>,
    /// The utterance as Alexa heard it, when the sidecar can supply it.
    #[validate(length(max = 2000))]
    pub raw_text: Option<String>,
}

/// Corrections a user makes before accepting an intake request.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct VoiceRequestDecision {
    #[validate(length(min = 1, max = MAX_NAME_LEN))]
    pub name: Option<String>,
    #[validate(range(min = 1, max = MAX_QUANTITY))]
    pub quantity: Option<i32>,
}

/// An intake request as the web app sees it.
#[derive(Debug, Serialize)]
pub struct VoiceRequestResponse {
    pub id: Uuid,
    pub source: IntakeSource,
    pub raw_text: String,
    pub parsed_name: String,
    pub parsed_quantity: i32,
    pub status: VoiceRequestStatus,
    pub created_at: DateTime<Utc>,
    /// Which queue triage put the request in.
    pub triage_status: TriageStatus,
    /// The classifier's reason, or why it could not answer.
    pub triage_reason: Option<String>,
    /// The classifier's confidence, 0 to 1, when it answered.
    pub triage_confidence: Option<f32>,
}

impl From<VoiceRequest> for VoiceRequestResponse {
    fn from(row: VoiceRequest) -> Self {
        Self {
            id: row.id,
            source: row.source,
            raw_text: row.raw_text,
            parsed_name: row.parsed_name,
            parsed_quantity: row.parsed_quantity,
            status: row.status,
            created_at: row.created_at,
            triage_status: row.triage_status,
            triage_reason: row.triage_reason,
            triage_confidence: row.triage_confidence,
        }
    }
}

/// Response after an intake request is accepted: the resulting list item.
#[derive(Debug, Serialize)]
pub struct VoiceRequestAcceptResult {
    pub voice_request: VoiceRequestResponse,
    pub grocery_item: GroceryItemResponse,
}
