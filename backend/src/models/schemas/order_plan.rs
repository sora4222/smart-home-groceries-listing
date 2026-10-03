//! The body of `GET /api/order-options`: the ways to buy the committed list,
//! with delivery, best first.
//!
//! Money is a decimal string. Lines reuse the order review's shape.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::order_review::{OrderLineResponse, UnchosenItemResponse};
use super::selections::OrderStorePick;
use crate::models::delivery_rows::OrderMode;
use crate::services::order_plan::{Missing, OptionKind, OrderPlan, PlanOption, StorePart};
use crate::services::stores::Store;

/// `GET /api/order-options?mode=`: a mode to rank by instead of the saved one.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct OrderOptionsQuery {
    pub mode: Option<OrderMode>,
}

/// The order's options.
#[derive(Debug, Serialize)]
pub struct OrderOptionsResponse {
    /// The mode the options are ranked by.
    pub mode: OrderMode,
    /// Best first.
    pub options: Vec<OrderOptionResponse>,
    /// Committed items with no product chosen at any store.
    pub unchosen: Vec<UnchosenItemResponse>,
    /// The best mix was proven best (every combination tried).
    pub exact: bool,
    /// The household's cap on delivery fees per order, if any.
    pub max_delivery_spend: Option<Decimal>,
}

/// One way to buy the order.
#[derive(Debug, Serialize)]
pub struct OrderOptionResponse {
    /// `split`, `woolworths`, `coles` or `current`.
    pub kind: &'static str,
    pub label: &'static str,
    /// Use this one: the best, and it can be placed as it is.
    pub recommended: bool,
    /// The order buys from these stores now.
    pub is_current: bool,
    pub stores: Vec<OptionStoreResponse>,
    pub missing: Vec<MissingItemResponse>,
    pub items_total: Decimal,
    pub delivery_total: Decimal,
    pub total: Decimal,
    pub complete: bool,
    /// Every store used has a delivery fee set in Settings › Delivery.
    pub fees_known: bool,
    pub meets_minimums: bool,
    pub within_delivery_cap: bool,
    /// What `PUT /api/order-stores` takes to use this option.
    pub picks: Vec<OrderStorePick>,
}

/// What an option buys at one store.
#[derive(Debug, Serialize)]
pub struct OptionStoreResponse {
    pub store: Store,
    pub store_name: &'static str,
    pub lines: Vec<OrderLineResponse>,
    pub subtotal: Decimal,
    pub delivery_fee: Decimal,
    pub fee_known: bool,
    pub free_delivery: bool,
    pub below_minimum: bool,
    pub minimum_order: Option<Decimal>,
}

/// An item an option cannot buy.
#[derive(Debug, Serialize)]
pub struct MissingItemResponse {
    pub grocery_item_id: Uuid,
    pub name: String,
    pub reason: String,
}

impl OrderOptionsResponse {
    /// The plan as the web app reads it.
    pub fn new(plan: OrderPlan, max_delivery_spend: Option<Decimal>) -> Self {
        let OrderPlan {
            mode,
            options,
            recommended,
            current,
            unchosen,
            exact,
        } = plan;
        Self {
            mode,
            options: options
                .into_iter()
                .enumerate()
                .map(|(i, option)| {
                    option_response(option, recommended == Some(i), current == Some(i))
                })
                .collect(),
            unchosen: unchosen.into_iter().map(Into::into).collect(),
            exact,
            max_delivery_spend,
        }
    }
}

fn option_response(option: PlanOption, recommended: bool, is_current: bool) -> OrderOptionResponse {
    let (kind, label) = match option.kind {
        OptionKind::Split => ("split", "Split between stores"),
        OptionKind::Only(Store::Woolworths) => ("woolworths", "All at Woolworths"),
        OptionKind::Only(Store::Coles) => ("coles", "All at Coles"),
        OptionKind::Current => ("current", "Your current stores"),
    };
    OrderOptionResponse {
        kind,
        label,
        recommended,
        is_current,
        complete: option.complete(),
        fees_known: option.fees_known(),
        meets_minimums: option.meets_minimums(),
        within_delivery_cap: option.within_cap,
        picks: option
            .picks()
            .into_iter()
            .map(|(grocery_item_id, store)| OrderStorePick {
                grocery_item_id,
                store,
            })
            .collect(),
        items_total: option.items_total,
        delivery_total: option.delivery_total,
        total: option.total,
        stores: option.parts.into_iter().map(store_response).collect(),
        missing: option.missing.into_iter().map(missing_response).collect(),
    }
}

fn store_response(part: StorePart) -> OptionStoreResponse {
    OptionStoreResponse {
        store: part.store,
        store_name: part.store.display_name(),
        lines: part.lines.into_iter().map(Into::into).collect(),
        subtotal: part.subtotal,
        delivery_fee: part.charge.fee,
        fee_known: part.charge.fee_known,
        free_delivery: part.charge.free,
        below_minimum: part.charge.below_minimum,
        minimum_order: part.minimum_order,
    }
}

fn missing_response(missing: Missing) -> MissingItemResponse {
    MissingItemResponse {
        grocery_item_id: missing.item.id,
        name: missing.item.name,
        reason: missing.reason,
    }
}
