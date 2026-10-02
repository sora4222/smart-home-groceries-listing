//! The four spending views over a handful of purchases.

use rust_decimal_macros::dec;

use super::*;
use crate::services::purchases::fixtures::purchase;

const SYDNEY: Tz = chrono_tz::Australia::Sydney;

fn day(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
}

fn bought(item: &str, category: &str, store: Store, total: Decimal, on: &str) -> Purchase {
    let mut p = purchase(store, item, total, on);
    p.item_name = item.to_string();
    p.item_key = item.to_lowercase();
    p.category = category.to_string();
    p
}

fn sample() -> Vec<Purchase> {
    let mut milk = bought("Milk", "Dairy & eggs", Store::Coles, dec!(5), "2026-07-03");
    milk.delivery_fee_share = dec!(2);
    vec![
        milk,
        bought("Bread", "Bakery", Store::Woolworths, dec!(4), "2026-07-20"),
        bought(
            "milk",
            "Dairy & eggs",
            Store::Woolworths,
            dec!(6),
            "2026-09-10",
        ),
    ]
}

#[test]
fn totals_keep_items_and_delivery_apart() {
    let result = build(&sample(), Period::Month, SYDNEY, &Store::ALL);
    assert_eq!(result.items_total, dec!(15));
    assert_eq!(result.delivery_total, dec!(2));
}

#[test]
fn over_time_fills_empty_months_with_zero_and_includes_delivery() {
    let result = build(&sample(), Period::Month, SYDNEY, &Store::ALL);
    assert_eq!(
        result.over_time,
        vec![
            PeriodSpend {
                start: day("2026-07-01"),
                total: dec!(11)
            },
            PeriodSpend {
                start: day("2026-08-01"),
                total: dec!(0)
            },
            PeriodSpend {
                start: day("2026-09-01"),
                total: dec!(6)
            },
        ]
    );
}

#[test]
fn over_time_by_quarter() {
    let result = build(&sample(), Period::Quarter, SYDNEY, &Store::ALL);
    assert_eq!(
        result.over_time,
        vec![PeriodSpend {
            start: day("2026-07-01"),
            total: dec!(17)
        }]
    );
}

#[test]
fn by_item_groups_names_by_their_key_biggest_first() {
    let result = build(&sample(), Period::Month, SYDNEY, &Store::ALL);
    let milk = &result.by_item[0];
    assert_eq!(milk.item_key, "milk");
    assert_eq!(milk.item_name, "milk", "the newest name is shown");
    assert_eq!(milk.times_bought, 2);
    assert_eq!(milk.quantity, 2);
    assert_eq!(milk.total, dec!(11));
    assert_eq!(result.by_item[1].item_key, "bread");
}

#[test]
fn by_store_lists_every_asked_store_even_with_nothing_spent() {
    let only_bread = vec![bought(
        "Bread",
        "Bakery",
        Store::Woolworths,
        dec!(4),
        "2026-07-20",
    )];
    let result = build(&only_bread, Period::Month, SYDNEY, &Store::ALL);
    assert_eq!(result.by_store.len(), 2);
    assert_eq!(result.by_store[0].store, Store::Woolworths);
    assert_eq!(result.by_store[0].total(), dec!(4));
    assert_eq!(result.by_store[1].store, Store::Coles);
    assert_eq!(result.by_store[1].total(), dec!(0));
}

#[test]
fn by_store_splits_items_and_delivery() {
    let result = build(&sample(), Period::Month, SYDNEY, &[Store::Coles]);
    assert_eq!(result.by_store.len(), 1);
    assert_eq!(result.by_store[0].items_total, dec!(5));
    assert_eq!(result.by_store[0].delivery_total, dec!(2));
}

#[test]
fn by_category_biggest_first() {
    let result = build(&sample(), Period::Month, SYDNEY, &Store::ALL);
    assert_eq!(
        result.by_category,
        vec![
            CategorySpend {
                category: "Dairy & eggs".into(),
                total: dec!(11)
            },
            CategorySpend {
                category: "Bakery".into(),
                total: dec!(4)
            },
        ]
    );
}

#[test]
fn nothing_bought_is_empty_views() {
    let result = build(&[], Period::Week, SYDNEY, &Store::ALL);
    assert!(result.over_time.is_empty());
    assert!(result.by_item.is_empty());
    assert!(result.by_category.is_empty());
    assert_eq!(result.items_total, dec!(0));
}
