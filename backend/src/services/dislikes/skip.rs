//! Which product an automated order may buy, given the household's dislikes.
//!
//! Pure: no database, no store. The optimiser (not built yet) loads a
//! [`DislikeBook`] for an item with `DislikeService::book_for_item`, ranks
//! its candidate products, and asks [`pick_for_automated_order`] for the
//! first one it may buy.
//!
//! The spec's rules:
//! - a [`DislikeScope::Member`] order skips what that member dislikes;
//! - a [`DislikeScope::Household`] order skips what any member dislikes;
//! - an override for the item ("buy it this time anyway") means no skip;
//! - when every candidate is disliked, the first is bought anyway and the
//!   order must warn before it is confirmed ([`AutomatedPick::OnlyDisliked`]).
//!
//! Manual choice never calls this: a person may choose a disliked product.

use crate::models::db::{DislikeOverride, ProductDislike};
use crate::services::stores::Store;

/// Whose dislikes an automated order respects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DislikeScope<'a> {
    /// Any member's dislike counts.
    Household,
    /// Only this member's (by user id) dislikes count.
    Member(&'a str),
}

/// A product the optimiser is weighing, named the way dislikes name it.
pub trait CandidateProduct {
    fn store(&self) -> Store;
    fn product_id(&self) -> &str;
}

/// The dislikes and the item's overrides that one decision needs.
#[derive(Debug, Clone, Default)]
pub struct DislikeBook {
    dislikes: Vec<ProductDislike>,
    overrides: Vec<DislikeOverride>,
}

impl DislikeBook {
    /// `overrides` must be the overrides of the one item being decided.
    pub fn new(dislikes: Vec<ProductDislike>, overrides: Vec<DislikeOverride>) -> Self {
        Self {
            dislikes,
            overrides,
        }
    }

    /// The names of the members in `scope` who dislike the product, in the
    /// order they disliked it. Overrides do not change who dislikes it.
    pub fn disliked_by(&self, store: Store, product_id: &str, scope: DislikeScope) -> Vec<&str> {
        self.dislikes
            .iter()
            .filter(|d| d.store == store && d.product_id == product_id)
            .filter(|d| match scope {
                DislikeScope::Household => true,
                DislikeScope::Member(user_id) => d.user_id == user_id,
            })
            .map(|d| d.user_name.as_str())
            .collect()
    }

    /// Whether someone said "buy it this time anyway" for this item.
    pub fn is_overridden(&self, store: Store, product_id: &str) -> bool {
        self.overrides
            .iter()
            .any(|o| o.store == store && o.product_id == product_id)
    }

    /// Whether an automated order in `scope` must skip the product.
    pub fn skips(&self, store: Store, product_id: &str, scope: DislikeScope) -> bool {
        !self.is_overridden(store, product_id)
            && !self.disliked_by(store, product_id, scope).is_empty()
    }
}

/// What an automated order may buy for one item.
#[derive(Debug, PartialEq, Eq)]
pub enum AutomatedPick<'c, T> {
    /// The best-ranked candidate nobody in scope dislikes.
    Allowed(&'c T),
    /// Every candidate is disliked. Buy the best-ranked one, but show who
    /// disliked it before the order is confirmed.
    OnlyDisliked {
        candidate: &'c T,
        disliked_by: Vec<String>,
    },
    /// There was nothing to choose from.
    NoCandidates,
}

/// The first of `candidates` (best first) an automated order may buy.
pub fn pick_for_automated_order<'c, T: CandidateProduct>(
    candidates: &'c [T],
    book: &DislikeBook,
    scope: DislikeScope,
) -> AutomatedPick<'c, T> {
    if let Some(allowed) = candidates
        .iter()
        .find(|c| !book.skips(c.store(), c.product_id(), scope))
    {
        return AutomatedPick::Allowed(allowed);
    }
    match candidates.first() {
        Some(first) => AutomatedPick::OnlyDisliked {
            candidate: first,
            disliked_by: book
                .disliked_by(first.store(), first.product_id(), scope)
                .into_iter()
                .map(str::to_string)
                .collect(),
        },
        None => AutomatedPick::NoCandidates,
    }
}

#[cfg(test)]
#[path = "skip_tests.rs"]
mod tests;
