//! Row types for the tables in `migrations/`.
//!
//! One type per table (`voice_requests`, `grocery_items`, `item_rules`) plus
//! the enumerations their TEXT + CHECK columns hold. Later features (purchase
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
    /// Reviewed and locked in for purchase. A committed item is read-only
    /// until the list is released back to `Active`.
    Committed,
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
    /// Free text a household member attached while reviewing the list.
    pub note: Option<String>,
    /// Terms narrowing the later product search, shown as removable chips.
    pub filter_terms: Vec<String>,
    pub added_by_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A persistent filter attached to grocery item names.
///
/// When an item whose name matches one of `triggers` reaches the list, the
/// rule's `filter_terms` are copied onto it as chips — always for accepted
/// voice items, and for web-app additions only when `apply_to_manual` is set.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemRule {
    pub id: Uuid,
    /// Item names or phrases the rule matches, case-insensitively.
    pub triggers: Vec<String>,
    /// The chips a matching item receives.
    pub filter_terms: Vec<String>,
    /// Whether items typed into the web app get the rule too.
    pub apply_to_manual: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
