//! Business rules for the household grocery list.
//!
//! This is the "review, modify and annotate before committing to purchase"
//! half of the application: reading the list, adding to it by hand, editing an
//! item's name, quantity, note and filter chips, removing an item, and locking
//! the whole list in for purchase. Items arriving by voice are the intake
//! queue's business ([`crate::services::voice`]); they become this module's
//! once accepted.
//!
//! SQL lives in [`repository`]. Every method that could race another browser
//! tab runs in one transaction and locks the row it decides on first.

pub mod repository;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemSource, GroceryItemStatus};
use crate::models::schemas::{
    DuplicateItemWarning, GroceryItemCreate, GroceryItemUpdate, MAX_FILTER_TERMS, MAX_QUANTITY,
};

/// Reads and writes the household list.
pub struct GroceryService<'a> {
    pool: &'a PgPool,
}

impl<'a> GroceryService<'a> {
    /// Borrows the pool for the duration of one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Everything on the list, newest first: items under review and items
    /// already committed for purchase.
    pub async fn list(&self) -> Result<Vec<GroceryItem>, ApiError> {
        repository::list_items(self.pool).await
    }

    /// Adds an item by hand.
    ///
    /// Returns [`ApiError::DuplicateActiveItem`] when an active item already
    /// covers the name and `merge` is false, so the web app can ask whether to
    /// add another or update the existing quantity — the spec's duplicate
    /// handling, applied inline for manual additions. `merge = true` adds to
    /// the existing quantity instead, capped at [`MAX_QUANTITY`] to stay
    /// inside the column's constraint.
    pub async fn add(
        &self,
        payload: &GroceryItemCreate,
        user_id: &str,
        merge: bool,
    ) -> Result<GroceryItem, ApiError> {
        let name = payload.name.trim();
        let note = clean_note(payload.note.as_deref());
        let filter_terms = clean_filter_terms(payload.filter_terms.as_deref().unwrap_or(&[]));

        let mut tx = self.pool.begin().await?;
        let item = match repository::lock_active_duplicate(&mut tx, name).await? {
            Some(existing) if !merge => {
                // Nothing has been written yet; the rollback only releases the
                // lock before the 409 goes out.
                tx.rollback().await?;
                return Err(duplicate_error(&existing));
            }
            Some(existing) => {
                let merged = existing
                    .quantity
                    .saturating_add(payload.quantity)
                    .min(MAX_QUANTITY);
                repository::set_item_quantity(&mut tx, existing.id, merged).await?
            }
            None => {
                repository::insert_item(
                    &mut tx,
                    name,
                    payload.quantity,
                    GroceryItemSource::Manual,
                    note.as_deref(),
                    &filter_terms,
                    user_id,
                )
                .await?
            }
        };
        tx.commit().await?;
        Ok(item)
    }

    /// Applies an edit to an item on the list.
    ///
    /// Absent fields are left as they were. A committed item is locked, so an
    /// edit to one is a 409 rather than a silent no-op: the user has to release
    /// the list before changing it again.
    pub async fn update(
        &self,
        item_id: Uuid,
        payload: &GroceryItemUpdate,
    ) -> Result<GroceryItem, ApiError> {
        let mut tx = self.pool.begin().await?;
        let existing = self.locked_editable_item(&mut tx, item_id).await?;

        let name = payload
            .name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(&existing.name)
            .to_string();
        let quantity = payload.quantity.unwrap_or(existing.quantity);
        // An absent note leaves the annotation alone; an empty one clears it.
        let note = match payload.note.as_deref() {
            Some(text) => clean_note(Some(text)),
            None => existing.note.clone(),
        };
        let filter_terms = match payload.filter_terms.as_deref() {
            Some(terms) => clean_filter_terms(terms),
            None => existing.filter_terms.clone(),
        };

        let item = repository::update_item(
            &mut tx,
            item_id,
            &name,
            quantity,
            note.as_deref(),
            &filter_terms,
        )
        .await?;
        tx.commit().await?;
        Ok(item)
    }

