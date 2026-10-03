//! The planner's modes, many items, and items no store prices. Pure.

use rust_decimal_macros::dec;

use super::build::plan;
use super::option::{Candidate, OptionKind};
use super::test_fixtures::{candidate, settings};
use crate::models::delivery_rows::OrderMode;
use crate::services::order_review::LineStatus;
use crate::services::stores::Store::{Coles, Woolworths};

#[test]
fn minimise_delivery_takes_the_cheaper_delivery_over_cheaper_items() {
    let mut fees = settings(dec!(9), None, None);
    fees.stores[1].delivery_fee = Some(dec!(0));
    let items = [candidate(
        "milk",
        Woolworths,
        &[(Woolworths, dec!(3)), (Coles, dec!(8))],
    )];

    let by_total = plan(OrderMode::MinimiseTotal, &items, vec![], &fees);
    let by_delivery = plan(OrderMode::MinimiseDelivery, &items, vec![], &fees);

    assert_eq!(by_total.options[0].kind, OptionKind::Only(Coles));
    assert_eq!(by_delivery.options[0].kind, OptionKind::Only(Coles));
    // Make Coles' items dearer than its free delivery saves: only the
    // delivery mode still picks Coles.
    let dear = [candidate(
        "milk",
        Woolworths,
        &[(Woolworths, dec!(3)), (Coles, dec!(15))],
    )];
    assert_eq!(
        plan(OrderMode::MinimiseTotal, &dear, vec![], &fees).options[0].kind,
        OptionKind::Only(Woolworths)
    );
    assert_eq!(
        plan(OrderMode::MinimiseDelivery, &dear, vec![], &fees).options[0].kind,
        OptionKind::Only(Coles)
    );
}

#[test]
fn a_single_store_mode_puts_that_store_first_and_lists_what_it_lacks() {
    let items = [
        candidate(
            "milk",
            Woolworths,
            &[(Woolworths, dec!(3)), (Coles, dec!(4))],
        ),
        candidate("bread", Woolworths, &[(Woolworths, dec!(3))]),
    ];
    let planned = plan(
        OrderMode::ColesOnly,
        &items,
        vec![],
        &settings(dec!(9), None, None),
    );

    let first = &planned.options[0];
    assert_eq!(first.kind, OptionKind::Only(Coles));
    assert_eq!(first.missing.len(), 1);
    assert_eq!(first.missing[0].item.name, "bread");
    assert_eq!(first.missing[0].reason, "No product chosen at Coles.");
    // It cannot buy everything, so it is not recommended as it is.
    assert_eq!(planned.recommended, None);
}

#[test]
fn manual_mode_puts_the_current_stores_first() {
    let items = [
        candidate("milk", Coles, &[(Woolworths, dec!(3)), (Coles, dec!(9))]),
        candidate(
            "bread",
            Woolworths,
            &[(Woolworths, dec!(3)), (Coles, dec!(9))],
        ),
    ];
    let planned = plan(
        OrderMode::Manual,
        &items,
        vec![],
        &settings(dec!(0), None, None),
    );

    assert_eq!(planned.options[0].kind, OptionKind::Current);
    assert_eq!(planned.current, Some(0));
    assert_eq!(planned.options[0].total, dec!(12));
}

#[test]
fn many_items_with_a_choice_still_find_the_obvious_best() {
    let items: Vec<Candidate> = (0..20)
        .map(|i| {
            candidate(
                &format!("item {i}"),
                Coles,
                &[(Woolworths, dec!(2)), (Coles, dec!(3))],
            )
        })
        .collect();
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(9), None, None),
    );

    assert!(!planned.exact);
    assert_eq!(planned.options[0].kind, OptionKind::Only(Woolworths));
    assert_eq!(planned.options[0].total, dec!(49));
}

#[test]
fn an_item_priced_nowhere_is_missing_from_every_option() {
    let mut gone = candidate("milk", Woolworths, &[(Woolworths, dec!(3))]);
    gone.offers[0].status = LineStatus::NotOffered;
    gone.offers[0].total_price = None;
    gone.offers[0].problem = Some("Woolworths no longer offers this product.".into());
    let items = [
        gone,
        candidate("bread", Woolworths, &[(Woolworths, dec!(4))]),
    ];

    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(9), None, None),
    );

    assert_eq!(planned.options.len(), 1);
    let only = &planned.options[0];
    assert_eq!(
        only.missing[0].reason,
        "Woolworths no longer offers this product."
    );
    assert_eq!(planned.recommended, None);
}
