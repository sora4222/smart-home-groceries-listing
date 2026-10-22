//! Hand-made planner inputs for this module's tests.

use rust_decimal::Decimal;

use super::option::Candidate;
use crate::models::delivery_rows::OrderMode;
use crate::services::delivery_settings::{DeliverySettings, FeeRules};
use crate::services::order_review::fixtures::{choice, item};
use crate::services::order_review::{LineStatus, OrderLine, PriceChange};
use crate::services::stores::Store::{self, Coles, Woolworths};

/// An item bought now at `current`, priced at each `(store, total)`.
pub fn candidate(name: &str, current: Store, prices: &[(Store, Decimal)]) -> Candidate {
    let it = item(name, 1);
    let offers = prices
        .iter()
        .map(|&(store, total)| OrderLine {
            item: it.clone(),
            choice: choice(&it, store, &format!("{name}-{store}"), total),
            status: LineStatus::Priced,
            price: Some(total),
            total_price: Some(total),
            deal_applied: false,
            price_change: Some(PriceChange::Same),
            problem: None,
        })
        .collect();
    Candidate {
        item: it,
        current,
        offers,
    }
}

pub fn settings(
    fee: Decimal,
    free_over: Option<Decimal>,
    minimum: Option<Decimal>,
) -> DeliverySettings {
    let rules = |store| FeeRules {
        store,
        delivery_fee: Some(fee),
        free_delivery_over: free_over,
        minimum_order: minimum,
    };
    DeliverySettings {
        stores: vec![rules(Woolworths), rules(Coles)],
        mode: OrderMode::MinimiseTotal,
        max_delivery_spend: None,
    }
}
