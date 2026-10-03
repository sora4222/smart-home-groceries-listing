//! Row type for `voice_list_changes` and the enumeration its `kind` column
//! holds (migration `20261002200000_voice_list_changes`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::db::{GroceryItemSource, IntakeSource};

/// What a spoken change did to the list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum VoiceChangeKind {
    /// The quantity went down and the item stayed on the list.
    Reduced,
    /// The item left the list.
    Removed,
}

/// One remove or reduce made by voice, with a snapshot of the item before it
/// so Undo can put the item back.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VoiceListChange {
    pub id: Uuid,
    pub source: IntakeSource,
    /// The channel's request id; makes a retried delivery a no-op.
    pub external_id: Option<String>,
    pub kind: VoiceChangeKind,
    /// The item changed. A removed item keeps this id when Undo restores it.
    pub grocery_item_id: Uuid,
    pub item_name: String,
    pub quantity_before: i32,
    /// `0` exactly when `kind` is [`VoiceChangeKind::Removed`].
    pub quantity_after: i32,
    pub item_source: GroceryItemSource,
    pub item_note: Option<String>,
    pub item_filter_terms: Vec<String>,
    pub item_added_by_user_id: Option<String>,
    pub item_created_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    /// When Undo reversed the change; `None` while it stands.
    pub undone_at: Option<DateTime<Utc>>,
    /// The request id of the Undo; makes a retried Undo a no-op.
    pub undo_external_id: Option<String>,
}
