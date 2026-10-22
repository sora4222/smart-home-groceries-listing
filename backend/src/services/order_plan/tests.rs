//! The planner's choices on small, hand-made orders: delivery fees, free
//! delivery, minimums and the cap. Pure: no database, no stores.

use rust_decimal_macros::dec;

use super::build::plan;
use super::option::OptionKind;
use super::test_fixtures::{candidate, settings};
use crate::models::delivery_rows::OrderMode;
use crate::services::order_review::fixtures::item;
use crate::services::stores::Store::{Coles, Woolworths};

#[test]
fn nothing_to_plan_has_no_options() {
    let planned = plan(
        OrderMode::MinimiseTotal,
        &[],
        vec![item("bread", 1)],
        &settings(dec!(9), None, None),
    );
    assert!(planned.options.is_empty());
    assert_eq!(planned.recommended, None);
    assert_eq!(planned.unchosen.len(), 1);
}

#[test]
fn splitting_wins_when_it_saves_more_than_the_second_delivery() {
    let items = [
        candidate(
            "milk",
            Woolworths,
            &[(Woolworths, dec!(3)), (Coles, dec!(20))],
        ),
        candidate(
            "bread",
            Woolworths,
            &[(Woolworths, dec!(20)), (Coles, dec!(3))],
        ),
    ];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(5), None, None),
    );

    let best = &planned.options[0];
    assert_eq!(best.kind, OptionKind::Split);
    assert_eq!(best.items_total, dec!(6));
    assert_eq!(best.delivery_total, dec!(10));
    assert_eq!(best.total, dec!(16));
    assert_eq!(planned.recommended, Some(0));
}

#[test]
fn one_store_wins_when_the_saving_is_less_than_a_second_delivery() {
    let items = [
        candidate("milk", Coles, &[(Woolworths, dec!(3)), (Coles, dec!(4))]),
        candidate("bread", Coles, &[(Woolworths, dec!(5)), (Coles, dec!(4))]),
    ];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(9), None, None),
    );

    // Split would save $1 on milk and cost $9 more delivery.
    let best = &planned.options[0];
    assert_eq!(best.kind, OptionKind::Only(Woolworths));
    assert_eq!(best.total, dec!(17));
    // Coles-only is what the order buys now.
    let coles = planned
        .options
        .iter()
        .position(|o| o.kind == OptionKind::Only(Coles));
    assert_eq!(planned.current, coles);
}

#[test]
fn moving_an_item_to_reach_free_delivery_beats_the_cheapest_per_item() {
    // Cheapest per item: milk at Woolworths, the rest at Coles ($45 + $9 fee
    // at Coles, $3 + $9 at Woolworths). Buying milk at Coles too reaches
    // Coles' $50 free delivery.
    let items = [
        candidate(
            "milk",
            Woolworths,
            &[(Woolworths, dec!(3)), (Coles, dec!(5))],
        ),
        candidate("meat", Coles, &[(Coles, dec!(45))]),
    ];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(9), Some(dec!(50)), None),
    );

    let best = &planned.options[0];
    assert_eq!(best.kind, OptionKind::Only(Coles));
    assert_eq!(best.total, dec!(50));
    assert!(best.parts[0].charge.free);
}

#[test]
fn an_order_under_a_stores_minimum_is_not_recommended_first() {
    let items = [
        candidate(
            "milk",
            Woolworths,
            &[(Woolworths, dec!(3)), (Coles, dec!(40))],
        ),
        candidate("meat", Coles, &[(Woolworths, dec!(60)), (Coles, dec!(45))]),
    ];
    let planned = plan(
        OrderMode::MinimiseTotal,
        &items,
        vec![],
        &settings(dec!(0), None, Some(dec!(30))),
    );

    // Milk alone at Woolworths ($3) is under the $30 minimum.
    let best = &planned.options[0];
    assert!(best.meets_minimums(), "{:?}", best.kind);
    assert_eq!(best.kind, OptionKind::Only(Woolworths));
    assert_eq!(best.total, dec!(63));
    let split = planned.options.iter().find(|o| o.kind == OptionKind::Split);
    assert!(
        split.is_none(),
        "the best mix is all at Woolworths, so it is not repeated"
    );
}

#[test]
fn the_best_mix_keeps_within_the_delivery_cap() {
    let items = [
        candidate(
            "milk",
            Woolworths,
            &[(Woolworths, dec!(3)), (Coles, dec!(20))],
        ),
        candidate(
            "bread",
            Woolworths,
            &[(Woolworths, dec!(20)), (Coles, dec!(3))],
        ),
    ];
    let mut capped = settings(dec!(5), None, None);
    capped.max_delivery_spend = Some(dec!(5));
    let planned = plan(OrderMode::MinimiseTotal, &items, vec![], &capped);

    // Two deliveries ($10) would be cheaper in total ($16) but break the cap,
    // so the mix is one store, and it is not listed twice.
    assert_eq!(planned.options[0].total, dec!(28));
    assert!(planned.options.iter().all(|o| o.within_cap));
    assert!(planned.options.iter().all(|o| o.kind != OptionKind::Split));
}

#[test]
fn an_option_over_the_cap_is_not_recommended() {
    let items = [candidate("milk", Woolworths, &[(Woolworths, dec!(3))])];
    let mut capped = settings(dec!(9), None, None);
    capped.max_delivery_spend = Some(dec!(5));

    let planned = plan(OrderMode::MinimiseTotal, &items, vec![], &capped);

    assert!(!planned.options[0].within_cap);
    assert_eq!(planned.recommended, None);
}
