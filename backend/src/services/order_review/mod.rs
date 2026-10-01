//! Reviewing the order before anything is bought.
//!
//! The order is every committed list item. Each item with a chosen product is
//! re-priced at its chosen store, at the item's current quantity; items with
//! no product are listed so the household can choose one. Nothing here picks
//! a product or a store for the household — it only reports what was chosen
//! and what it costs now.
//!
//! Re-pricing rules are in [`line`]; grouping and totals in [`summary`].

mod line;
mod log;
mod summary;

pub use line::{LineStatus, OrderLine, PriceChange};
pub use summary::{OrderReview, StoreOrder};

use std::collections::HashMap;

use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::db::GroceryItemStatus;
use crate::services::grocery::repository as items;
use crate::services::product_search::ProductSearchService;
use crate::services::selections::repository as selections;
use crate::services::stores::StoreClients;

/// Builds the order review.
pub struct OrderReviewService<'a> {
    pool: &'a PgPool,
    stores: &'a StoreClients,
}

impl<'a> OrderReviewService<'a> {
    /// Borrows the pool and the store clients for one request.
    pub fn new(pool: &'a PgPool, stores: &'a StoreClients) -> Self {
        Self { pool, stores }
    }

    /// The committed items, each chosen product priced as of now.
    ///
    /// Items are re-priced one after another, and each only at its chosen
    /// store: the stores see a household's pace, not a burst. Most answers
    /// come from the 10-minute search cache the price comparison filled.
    pub async fn review(&self) -> Result<OrderReview, ApiError> {
        let committed = items::list_items(self.pool)
            .await?
            .into_iter()
            .filter(|item| item.status == GroceryItemStatus::Committed);
        let mut choices: HashMap<_, _> = selections::list_all(self.pool)
            .await?
            .into_iter()
            .map(|choice| (choice.grocery_item_id, choice))
            .collect();
        let search = ProductSearchService::new(self.pool, self.stores);

        let mut lines = Vec::new();
        let mut unchosen = Vec::new();
        for item in committed {
            let Some(choice) = choices.remove(&item.id) else {
                unchosen.push(item);
                continue;
            };
            let outcome = search.search_at(&item, choice.store).await;
            let line = line::reprice(item, choice, outcome);
            log_line(&line);
            lines.push(line);
        }

        let review = summary::build(lines, unchosen);
        log::reviewed(&review);
        Ok(review)
    }
}

/// Logs a line worth a second look.
fn log_line(line: &OrderLine) {
    if line.problem.is_some() {
        log::line_problem(line);
    }
    if matches!(line.price_change, Some(PriceChange::Up | PriceChange::Down)) {
        log::price_changed(line);
    }
}
