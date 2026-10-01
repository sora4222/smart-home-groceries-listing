//! Finding a list item's products at every store.
//!
//! For one grocery item: ask each store for the item's name and chips at
//! the same time, keep the products that match every chip, price them at
//! the item's quantity (deals applied), flag unit prices that cannot be
//! compared at face value, and order them cheapest first.
//!
//! A store that fails does not fail the search — its entry carries the
//! error and the other store's products still show.

mod comparability;
mod filters;
mod log;
mod ordering;
mod outcome;
mod priced;
mod query;

pub use comparability::{NO_UNIT_PRICE, UNITS_DIFFER};
pub use outcome::{ItemSearch, StoreOutcome};
pub use priced::PricedProduct;

use futures_util::future::join_all;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::GroceryItem;
use crate::services::grocery::repository;
use crate::services::stores::{Product, StoreClient, StoreClients, StoreError};

/// Searches the stores for list items.
pub struct ProductSearchService<'a> {
    pool: &'a PgPool,
    stores: &'a StoreClients,
}

impl<'a> ProductSearchService<'a> {
    /// Borrows the pool and the store clients for one request.
    pub fn new(pool: &'a PgPool, stores: &'a StoreClients) -> Self {
        Self { pool, stores }
    }

    /// Searches every store for the item with `item_id`. 404 if it is not
    /// on the list.
    pub async fn search_item(&self, item_id: Uuid) -> Result<ItemSearch, ApiError> {
        let item = repository::find_item(self.pool, item_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Grocery item not found".into()))?;
        let query = query::for_item(&item.name, &item.filter_terms);

        let results = join_all(self.stores.iter().map(|client| client.search(&query))).await;
        let mut stores: Vec<StoreOutcome> = self
            .stores
            .iter()
            .zip(results)
            .map(|(client, result)| outcome_for(&item, &query, client.as_ref(), result))
            .collect();

        // One basis for every store, so Woolworths and Coles are compared
        // against each other, not just within themselves.
        let all: Vec<PricedProduct> = stores
            .iter()
            .flat_map(|s| s.products.iter().cloned())
            .collect();
        let common = comparability::common_basis(&all);
        for store in &mut stores {
            comparability::annotate(&mut store.products, common);
            ordering::sort(&mut store.products, common);
        }

        Ok(ItemSearch {
            item,
            query,
            stores,
        })
    }
}

/// Filters and prices one store's answer, or records its failure.
fn outcome_for(
    item: &GroceryItem,
    query: &str,
    client: &dyn StoreClient,
    result: Result<Vec<Product>, StoreError>,
) -> StoreOutcome {
    let store = client.store();
    match result {
        Ok(found) => {
            let found_count = found.len();
            let quantity = u32::try_from(item.quantity).unwrap_or(1).max(1);
            let products: Vec<PricedProduct> = filters::apply(found, &item.filter_terms)
                .into_iter()
                .map(|product| PricedProduct::new(product, quantity))
                .collect();
            log::store_answered(item.id, store, query, found_count, products.len());
            StoreOutcome {
                store,
                products,
                error: None,
            }
        }
        Err(err) => {
            log::store_failed(item.id, store, query, &err);
            StoreOutcome {
                store,
                products: Vec::new(),
                error: Some(err),
            }
        }
    }
}
