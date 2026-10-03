//! Row types for the tables in `migrations/`.
//!
//! One type per table (`voice_requests`, `grocery_items`, `item_rules`,
//! `item_selections`; trolley handoffs in `trolley_handoff_rows`) plus
//! the enumerations their TEXT + CHECK columns hold. Later features (purchase
//! history, ...) add their own types alongside these without changing these.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::stores::{Basis, Store};

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
    /// A task polled from the household's Google Tasks list.
    Tasks,
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

/// Where the LLM triage step left an intake request
/// (`docs/features/FEATURE_TRIAGE.md`). Triage never accepts an item; it
/// only decides which queue a person sees it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TriageStatus {
    /// Recorded; the classifier has not answered yet. Shown nowhere.
    Unchecked,
    /// A plausible supermarket item; shown in Pending Requests.
    Approved,
    /// Not a supermarket item; shown in the Triage view's Rejected tab.
    Rejected,
    /// Low confidence or a classifier failure; Held for review.
    Held,
    /// Triage bypassed: switched off, or a person moved the item on.
    Skipped,
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
    /// Which queue triage put the request in.
    pub triage_status: TriageStatus,
    /// The classifier's one-sentence reason, or why it could not answer.
    pub triage_reason: Option<String>,
    /// The classifier's confidence, 0 to 1, when it answered.
    pub triage_confidence: Option<f32>,
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

/// The one product chosen for a list item, as the store described it when it
/// was chosen. The order screen reads these to know what to buy.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemSelection {
    pub id: Uuid,
    pub grocery_item_id: Uuid,
    pub store: Store,
    /// The store's own id for the product.
    pub product_id: String,
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    /// Shelf price for one, when the store showed one.
    pub price: Option<Decimal>,
    /// The normalised unit price; present exactly when `unit_price_per` is.
    pub unit_price: Option<Decimal>,
    pub unit_price_per: Option<Basis>,
    /// What `priced_quantity` cost with the best deal applied.
    pub total_price: Option<Decimal>,
    /// The item's quantity when the product was chosen.
    pub priced_quantity: i32,
    pub url: String,
    /// The household member who chose it.
    pub selected_by: String,
    pub selected_at: DateTime<Utc>,
}

// Trolley handoff and Google Tasks rows live in their own files; re-exported
// so callers keep importing them from `models::db`.
pub use super::google_tasks_rows::*;
pub use super::trolley_handoff_rows::*;

// Purchase history rows, likewise.
pub use super::purchase_rows::*;

// Voice list changes (remove / reduce by voice, and their Undo), likewise.
pub use super::voice_change_rows::*;
