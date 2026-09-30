//! Bodies for the household grocery list.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::common::{
    default_quantity, validate_filter_terms, MAX_NAME_LEN, MAX_NOTE_LEN, MAX_QUANTITY,
};
use crate::models::db::{GroceryItem, GroceryItemSource, GroceryItemStatus};

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
