//! The spending analysis: where the household's grocery money went.
//!
//! Reads saved purchases (`services/purchases/`), narrows them with the
//! page's filters, and builds every view at once ([`breakdown`]); the page
//! switches views without asking again. Days are the household's own
//! calendar days in the browser's time zone ([`range`]).

pub mod breakdown;
pub mod period;
pub mod range;

pub use breakdown::Breakdown;
pub use period::Period;

use chrono::NaiveDate;
use chrono_tz::Tz;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::db::{Purchase, PurchaseOrder};
use crate::services::grocery::repository::normalise;
use crate::services::purchases::read_repository::{self, PurchaseFilter};
use crate::services::purchases::repository as orders;
use crate::services::stores::Store;

/// The page's filters. Every one is optional except the grouping and zone.
#[derive(Debug, Clone)]
pub struct SpendingQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub store: Option<Store>,
    /// Part of an item's name, any case.
    pub item: Option<String>,
    pub category: Option<String>,
    pub period: Period,
    pub tz: Tz,
}

/// Everything the Analysis page shows.
#[derive(Debug, Clone)]
pub struct SpendingReport {
    /// The newest shop, whatever the filters say.
    pub latest_order: Option<PurchaseOrder>,
    /// Every category any purchase is in, for the category filter.
    pub categories: Vec<String>,
    pub breakdown: Breakdown,
}

/// Builds the spending analysis.
pub struct SpendingService<'a> {
    pool: &'a PgPool,
}

impl<'a> SpendingService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Every view of the purchases the filters let through.
    pub async fn analyse(&self, query: &SpendingQuery) -> Result<SpendingReport, ApiError> {
        let (from, until) = range::bounds(query.tz, query.from, query.to);
        let item_part = query
            .item
            .as_deref()
            .map(normalise)
            .filter(|s| !s.is_empty());
        let filter = PurchaseFilter {
            from,
            until,
            store: query.store,
            item_key_part: item_part.as_deref(),
            category: query.category.as_deref(),
        };
        let purchases = read_repository::matching(self.pool, &filter).await?;
        let stores: Vec<Store> = match query.store {
            Some(store) => vec![store],
            None => Store::ALL.to_vec(),
        };
        Ok(SpendingReport {
            latest_order: orders::recent_orders(self.pool, 1)
                .await?
                .into_iter()
                .next(),
            categories: read_repository::categories(self.pool).await?,
            breakdown: breakdown::build(&purchases, query.period, query.tz, &stores),
        })
    }

    /// What one item cost each time it was bought, at any store, oldest first.
    pub async fn item_prices(&self, item_name: &str) -> Result<Vec<Purchase>, ApiError> {
        read_repository::of_item(self.pool, &normalise(item_name)).await
    }
}
