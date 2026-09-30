//! Row types for the tables in `migrations/`.
//!
//! Only the tables the voice-intake goal needs are defined here
//! (`voice_requests`, `grocery_items`). Later features (item rules, purchase
//! history, ...) add their own types alongside these without changing these.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Where an intake request came from.
///
/// `voice_requests` is kept as the table name to limit churn, but the concept
/// is an *intake* request: a spoken or forwarded item awaiting confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum IntakeSource {
    /// The generic shared-secret webhook (Home Assistant, IFTTT, curl, tests).
    Webhook,
    /// Forwarded by the Alexa bridge sidecar after it verified the request.
    Alexa,
}

/// Lifecycle of an intake request. An accepted request is a closed decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum VoiceRequestStatus {
    Pending,
    Accepted,
    Rejected,
}

/// Lifecycle of an item on the household list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GroceryItemStatus {
    Pending,
    Active,
    Ordered,
}

/// How an item reached the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GroceryItemSource {
    Voice,
    Manual,
}

/// A raw voice-added item, unconfirmed until a household member acts on it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VoiceRequest {
    pub id: Uuid,
    pub source: IntakeSource,
    /// The channel's own identifier for this item, when it has one. Makes
    /// re-delivery (an Alexa retry, a re-poll) idempotent.
    pub external_id: Option<String>,
    pub raw_text: String,
    pub parsed_name: String,
    pub parsed_quantity: i32,
    pub status: VoiceRequestStatus,
    pub grocery_item_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// An item on the shared household grocery list.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GroceryItem {
    pub id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub status: GroceryItemStatus,
    pub source: GroceryItemSource,
    pub added_by_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
