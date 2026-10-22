//! Row types for Settings › Delivery: each store's delivery fee rules and the
//! order planner's preferences.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::services::stores::Store;

/// One store's delivery fee rules, as the household typed them.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct StoreDeliveryFees {
    pub store: Store,
    /// What one delivery costs; `None` until the household sets it.
    pub delivery_fee: Option<Decimal>,
    /// An order this big or bigger delivers free; `None` = never free.
    pub free_delivery_over: Option<Decimal>,
    /// The store refuses an order smaller than this; `None` = no minimum.
    pub minimum_order: Option<Decimal>,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
}

/// How the order planner ranks the ways to split an order between stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderMode {
    /// Lowest items + delivery, split between stores when that is cheaper.
    MinimiseTotal,
    /// Lowest delivery fees first, then lowest items.
    MinimiseDelivery,
    /// Everything from Woolworths.
    WoolworthsOnly,
    /// Everything from Coles.
    ColesOnly,
    /// The household picks each item's store; the planner only adds it up.
    Manual,
}

/// The planner's saved preferences.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct OrderPreferences {
    pub mode: OrderMode,
    /// Options whose delivery fees add up to more are not recommended.
    pub max_delivery_spend: Option<Decimal>,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
}
