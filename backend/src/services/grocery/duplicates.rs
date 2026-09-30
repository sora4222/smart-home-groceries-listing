//! What happens when an addition names an item already on the active list.
//!
//! The spec's "Duplicate handling": the household is asked "add another or
//! update the existing quantity?". [`OnDuplicate`] is the answer a request
//! carries, and [`duplicate_error`] is the question when it carries none.

use crate::error::ApiError;
use crate::models::db::GroceryItem;
use crate::models::schemas::DuplicateItemWarning;

/// What to do when a manual addition names an item already on the list.
///
/// The spec offers the user a choice, so the default is to ask rather than to
/// pick for them: `Ask` answers 409 carrying the clashing item.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnDuplicate {
    /// Answer 409 with the existing item so the web app can put the question.
    #[default]
    Ask,
    /// Add the new quantity to the existing item.
    Merge,
    /// Keep both: a second entry under the same name, annotated differently.
    Separate,
}

/// Builds the 409 that lets the web app offer a merge.
///
/// Shared with the intake confirmation queue: whichever path is about to add a
/// second copy of an active item asks the user the same question.
pub fn duplicate_error(existing: &GroceryItem) -> ApiError {
    match serde_json::to_value(DuplicateItemWarning::for_item(existing)) {
        Ok(detail) => ApiError::DuplicateActiveItem { detail },
        Err(err) => ApiError::Internal(err.into()),
    }
}
