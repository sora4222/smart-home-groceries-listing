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

mod annotations;
mod duplicates;
mod log;
pub mod repository;

pub use duplicates::{duplicate_error, OnDuplicate};

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemSource, GroceryItemStatus};
use crate::models::schemas::{GroceryItemCreate, GroceryItemUpdate, MAX_QUANTITY};
use crate::services::filter_terms;
use crate::services::item_rules::{self, AddedVia};
use crate::services::selections;
use annotations::clean_note;

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
    /// When an active item already covers the name, `on_duplicate` decides:
    /// [`OnDuplicate::Ask`] returns [`ApiError::DuplicateActiveItem`] so the
    /// web app can ask whether to add another or update the existing quantity —
    /// the spec's duplicate handling, applied inline for manual additions;
    /// [`OnDuplicate::Merge`] adds to the existing quantity, capped at
    /// [`MAX_QUANTITY`] to stay inside the column's constraint; and
    /// [`OnDuplicate::Separate`] keeps both entries.
    ///
    /// A new entry also gets the chips of every item rule that matches its
    /// name and has "apply to manual additions" switched on, after the chips
    /// the user typed.
    pub async fn add(
        &self,
        payload: &GroceryItemCreate,
        user_id: &str,
        on_duplicate: OnDuplicate,
    ) -> Result<GroceryItem, ApiError> {
        let name = payload.name.trim();
        let note = clean_note(payload.note.as_deref());
        let filter_terms = filter_terms::clean(payload.filter_terms.as_deref().unwrap_or(&[]));

        let mut tx = self.pool.begin().await?;
        let clash = repository::lock_active_duplicate(&mut tx, name).await?;
        let (item, how) = match (clash, on_duplicate) {
            (Some(existing), OnDuplicate::Ask) => {
                // Nothing has been written yet; the rollback only releases the
                // lock before the 409 goes out.
                tx.rollback().await?;
                log::duplicate_asked(name, &existing);
                return Err(duplicate_error(&existing));
            }
            (Some(existing), OnDuplicate::Merge) => {
                let merged = existing
                    .quantity
                    .saturating_add(payload.quantity)
                    .min(MAX_QUANTITY);
                let item = repository::set_item_quantity(&mut tx, existing.id, merged).await?;
                (item, log::Added::Merged)
            }
            // No clash, or the user asked for both entries to exist. Only a
            // new entry picks up rule chips; a merge leaves the existing
            // item's chips as the household last left them.
            (_, _) => {
                let rule_terms =
                    item_rules::filter_terms_for(&mut *tx, name, AddedVia::Manual).await?;
                let item = repository::insert_item(
                    &mut tx,
                    name,
                    payload.quantity,
                    GroceryItemSource::Manual,
                    note.as_deref(),
                    &filter_terms::merge(&filter_terms, &rule_terms),
                    user_id,
                )
                .await?;
                (item, log::Added::NewEntry)
            }
        };
        tx.commit().await?;
        log::added(&item, how, user_id);
        Ok(item)
    }

    /// Applies an edit to an item on the list.
    ///
    /// Absent fields are left as they were. Renaming the item or changing its
    /// chips also drops the product chosen for it. A committed item is locked, so an
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
            Some(terms) => filter_terms::clean(terms),
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
        // A renamed or re-chipped item no longer matches the product chosen
        // for it; dropping the choice here lands with the edit.
        selections::forget_if_stale(&mut tx, &existing, &item).await?;
        tx.commit().await?;
        log::updated(&item);
        Ok(item)
    }

    /// Removes an item from the list. A committed item is locked, as for edits.
    pub async fn delete(&self, item_id: Uuid) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        let item = self.locked_editable_item(&mut tx, item_id).await?;
        repository::delete_item(&mut tx, item_id).await?;
        tx.commit().await?;
        log::removed(&item);
        Ok(())
    }

    /// Locks in the whole list for purchase: every active item becomes
    /// committed. Returns the items it changed, which is empty when there was
    /// nothing under review — pressing the button twice is harmless.
    pub async fn commit_list(&self) -> Result<Vec<GroceryItem>, ApiError> {
        self.move_all(GroceryItemStatus::Active, GroceryItemStatus::Committed)
            .await
    }

    /// Reopens a committed list for editing.
    pub async fn release_list(&self) -> Result<Vec<GroceryItem>, ApiError> {
        self.move_all(GroceryItemStatus::Committed, GroceryItemStatus::Active)
            .await
    }

    /// Moves every item in one status to another, and logs how many moved.
    async fn move_all(
        &self,
        from: GroceryItemStatus,
        to: GroceryItemStatus,
    ) -> Result<Vec<GroceryItem>, ApiError> {
        let items = repository::move_all(self.pool, from, to).await?;
        log::list_moved(from, to, &items);
        Ok(items)
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
            Some(item) => {
                log::locked(&item);
                Err(ApiError::Conflict(format!(
                    "{item_id} is committed for purchase — release the list to change it"
                )))
            }
            None => Err(ApiError::NotFound(item_id.to_string())),
        }
    }
}
