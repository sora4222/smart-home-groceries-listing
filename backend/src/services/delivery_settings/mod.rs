//! Settings › Delivery: each store's delivery fee rules and the order
//! planner's preferences.
//!
//! The app cannot read a store's delivery fees without the household's own
//! logged-in browser, so the household types them once here and the order
//! planner (`services/order_plan`) adds them to every option. A store that was
//! never saved has no rules yet (every value `None`); the planner says so
//! instead of guessing a fee.
//!
//! SQL lives in [`repository`].

mod log;
pub mod repository;

use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::delivery_rows::OrderMode;
use crate::services::stores::Store;

/// One store's delivery fee rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeRules {
    pub store: Store,
    /// What one delivery costs; `None` until the household sets it.
    pub delivery_fee: Option<Decimal>,
    /// An order this big or bigger delivers free; `None` = never free.
    pub free_delivery_over: Option<Decimal>,
    /// The store refuses an order smaller than this; `None` = no minimum.
    pub minimum_order: Option<Decimal>,
}

impl FeeRules {
    /// A store whose rules were never saved.
    pub fn unset(store: Store) -> Self {
        Self {
            store,
            delivery_fee: None,
            free_delivery_over: None,
            minimum_order: None,
        }
    }
}

/// Everything on Settings › Delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliverySettings {
    /// One entry per store, in the stores' display order.
    pub stores: Vec<FeeRules>,
    /// The planner's default mode.
    pub mode: OrderMode,
    /// Options whose delivery fees add up to more are not recommended.
    pub max_delivery_spend: Option<Decimal>,
}

impl DeliverySettings {
    /// `store`'s rules.
    pub fn rules_for(&self, store: Store) -> FeeRules {
        self.stores
            .iter()
            .copied()
            .find(|rules| rules.store == store)
            .unwrap_or_else(|| FeeRules::unset(store))
    }
}

/// The mode a household that never saved its preferences gets.
pub const DEFAULT_MODE: OrderMode = OrderMode::MinimiseTotal;

/// Reads and saves Settings › Delivery.
pub struct DeliverySettingsService<'a> {
    pool: &'a PgPool,
}

impl<'a> DeliverySettingsService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// The saved settings, with every store present.
    pub async fn read(&self) -> Result<DeliverySettings, ApiError> {
        let saved = repository::list_fees(self.pool).await?;
        let stores = Store::ALL
            .into_iter()
            .map(|store| {
                saved
                    .iter()
                    .find(|row| row.store == store)
                    .map(|row| FeeRules {
                        store,
                        delivery_fee: row.delivery_fee,
                        free_delivery_over: row.free_delivery_over,
                        minimum_order: row.minimum_order,
                    })
                    .unwrap_or_else(|| FeeRules::unset(store))
            })
            .collect();
        let preferences = repository::find_preferences(self.pool).await?;
        Ok(DeliverySettings {
            stores,
            mode: preferences.as_ref().map_or(DEFAULT_MODE, |p| p.mode),
            max_delivery_spend: preferences.and_then(|p| p.max_delivery_spend),
        })
    }

    /// Saves the stores' rules given (others keep theirs) and the
    /// preferences, together, and returns the settings as they now stand.
    pub async fn save(
        &self,
        stores: &[FeeRules],
        mode: OrderMode,
        max_delivery_spend: Option<Decimal>,
        user_id: &str,
    ) -> Result<DeliverySettings, ApiError> {
        let mut tx = self.pool.begin().await?;
        for rules in stores {
            repository::upsert_fees(&mut *tx, rules, user_id).await?;
        }
        repository::upsert_preferences(&mut *tx, mode, max_delivery_spend, user_id).await?;
        tx.commit().await?;
        let settings = self.read().await?;
        log::saved(&settings, user_id);
        Ok(settings)
    }
}