    /// Removes an item from the list. A committed item is locked, as for edits.
    pub async fn delete(&self, item_id: Uuid) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        self.locked_editable_item(&mut tx, item_id).await?;
        repository::delete_item(&mut tx, item_id).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Locks in the whole list for purchase: every active item becomes
    /// committed. Returns the items it changed, which is empty when there was
    /// nothing under review — pressing the button twice is harmless.
    pub async fn commit_list(&self) -> Result<Vec<GroceryItem>, ApiError> {
        repository::move_all(
            self.pool,
            GroceryItemStatus::Active,
            GroceryItemStatus::Committed,
        )
        .await
    }

    /// Reopens a committed list for editing.
    pub async fn release_list(&self) -> Result<Vec<GroceryItem>, ApiError> {
        repository::move_all(
            self.pool,
            GroceryItemStatus::Committed,
            GroceryItemStatus::Active,
        )
        .await
    }

    /// Locks an item and refuses if it is not open to changes.
    ///
    /// Rolls the transaction back before returning the error so the lock is
    /// released rather than held until the connection is reclaimed.
    async fn locked_editable_item(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item_id: Uuid,
    ) -> Result<GroceryItem, ApiError> {
        match repository::lock_item(tx, item_id).await? {
            Some(item) if item.status == GroceryItemStatus::Active => Ok(item),
            Some(_) => Err(ApiError::Conflict(format!(
                "{item_id} is committed for purchase — release the list to change it"
            ))),
            None => Err(ApiError::NotFound(item_id.to_string())),
        }
    }
}

/// Trims an annotation, treating a blank one as no annotation at all.
fn clean_note(note: Option<&str>) -> Option<String> {
    note.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// Tidies the chip list a client sent: trims each term, drops blanks, removes
/// case-insensitive repeats keeping the first spelling, and caps the count.
///
/// The bounds are also `CHECK` constraints on `filter_terms`; doing the work
/// here means a client that sends `["3 Ply", " 3 ply "]` gets one chip instead
/// of a 422.
fn clean_filter_terms(terms: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut cleaned: Vec<String> = Vec::new();
    for term in terms {
        let term = term.trim();
        if term.is_empty() || cleaned.len() >= MAX_FILTER_TERMS {
            continue;
        }
        let key = term.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        cleaned.push(term.to_string());
    }
    cleaned
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

#[cfg(test)]
mod tests {
    use super::{clean_filter_terms, clean_note};

    #[test]
    fn a_blank_note_is_no_note() {
        assert_eq!(clean_note(Some("   ")), None);
        assert_eq!(clean_note(Some("")), None);
        assert_eq!(clean_note(None), None);
    }

    #[test]
    fn a_note_is_trimmed() {
        assert_eq!(
            clean_note(Some("  the recycled one ")).as_deref(),
            Some("the recycled one")
        );
    }

    #[test]
    fn filter_terms_are_trimmed_and_blanks_dropped() {
        let terms = vec![
            "  3 ply ".to_string(),
            "  ".to_string(),
            "recycled".to_string(),
        ];
        assert_eq!(clean_filter_terms(&terms), vec!["3 ply", "recycled"]);
    }

    #[test]
    fn filter_terms_keep_the_first_spelling_of_a_repeat() {
        let terms = vec![
            "3 Ply".to_string(),
            "3 ply".to_string(),
            " 3 PLY ".to_string(),
        ];
        assert_eq!(clean_filter_terms(&terms), vec!["3 Ply"]);
    }

    #[test]
    fn filter_terms_are_capped_at_the_column_limit() {
        let terms: Vec<String> = (0..25).map(|n| format!("term {n}")).collect();
        assert_eq!(clean_filter_terms(&terms).len(), 10);
    }

    #[test]
    fn filter_terms_of_an_empty_list_stay_empty() {
        assert!(clean_filter_terms(&[]).is_empty());
    }
}
