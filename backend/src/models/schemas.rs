//! Request and response bodies for the intake and grocery-list API.
//!
//! Field names and constraints match the previous Pydantic schemas exactly,
//! so the existing web app needs no changes. Incoming bodies are validated
//! before they reach a service — length and range limits are enforced here,
//! not in SQL alone.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::db::{
    GroceryItem, GroceryItemSource, GroceryItemStatus, IntakeSource, VoiceRequest,
    VoiceRequestStatus,
};

/// Upper bound on a single item's quantity, matching the web app's input.
pub const MAX_QUANTITY: i32 = 999;
/// Upper bound on an item name, matching the `name` column's CHECK constraint.
pub const MAX_NAME_LEN: u64 = 200;
/// Upper bound on an item's free-text note, matching the `note` CHECK.
pub const MAX_NOTE_LEN: u64 = 500;
/// How many filter chips one item may carry, matching the `filter_terms` CHECK.
pub const MAX_FILTER_TERMS: usize = 10;
/// Longest single filter term, matching the `filter_terms` CHECK.
pub const MAX_FILTER_TERM_LEN: usize = 60;

fn default_quantity() -> i32 {
    1
}

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
        }
    }
}

/// A grocery list item as the web app sees it.
#[derive(Debug, Serialize)]
pub struct GroceryItemResponse {
    pub id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub status: GroceryItemStatus,
    pub source: GroceryItemSource,
    /// The free-text annotation, absent when the item has none.
    pub note: Option<String>,
    /// Filter chips, in the order they will be shown. Never null — an item
    /// with no chips serialises as `[]`, so the web app needs no fallback.
    pub filter_terms: Vec<String>,
    pub added_by_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<GroceryItem> for GroceryItemResponse {
    fn from(row: GroceryItem) -> Self {
        Self {
            id: row.id,
            name: row.name,
            quantity: row.quantity,
            status: row.status,
            source: row.source,
            note: row.note,
            filter_terms: row.filter_terms,
            added_by_user_id: row.added_by_user_id,
            created_at: row.created_at,
        }
    }
}

/// A manually added grocery item.
///
/// The web app's add-item form. Voice items do not use this — they arrive as
/// intake requests and reach the list only once a household member accepts
/// one.
#[derive(Debug, Deserialize, Validate)]
pub struct GroceryItemCreate {
    #[validate(length(min = 1, max = MAX_NAME_LEN))]
    pub name: String,
    #[serde(default = "default_quantity")]
    #[validate(range(min = 1, max = MAX_QUANTITY))]
    pub quantity: i32,
    #[validate(length(max = MAX_NOTE_LEN))]
    pub note: Option<String>,
    #[serde(default)]
    #[validate(custom(function = "validate_filter_terms"))]
    pub filter_terms: Option<Vec<String>>,
}

/// Changes to an item already on the list: review, modify and annotate.
///
/// Every field is optional and an absent field is left as it was. A `note` of
/// `""` (or only whitespace) clears the annotation; a `filter_terms` array
/// replaces the chips wholesale, which is how removing one chip is expressed.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct GroceryItemUpdate {
    #[validate(length(min = 1, max = MAX_NAME_LEN))]
    pub name: Option<String>,
    #[validate(range(min = 1, max = MAX_QUANTITY))]
    pub quantity: Option<i32>,
    #[validate(length(max = MAX_NOTE_LEN))]
    pub note: Option<String>,
    #[serde(default)]
    #[validate(custom(function = "validate_filter_terms"))]
    pub filter_terms: Option<Vec<String>>,
}

/// Rejects a chip list the `filter_terms` CHECK constraint would also reject.
///
/// The service trims and de-duplicates terms before storing them, so this only
/// has to catch what trimming cannot fix: too many chips, or one too long.
fn validate_filter_terms(terms: &[String]) -> Result<(), ValidationError> {
    if terms.len() > MAX_FILTER_TERMS {
        return Err(ValidationError::new("too_many_filter_terms"));
    }
    if terms
        .iter()
        .any(|term| term.trim().chars().count() > MAX_FILTER_TERM_LEN)
    {
        return Err(ValidationError::new("filter_term_too_long"));
    }
    Ok(())
}

/// Response after an intake request is accepted: the resulting list item.
#[derive(Debug, Serialize)]
pub struct VoiceRequestAcceptResult {
    pub voice_request: VoiceRequestResponse,
    pub grocery_item: GroceryItemResponse,
}

/// The `detail` payload of the 409 returned when accepting would duplicate an
/// active item, so the web app can offer the user a choice.
#[derive(Debug, Serialize)]
pub struct DuplicateItemWarning {
    pub existing_item: ExistingItem,
    pub message: String,
}

/// The minimal view of the clashing item the warning carries.
#[derive(Debug, Serialize)]
pub struct ExistingItem {
    pub id: Uuid,
    pub name: String,
    pub quantity: i32,
}

impl DuplicateItemWarning {
    /// Builds the warning for an active item that already covers this name.
    pub fn for_item(item: &GroceryItem) -> Self {
        Self {
            existing_item: ExistingItem {
                id: item.id,
                name: item.name.clone(),
                quantity: item.quantity,
            },
            message: format!(
                "{} is already on the list. Add another or update the existing quantity?",
                item.name
            ),
        }
    }
}
