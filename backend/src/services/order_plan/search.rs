//! Finding the best mix of stores for the order.
//!
//! Each item may be priced at more than one store (the household chose a
//! product at each). Delivery fees depend on each store's subtotal (free from
//! an amount, a minimum order), so the cheapest store per item is not always
//! the cheapest order: moving an item can make one store's delivery free, or
//! save a whole delivery.
//!
//! Up to [`EXACT_LIMIT`] items with a choice of store, every combination is
//! tried, so the answer is the best one. Beyond that, a local search starts
//! from a few sensible points and moves one item at a time while that helps;
//! the answer is then good but not proven best (`exact: false`). Pure.

use std::cmp::Ordering;

use rust_decimal::Decimal;

use super::fees::charge;
use super::option::Candidate;
use super::rank::{compare, Measure};
use crate::models::delivery_rows::OrderMode;
use crate::services::delivery_settings::DeliverySettings;
use crate::services::stores::Store;

/// Most items with a choice of store that are tried in every combination
/// (2^16 combinations for two stores).
pub const EXACT_LIMIT: usize = 16;

/// The best assignment found: `assignment[i]` is where `candidates[i]` is
/// bought, `None` for an item no store prices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub assignment: Vec<Option<Store>>,
    /// Every combination was tried.
    pub exact: bool,
}

/// The best way to buy every item that can be bought, any mix of stores.
pub fn best_mix(mode: OrderMode, candidates: &[Candidate], settings: &DeliverySettings) -> Found {
    let priced: Vec<Vec<(Store, Decimal)>> = candidates
        .iter()
        .map(|candidate| {
            Store::ALL
                .into_iter()
                .filter_map(|store| candidate.price_at(store).map(|price| (store, price)))
                .collect()
        })
        .collect();
    let flexible: Vec<usize> = (0..priced.len()).filter(|&i| priced[i].len() > 1).collect();
    let cheapest: Vec<Option<Store>> = priced
        .iter()
        .map(|offers| {
            offers
                .iter()
                .min_by_key(|(_, price)| *price)
                .map(|(s, _)| *s)
        })
        .collect();

    let scorer = Scorer {
        mode,
        priced: &priced,
        settings,
    };
    if flexible.len() <= EXACT_LIMIT {
        return Found {
            assignment: scorer.every_combination(&flexible, cheapest),
            exact: true,
        };
    }
    let starts = [
        cheapest.clone(),
        prefer(&priced, Store::Woolworths),
        prefer(&priced, Store::Coles),
    ];
    let assignment = starts
        .into_iter()
        .map(|start| scorer.improve(&flexible, start))
        .min_by(|a, b| scorer.compare(a, b))
        .unwrap_or(cheapest);
    Found {
        assignment,
        exact: false,
    }
}

/// Each item at `store` where it is priced there, else its first store.
fn prefer(priced: &[Vec<(Store, Decimal)>], store: Store) -> Vec<Option<Store>> {
    priced
        .iter()
        .map(|offers| {
            offers
                .iter()
                .find(|(s, _)| *s == store)
                .or_else(|| offers.first())
                .map(|(s, _)| *s)
        })
        .collect()
}

/// Measures assignments without building whole options.
struct Scorer<'a> {
    mode: OrderMode,
    priced: &'a [Vec<(Store, Decimal)>],
    settings: &'a DeliverySettings,
}

impl Scorer<'_> {
    fn measure(&self, assignment: &[Option<Store>]) -> Measure {
        let mut total = Decimal::ZERO;
        let mut delivery = Decimal::ZERO;
        let mut acceptable = true;
        let mut stores_used = 0;
        for store in Store::ALL {
            let mut subtotal = Decimal::ZERO;
            let mut used = false;
            for (offers, at) in self.priced.iter().zip(assignment) {
                if *at == Some(store) {
                    used = true;
                    subtotal += offers
                        .iter()
                        .find(|(s, _)| *s == store)
                        .map_or(Decimal::ZERO, |(_, p)| *p);
                }
            }
            if !used {
                continue;
            }
            let charged = charge(&self.settings.rules_for(store), subtotal);
            stores_used += 1;
            total += subtotal + charged.fee;
            delivery += charged.fee;
            acceptable &= !charged.below_minimum;
        }
        acceptable &= self
            .settings
            .max_delivery_spend
            .is_none_or(|cap| delivery <= cap);
        Measure {
            complete: true,
            acceptable,
            total,
            delivery,
            stores_used,
        }
    }

    fn compare(&self, a: &[Option<Store>], b: &[Option<Store>]) -> Ordering {
        compare(self.mode, &self.measure(a), &self.measure(b))
    }

    /// Tries every store for every flexible item, keeping the best.
    fn every_combination(
        &self,
        flexible: &[usize],
        start: Vec<Option<Store>>,
    ) -> Vec<Option<Store>> {
        let mut best = start.clone();
        let mut current = start;
        let mut choice = vec![0usize; flexible.len()];
        loop {
            for (slot, &item) in flexible.iter().enumerate() {
                current[item] = Some(self.priced[item][choice[slot]].0);
            }
            if self.compare(&current, &best) == Ordering::Less {
                best.clone_from(&current);
            }
            // Next combination, like counting with each item's own base.
            let mut slot = 0;
            loop {
                if slot == flexible.len() {
                    return best;
                }
                choice[slot] += 1;
                if choice[slot] < self.priced[flexible[slot]].len() {
                    break;
                }
                choice[slot] = 0;
                slot += 1;
            }
        }
    }

    /// Moves one item to another store at a time while that helps.
    fn improve(&self, flexible: &[usize], start: Vec<Option<Store>>) -> Vec<Option<Store>> {
        let mut best = start;
        let mut improved = true;
        while improved {
            improved = false;
            for &item in flexible {
                for &(store, _) in &self.priced[item] {
                    let mut tried = best.clone();
                    tried[item] = Some(store);
                    if self.compare(&tried, &best) == Ordering::Less {
                        best = tried;
                        improved = true;
                    }
                }
            }
        }
        best
    }
}
