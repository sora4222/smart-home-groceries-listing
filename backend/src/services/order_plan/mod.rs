//! The order planner: the ways to buy the committed list, with delivery,
//! ranked the household's way.
//!
//! Every committed item's chosen products (one per store at most) are
//! re-priced today at their stores, the same way the order review does. The
//! planner then costs four ways to buy them — all at Woolworths, all at
//! Coles, the best mix, and the stores the order buys from now — adding each
//! store's delivery fee from Settings › Delivery, and ranks them by the mode
//! (`docs/features/FEATURE_ORDER_OPTIMISATION.md`).
//!
//! The planner never picks a product: it only moves an item between products
//! the household chose. Using an option is `PUT /api/order-stores`
//! (`SelectionService::buy_at`).
//!
//! Delivery charge in [`fees`]; one option in [`option`]; the best mix in
//! [`search`]; ranking in [`rank`]; putting it together in [`build`].

mod build;
mod fees;
mod log;
mod option;
mod rank;
mod search;

pub use fees::DeliveryCharge;
pub use option::{Missing, OptionKind, PlanOption, StorePart};

use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::db::{GroceryItem, GroceryItemStatus, ItemSelection};
use crate::models::delivery_rows::OrderMode;
use crate::services::delivery_settings::{DeliverySettings, DeliverySettingsService};
use crate::services::grocery::repository as items;
use crate::services::order_review::reprice;
use crate::services::product_search::ProductSearchService;
use crate::services::selections::repository as selections;
use crate::services::stores::StoreClients;
use option::Candidate;

/// The order's options, best first.
#[derive(Debug, Clone)]
pub struct OrderPlan {
    pub mode: OrderMode,
    pub options: Vec<PlanOption>,
    /// The index of the option to use, when the best one can be placed as it
    /// is (every item bought, store minimums met, within the delivery cap).
    pub recommended: Option<usize>,
    /// The index of the option the order buys from now, if it is listed.
    pub current: Option<usize>,
    /// Committed items with no product chosen at any store.
    pub unchosen: Vec<GroceryItem>,
    /// The best mix was proven best (every combination tried).
    pub exact: bool,
}

/// Plans the order.
pub struct OrderPlanService<'a> {
    pool: &'a PgPool,
    stores: &'a StoreClients,
}

impl<'a> OrderPlanService<'a> {
    /// Borrows the pool and the store clients for one request.
    pub fn new(pool: &'a PgPool, stores: &'a StoreClients) -> Self {
        Self { pool, stores }
    }

    /// The committed list's options under `mode`, or the saved mode.
    ///
    /// Items are re-priced one store at a time, as the order review does;
    /// most answers come from the 10-minute search cache.
    pub async fn plan(
        &self,
        mode: Option<OrderMode>,
    ) -> Result<(OrderPlan, DeliverySettings), ApiError> {
        let settings = DeliverySettingsService::new(self.pool).read().await?;
        let mode = mode.unwrap_or(settings.mode);
        let committed = items::list_items(self.pool)
            .await?
            .into_iter()
            .filter(|item| item.status == GroceryItemStatus::Committed);
        let mut choices: HashMap<Uuid, Vec<ItemSelection>> = HashMap::new();
        for choice in selections::list_every(self.pool).await? {
            choices
                .entry(choice.grocery_item_id)
                .or_default()
                .push(choice);
        }
        let search = ProductSearchService::new(self.pool, self.stores);

        let mut candidates = Vec::new();
        let mut unchosen = Vec::new();
        for item in committed {
            let Some(item_choices) = choices.remove(&item.id) else {
                unchosen.push(item);
                continue;
            };
            let Some(current) = item_choices.iter().find(|c| c.for_order).map(|c| c.store) else {
                unchosen.push(item);
                continue;
            };
            let mut offers = Vec::new();
            for choice in item_choices {
                let outcome = search.search_at(&item, choice.store).await;
                offers.push(reprice(item.clone(), choice, outcome));
            }
            candidates.push(Candidate {
                item,
                current,
                offers,
            });
        }

        let plan = build::plan(mode, &candidates, unchosen, &settings);
        log::planned(&plan);
        Ok((plan, settings))
    }
}

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_modes;
