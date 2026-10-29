//! Choosing products for each list item.
//!
//! A household member opens an item's Select item page and picks a product.
//! The choice is checked against the store's own answer for the item (the
//! search is re-run, usually from the 10-minute cache) and saved with the
//! store's details, so the order screen knows what to buy for every item.
//!
//! An item wanting one has one product per store: choosing again there
//! replaces it. An item wanting more may have up to that many products at a
//! store, with a count each ([`room`], [`allocation`]). The store chosen at
//! last is the one the order buys from; the other store's products are kept,
//! so the order planner can switch the item between stores
//! ([`SelectionService::buy_at`]). Renaming an item or changing its chips
//! drops its choices ([`forget_if_stale`]); removing the item removes them.
//! Ordered items cannot be re-chosen.
//!
//! SQL lives in [`repository`]; finding the product in a search in [`offer`];
//! grouping each item's choices in [`grouping`].

pub mod allocation;
mod grouping;
mod log;
mod offer;
pub mod parts;
pub mod repository;
mod room;
mod staleness;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemStatus, ItemSelection};
use crate::services::grocery::repository as items;
use crate::services::product_search::ProductSearchService;
use crate::services::stores::{Store, StoreClients};
pub use grouping::{Counted, ItemChoices};
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

    /// Every item's choices: the products the order buys, with how many of
    /// each, and the item's choices at the other store. Oldest choice first.
    pub async fn list(&self) -> Result<Vec<ItemChoices>, ApiError> {
        let quantities = items::list_items(self.pool)
            .await?
            .into_iter()
            .map(|item| (item.id, item.quantity))
            .collect();
        let rows = repository::list_every(self.pool).await?;
        Ok(grouping::by_item(rows, &quantities))
    }

    /// Chooses `product_id` at `store` for the item and makes that store's
    /// products the ones the order buys. For an item wanting one it replaces
    /// the store's earlier choice; otherwise it is added beside them.
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
                "{} changed while its product was being chosen. Open Select item again.",
                item.name
            )));
        }

        let product = &priced.product;
        let quantity = room::make_room(&mut tx, &item, store, &product.product_id).await?;
        let unit = product.unit_price.as_ref();
        let selection = repository::upsert(
            &mut tx,
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
                image_url: product.image_url.as_deref(),
                selected_by: user_id,
                quantity,
            },
        )
        .await?;
        tx.commit().await?;
        log::chosen(&selection);
        Ok(selection)
    }

    /// Clears one product at `store` (`product_id`), every product at
    /// `store`, or every choice when `store` is `None`. Clearing the store
    /// the order buys from hands that role to the item's other store.
    /// Clearing nothing is harmless; an unknown item is a 404.
    pub async fn clear(
        &self,
        item_id: Uuid,
        store: Option<Store>,
        product_id: Option<&str>,
    ) -> Result<(), ApiError> {
        if items::find_item(self.pool, item_id).await?.is_none() {
            return Err(ApiError::NotFound("Grocery item not found".into()));
        }
        let had_choice = match store {
            None => repository::delete_for_item(self.pool, item_id).await?,
            Some(store) => {
                let mut tx = self.pool.begin().await?;
                let had = match product_id {
                    Some(product_id) => {
                        repository::delete_product(&mut tx, item_id, store, product_id).await?
                    }
                    None => repository::delete_at_store(&mut tx, item_id, store).await?,
                };
                tx.commit().await?;
                had
            }
        };
        log::cleared(item_id, had_choice);
        Ok(())
    }

    /// Sets how many of one chosen product to buy, when the item has several
    /// products at `store`; the others change so the counts still add up to
    /// the item's quantity. 422 when there is only one product there, or the
    /// count leaves no room for the others.
    pub async fn set_quantity(
        &self,
        item_id: Uuid,
        store: Store,
        product_id: &str,
        wanted: i32,
    ) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        let item = lock_choosable_item(&mut tx, item_id).await?;
        let at_store = repository::lock_at_store(&mut tx, item.id, store).await?;
        let index = at_store
            .iter()
            .position(|row| row.product_id == product_id)
            .ok_or_else(|| ApiError::NotFound("That product is not chosen".into()))?;
        let stored: Vec<_> = at_store.iter().map(|row| row.quantity).collect();
        let counts = allocation::allocate(&stored, item.quantity);
        let next = (at_store.len() > 1)
            .then(|| allocation::rebalance(&counts, index, wanted))
            .flatten()
            .ok_or_else(|| {
                ApiError::UnprocessableEntity(format!(
                    "{} of {} cannot be split that way.",
                    item.quantity, item.name
                ))
            })?;
        for (row, count) in at_store.iter().zip(next) {
            repository::set_quantity(&mut tx, row.id, count).await?;
        }
        tx.commit().await?;
        log::quantity_set(item_id, product_id, wanted);
        Ok(())
    }

    /// Makes each item's choice at the paired store the one the order buys,
    /// all together. Refused with 422, changing nothing, when an item has no
    /// choice at its store.
    pub async fn buy_at(&self, picks: &[(Uuid, Store)]) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        for &(item_id, store) in picks {
            if !repository::buy_at_store(&mut tx, item_id, store).await? {
                tx.rollback().await?;
                return Err(ApiError::UnprocessableEntity(format!(
                    "That item has no product chosen at {}. Choose one on Select item first.",
                    store.display_name()
                )));
            }
        }
        tx.commit().await?;
        log::bought_at(picks);
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
