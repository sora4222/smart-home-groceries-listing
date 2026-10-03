//! Turning the re-priced items into the ranked options the household sees.
//!
//! Pure. Four candidates are built: everything at Woolworths, everything at
//! Coles, the best mix ([`search`](super::search)), and the stores the order
//! buys from now. Two that buy the same items at the same stores are one
//! option; the stores bought from now are then marked on it.

use uuid::Uuid;

use super::option::{evaluate, Candidate, OptionKind, PlanOption};
use super::rank;
use super::search::best_mix;
use super::OrderPlan;
use crate::models::db::GroceryItem;
use crate::models::delivery_rows::OrderMode;
use crate::services::delivery_settings::DeliverySettings;
use crate::services::stores::Store;

/// Plans the order from `candidates` (items with at least one choice) and
/// `unchosen` (committed items with none).
pub fn plan(
    mode: OrderMode,
    candidates: &[Candidate],
    unchosen: Vec<GroceryItem>,
    settings: &DeliverySettings,
) -> OrderPlan {
    if candidates.is_empty() {
        return OrderPlan {
            mode,
            options: Vec::new(),
            recommended: None,
            current: None,
            unchosen,
            exact: true,
        };
    }
    let found = best_mix(mode, candidates, settings);
    let only = |store: Store| -> Vec<Option<Store>> {
        candidates
            .iter()
            .map(|c| c.price_at(store).map(|_| store))
            .collect()
    };
    let now = now_assignment(candidates);

    let mut options: Vec<PlanOption> = Vec::new();
    for (kind, assignment) in [
        (OptionKind::Only(Store::Woolworths), only(Store::Woolworths)),
        (OptionKind::Only(Store::Coles), only(Store::Coles)),
        (OptionKind::Split, found.assignment),
        (OptionKind::Current, now),
    ] {
        let option = evaluate(kind, candidates, &assignment, settings);
        let empty = option.parts.is_empty();
        let repeated = options.iter().any(|o| same_picks(o, &option));
        if !empty && !repeated {
            options.push(option);
        }
    }
    rank::sort(mode, &mut options);

    let bought_now = evaluate(
        OptionKind::Current,
        candidates,
        &now_assignment(candidates),
        settings,
    );
    let current = options.iter().position(|o| same_picks(o, &bought_now));
    let recommended = options
        .first()
        .filter(|best| best.acceptable() && best.complete())
        .map(|_| 0);
    OrderPlan {
        mode,
        options,
        recommended,
        current,
        unchosen,
        exact: found.exact,
    }
}

/// The stores the order buys from now, where the item is priced there.
fn now_assignment(candidates: &[Candidate]) -> Vec<Option<Store>> {
    candidates
        .iter()
        .map(|c| c.price_at(c.current).map(|_| c.current))
        .collect()
}

/// Both options buy exactly the same items at the same stores.
fn same_picks(a: &PlanOption, b: &PlanOption) -> bool {
    sorted(a.picks()) == sorted(b.picks())
}

fn sorted(mut picks: Vec<(Uuid, Store)>) -> Vec<(Uuid, &'static str)> {
    picks.sort_by_key(|(id, _)| *id);
    picks
        .into_iter()
        .map(|(id, store)| (id, store.as_str()))
        .collect()
}
