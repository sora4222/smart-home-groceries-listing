//! Re-pricing outcomes, one test per way a chosen product can turn out.

use super::fixtures::{choice, item};
use super::*;
use crate::services::product_search::PricedProduct;
use crate::services::stores::product::fixtures::product;
use crate::services::stores::{Deal, Product, Store, StoreError};
use rust_decimal_macros::dec;

fn coles(products: Vec<Product>, quantity: u32) -> Option<StoreOutcome> {
    Some(StoreOutcome {
        store: Store::Coles,
        products: products
            .into_iter()
            .map(|p| PricedProduct::new(p, quantity))
            .collect(),
        error: None,
    })
}

#[test]
fn prices_the_items_current_quantity() {
    let milk = item("milk", 3);
    let chosen = choice(&milk, Store::Coles, "milk", dec!(4.95));

    let line = reprice(
        milk,
        chosen,
        coles(vec![product(Store::Coles, "Milk", dec!(4.95))], 3),
    );

    assert_eq!(line.status, LineStatus::Priced);
    assert_eq!(line.price, Some(dec!(4.95)));
    assert_eq!(line.total_price, Some(dec!(14.85)));
    assert_eq!(line.counted_total(), Some(dec!(14.85)));
    assert_eq!(line.price_change, Some(PriceChange::Same));
    assert_eq!(line.problem, None);
}

#[test]
fn a_dearer_or_cheaper_shelf_price_is_flagged() {
    let milk = item("milk", 1);
    let up = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "milk", dec!(4.50)),
        coles(vec![product(Store::Coles, "Milk", dec!(4.95))], 1),
    );
    let down = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "milk", dec!(5.20)),
        coles(vec![product(Store::Coles, "Milk", dec!(4.95))], 1),
    );
    assert_eq!(up.price_change, Some(PriceChange::Up));
    assert_eq!(down.price_change, Some(PriceChange::Down));
}

#[test]
fn a_deal_reached_at_the_new_quantity_is_applied() {
    let cola = item("cola", 2);
    let mut on_deal = product(Store::Coles, "Cola", dec!(4.50));
    on_deal.deals.push(Deal {
        description: "2 for $5.80".into(),
        min_quantity: 2,
        unit_price: dec!(2.90),
    });

    let line = reprice(
        cola.clone(),
        choice(&cola, Store::Coles, "cola", dec!(4.50)),
        coles(vec![on_deal], 2),
    );

    assert_eq!(line.total_price, Some(dec!(5.80)));
    assert!(line.deal_applied);
}

#[test]
fn a_product_missing_from_the_results_is_not_offered() {
    let milk = item("milk", 1);
    let line = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "gone", dec!(1)),
        coles(Vec::new(), 1),
    );

    assert_eq!(line.status, LineStatus::NotOffered);
    assert_eq!(line.counted_total(), None);
    assert!(line.problem.unwrap().contains("no longer offers"));
}

#[test]
fn an_unavailable_product_is_not_counted() {
    let milk = item("milk", 1);
    let mut gone = product(Store::Coles, "Milk", dec!(4.95));
    gone.available = false;

    let line = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "milk", dec!(4.95)),
        coles(vec![gone], 1),
    );

    assert_eq!(line.status, LineStatus::Unavailable);
    assert_eq!(line.total_price, None);
    assert!(line.problem.unwrap().contains("Unavailable at Coles"));
}

#[test]
fn a_failed_store_keeps_the_choice_but_has_no_price() {
    let milk = item("milk", 1);
    let failed = Some(StoreOutcome {
        store: Store::Coles,
        products: Vec::new(),
        error: Some(StoreError::Blocked { status: 403 }),
    });

    let line = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "milk", dec!(4.95)),
        failed,
    );

    assert_eq!(line.status, LineStatus::StoreFailed);
    assert_eq!(line.choice.product_id, "milk");
    assert!(line.problem.unwrap().contains("Coles refused"));
}

#[test]
fn a_store_that_is_not_set_up_is_a_failed_store() {
    let milk = item("milk", 1);
    let line = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "milk", dec!(4.95)),
        None,
    );
    assert_eq!(line.status, LineStatus::StoreFailed);
}

#[test]
fn a_product_without_a_shelf_price_says_so() {
    let milk = item("milk", 1);
    let mut unpriced = product(Store::Coles, "Milk", dec!(1));
    unpriced.price = None;

    let line = reprice(
        milk.clone(),
        choice(&milk, Store::Coles, "milk", dec!(4.95)),
        coles(vec![unpriced], 1),
    );

    assert_eq!(line.status, LineStatus::Priced);
    assert_eq!(line.counted_total(), None);
    assert_eq!(line.price_change, None);
    assert!(line.problem.unwrap().contains("no price"));
}
