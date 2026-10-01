//! Choosing one product for each list item.
//!
//! A household member opens an item's price comparison and picks a product.
//! The choice is checked against the store's own answer for the item (the
//! search is re-run, usually from the 10-minute cache) and saved with the
//! store's details, so the order screen knows what to buy for every item.
//!
//! One product per item: choosing again replaces the choice. Renaming an item
//! or changing its chips drops it ([`forget_if_stale`]); removing the item
//! removes it. Ordered items are history and cannot be re-chosen.
//!
//! SQL lives in [`repository`]; finding the product in a search in [`offer`].

mod log;
mod offer;
pub mod repository;
mod staleness;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemStatus, ItemSelection};
use crate::services::grocery::repository as items;
use crate::services::product_search::ProductSearchService;
use crate::services::stores::{Store, StoreClients};
use repository::NewSelection;

/// Reads and writes item choices.
pub struct SelectionService<'a> {
    pool: &'a PgPool,
    stores: &'a StoreClients,
}

impl<'a> SelectionService<'a> {
    /// Borrows the pool and the store clients for one request.
    pub fn new(pool: &'a PgPool, stores: &'a StoreClients) -> Self {
        Self { pool, stores }
    }

    /// Every saved choice, oldest first.
    pub async fn list(&self) -> Result<Vec<ItemSelection>, ApiError> {
        repository::list_all(self.pool).await
    }

    /// Chooses `product_id` at `store` for the item, replacing any earlier
    /// choice.
    ///
    /// The stores are searched first, outside any transaction, so a slow
    /// store never holds a row lock. The item is then locked and must still
    /// be the item that was searched for; if a tab renamed it meanwhile the
    /// choice is refused with 409 rather than saved against the new name.
    pub async fn choose(
        &self,
        item_id: Uuid,
        store: Store,
        product_id: &str,
        user_id: &str,
    ) -> Result<ItemSelection, ApiError> {
        let search = ProductSearchService::new(self.pool, self.stores)
            .search_item(item_id)
            .await?;
        let searched = search.item.clone();
        let priced = offer::find(search, store, product_id).inspect_err(|err| {
            log::refused(item_id, store.as_str(), product_id, &err.to_string());
        })?;

        let mut tx = self.pool.begin().await?;
        let item = lock_choosable_item(&mut tx, item_id).await?;
        if !staleness::choice_still_fits(&searched, &item) {
            tx.rollback().await?;
            log::item_changed_while_choosing(item_id);
            return Err(ApiError::Conflict(format!(
                "{} changed while its product was being chosen — compare prices again.",
                item.name
            )));
        }

        let product = &priced.product;
        let unit = product.unit_price.as_ref();
        let selection = repository::upsert(
            &mut *tx,
            &NewSelection {
                grocery_item_id: item.id,
                store,
                product_id: &product.product_id,
                product_name: &product.name,
                brand: product.brand.as_deref(),
                package_size: product.package_size.as_deref(),
                price: product.price,
                unit_price: unit.map(|u| u.amount),
                unit_price_per: unit.map(|u| u.per),
                total_price: priced.total_price,
                // The quantity the total was worked out for, which a
                // concurrent quantity edit may since have changed; the order
                // screen re-prices either way.
                priced_quantity: searched.quantity,
                url: &product.url,
                selected_by: user_id,
            },
        )
        .await?;
        tx.commit().await?;
        log::chosen(&selection);
        Ok(selection)
    }

    /// Clears the item's choice. Clearing an item with no choice is harmless;
    /// an unknown item is a 404.
    pub async fn clear(&self, item_id: Uuid) -> Result<(), ApiError> {
        if items::find_item(self.pool, item_id).await?.is_none() {
            return Err(ApiError::NotFound("Grocery item not found".into()));
        }
        let had_choice = repository::delete_for_item(self.pool, item_id).await?;
        log::cleared(item_id, had_choice);
        Ok(())
    }
}

/// Drops the choice for an item an edit has renamed or re-chipped.
///
/// Called by the grocery list inside the edit's own transaction, so the edit
/// and the forgetting land together.
pub async fn forget_if_stale(
    tx: &mut Transaction<'_, Postgres>,
    before: &GroceryItem,
    after: &GroceryItem,
) -> Result<(), ApiError> {
    if staleness::choice_still_fits(before, after) {
        return Ok(());
    }
    if repository::delete_for_item(&mut **tx, after.id).await? {
        log::forgotten_after_edit(after.id, &after.name);
    }
    Ok(())
}

/// Locks the item and refuses one that is no longer on the list.
async fn lock_choosable_item(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
) -> Result<GroceryItem, ApiError> {
    match items::lock_item(tx, item_id).await? {
        Some(item)
            if matches!(
                item.status,
                GroceryItemStatus::Active | GroceryItemStatus::Committed
            ) =>
        {
            Ok(item)
        }
        Some(item) => Err(ApiError::Conflict(format!(
            "{} is no longer on the list, so its product cannot be changed.",
            item.name
        ))),
        None => Err(ApiError::NotFound("Grocery item not found".into())),
    }
}
