//! Pricing a product put in the trolley: from the store, else from the choice.

use rust_decimal_macros::dec;

use super::*;
use crate::models::db::GroceryItemSource;
use crate::services::stores::product::fixtures::product;
use crate::services::stores::{Deal, StoreError};

fn item(quantity: i32) -> GroceryItem {
    GroceryItem {
        id: Uuid::new_v4(),
        name: "milk".into(),
        quantity,
        status: GroceryItemStatus::Committed,
        source: GroceryItemSource::Manual,
        note: None,
        filter_terms: Vec::new(),
        added_by_user_id: None,
        created_at: chrono::Utc::now(),
    }
}

fn line(item: &GroceryItem, product_id: &str, quantity: i32) -> TrolleyHandoffLine {
    TrolleyHandoffLine {
        handoff_id: Uuid::new_v4(),
        grocery_item_id: item.id,
        product_id: product_id.into(),
        product_name: "Handoff name".into(),
        quantity,
        outcome: None,
        problem: None,
    }
}

fn choice(item: &GroceryItem, product_id: &str, total: Decimal, priced: i32) -> ItemSelection {
    ItemSelection {
        id: Uuid::new_v4(),
        grocery_item_id: item.id,
        store: Store::Coles,
        product_id: product_id.into(),
        product_name: "Chosen name".into(),
        brand: Some("Chosen brand".into()),
        package_size: Some("3L".into()),
        price: Some(dec!(4.95)),
        unit_price: None,
        unit_price_per: None,
        total_price: Some(total),
        priced_quantity: priced,
        url: String::new(),
        selected_by: "user".into(),
        selected_at: chrono::Utc::now(),
    }
}

fn answer(products: Vec<crate::services::stores::Product>) -> StoreOutcome {
    StoreOutcome {
        store: Store::Coles,
        products: products
            .into_iter()
            .map(|p| PricedProduct::new(p, 1))
            .collect(),
        error: None,
    }
}

#[test]
fn the_stores_answer_prices_the_trolley_quantity_with_deals() {
    let milk = item(3);
    let mut found = product(Store::Coles, "Oat Milk", dec!(3.90));
    found.deals = vec![Deal {
        description: "2 for $6.50".into(),
        min_quantity: 2,
        unit_price: dec!(3.25),
    }];
    found.category = Some("Long-Life Milk".into());
    let outcome = answer(vec![found]);

    let bought = price_line(
        Store::Coles,
        &line(&milk, "oat-milk", 3),
        &milk,
        None,
        Some(&outcome),
    )
    .unwrap();

    assert_eq!(bought.total_price, dec!(10.40));
    assert_eq!(bought.unit_price, dec!(3.47));
    assert_eq!(bought.shelf_price, Some(dec!(3.90)));
    assert_eq!(bought.product_name, "Oat Milk");
    assert_eq!(bought.store_category.as_deref(), Some("Long-Life Milk"));
    assert_eq!(bought.item_name, "milk");
    assert_eq!(bought.item_status_before, GroceryItemStatus::Committed);
}

#[test]
fn the_saved_choice_is_used_when_the_store_failed() {
    let milk = item(2);
    let outcome = StoreOutcome {
        store: Store::Coles,
        products: Vec::new(),
        error: Some(StoreError::Unreachable("down".into())),
    };
    let saved = choice(&milk, "c-milk", dec!(9.90), 2);

    let bought = price_line(
        Store::Coles,
        &line(&milk, "c-milk", 3),
        &milk,
        Some(&saved),
        Some(&outcome),
    )
    .unwrap();

    // 9.90 for 2 → 4.95 each → 14.85 for 3.
    assert_eq!(bought.total_price, dec!(14.85));
    assert_eq!(bought.unit_price, dec!(4.95));
    assert_eq!(bought.product_name, "Handoff name");
    assert_eq!(bought.brand.as_deref(), Some("Chosen brand"));
    assert_eq!(bought.store_category, None);
}

#[test]
fn a_choice_of_another_product_is_not_used() {
    let milk = item(1);
    let saved = choice(&milk, "something-else", dec!(5), 1);

    let bought = price_line(
        Store::Coles,
        &line(&milk, "c-milk", 1),
        &milk,
        Some(&saved),
        None,
    );

    assert_eq!(bought, None);
}

#[test]
fn a_product_the_store_no_longer_lists_falls_back_to_the_choice() {
    let milk = item(1);
    let outcome = answer(vec![product(Store::Coles, "Other", dec!(1))]);
    let saved = choice(&milk, "c-milk", dec!(4.95), 1);

    let bought = price_line(
        Store::Coles,
        &line(&milk, "c-milk", 1),
        &milk,
        Some(&saved),
        Some(&outcome),
    )
    .unwrap();

    assert_eq!(bought.total_price, dec!(4.95));
}

#[test]
fn no_price_anywhere_is_none() {
    let milk = item(1);
    assert_eq!(
        price_line(Store::Coles, &line(&milk, "c-milk", 1), &milk, None, None),
        None
    );
}
