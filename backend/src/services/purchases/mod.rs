//! Purchase history: what the household bought, and what it paid.
//!
//! A shop is saved as bought when the "Fill trolley" bookmarklet reports that
//! it filled the store's trolley ([`record_in_background`], [`record`]).
//! Payment happens on the store's website, which the app never sees, so a
//! shop saved by mistake can be undone ([`undo`]). The saved purchases feed
//! the product picker ([`history`]) and the spending analysis
//! (`services/spending/`).
//!
//! SQL lives in [`repository`] (writes) and [`read_repository`] (reads).

pub mod category;
mod fee;
pub mod history;
mod log;
mod price;
pub mod read_repository;
mod record;
pub mod repository;
mod undo;

#[cfg(test)]
pub(crate) mod fixtures;

pub use history::ProductHistory;
pub use undo::Undone;

use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::PurchaseOrder;
use crate::services::stores::{Store, StoreClients};

/// How many recent shops the order list shows.
pub const RECENT_ORDERS: i64 = 20;

/// A saved shop and how many products it holds.
#[derive(Debug, Clone)]
pub struct OrderSummary {
    pub order: PurchaseOrder,
    pub products: i64,
}

/// Reads and changes the purchase history.
pub struct PurchaseService<'a> {
    pool: &'a PgPool,
}

impl<'a> PurchaseService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// The newest shops, newest first.
    pub async fn recent_orders(&self) -> Result<Vec<OrderSummary>, ApiError> {
        let orders = repository::recent_orders(self.pool, RECENT_ORDERS).await?;
        let ids: Vec<Uuid> = orders.iter().map(|order| order.id).collect();
        let counts: HashMap<Uuid, i64> = repository::line_counts(self.pool, &ids)
            .await?
            .into_iter()
            .collect();
        Ok(orders
            .into_iter()
            .map(|order| OrderSummary {
                products: counts.get(&order.id).copied().unwrap_or(0),
                order,
            })
            .collect())
    }

    /// Forgets a shop saved by mistake and puts its items back on the list.
    pub async fn undo(&self, order_id: Uuid, user_id: &str) -> Result<Undone, ApiError> {
        undo::undo(self.pool, order_id, user_id).await
    }

    /// Every purchase of the given products, grouped by product.
    pub async fn product_history(
        &self,
        products: &[(Store, String)],
    ) -> Result<Vec<ProductHistory>, ApiError> {
        if products.is_empty() {
            return Ok(Vec::new());
        }
        let purchases = read_repository::of_products(self.pool, products).await?;
        Ok(history::group(purchases))
    }

    /// Works out every purchase's category again with today's keywords.
    /// Returns how many changed.
    pub async fn recategorise(&self) -> Result<usize, ApiError> {
        let mut tx = self.pool.begin().await?;
        let inputs = repository::category_inputs(&mut *tx).await?;
        let mut changed = 0;
        for (id, store_category, product_name) in &inputs {
            let found = category::categorise(store_category.as_deref(), product_name);
            if repository::set_category(&mut *tx, *id, found.category, found.source).await? {
                changed += 1;
            }
        }
        tx.commit().await?;
        log::recategorised(inputs.len(), changed);
        Ok(changed)
    }
}

/// Saves the products a reported handoff added, as one order. `None` when
/// there was nothing to save or it was saved already.
pub async fn record(
    pool: &PgPool,
    stores: &StoreClients,
    handoff_id: Uuid,
) -> Result<Option<PurchaseOrder>, ApiError> {
    record::trolley_fill(pool, stores, handoff_id).await
}

/// [`record`] in a spawned task, so the bookmarklet's report is answered
/// without waiting for the stores to price each product.
pub fn record_in_background(pool: PgPool, stores: StoreClients, handoff_id: Uuid) {
    tokio::spawn(async move {
        if let Err(err) = record(&pool, &stores, handoff_id).await {
            log::save_failed(handoff_id, &err);
        }
    });
}
