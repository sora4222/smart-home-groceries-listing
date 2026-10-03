//! The automated order's dislike rules, one spec line per test.

use chrono::Utc;
use uuid::Uuid;

use super::*;

#[derive(Debug, PartialEq, Eq)]
struct Candidate(Store, &'static str);

impl CandidateProduct for Candidate {
    fn store(&self) -> Store {
        self.0
    }
    fn product_id(&self) -> &str {
        self.1
    }
}

fn dislike(user: &str, store: Store, product_id: &str) -> ProductDislike {
    ProductDislike {
        id: Uuid::new_v4(),
        user_id: user.to_string(),
        user_name: format!("{user} name"),
        store,
        product_id: product_id.to_string(),
        product_name: "Milk".to_string(),
        brand: None,
        package_size: None,
        disliked_at: Utc::now(),
    }
}

fn override_for(store: Store, product_id: &str) -> DislikeOverride {
    DislikeOverride {
        id: Uuid::new_v4(),
        grocery_item_id: Uuid::new_v4(),
        store,
        product_id: product_id.to_string(),
        overridden_by: "jesse".to_string(),
        overridden_at: Utc::now(),
    }
}

const WOOLIES: Store = Store::Woolworths;

fn candidates() -> Vec<Candidate> {
    vec![Candidate(WOOLIES, "a"), Candidate(Store::Coles, "b")]
}

#[test]
fn nobody_dislikes_it_so_the_best_candidate_is_allowed() {
    let list = candidates();
    let pick = pick_for_automated_order(&list, &DislikeBook::default(), DislikeScope::Household);
    assert_eq!(pick, AutomatedPick::Allowed(&list[0]));
}

#[test]
fn a_household_order_skips_what_any_member_dislikes() {
    let list = candidates();
    let book = DislikeBook::new(vec![dislike("phu", WOOLIES, "a")], vec![]);
    let pick = pick_for_automated_order(&list, &book, DislikeScope::Household);
    assert_eq!(pick, AutomatedPick::Allowed(&list[1]));
}

#[test]
fn a_members_order_skips_only_what_that_member_dislikes() {
    let list = candidates();
    let book = DislikeBook::new(vec![dislike("phu", WOOLIES, "a")], vec![]);

    let jesse = pick_for_automated_order(&list, &book, DislikeScope::Member("jesse"));
    assert_eq!(jesse, AutomatedPick::Allowed(&list[0]));

    let phu = pick_for_automated_order(&list, &book, DislikeScope::Member("phu"));
    assert_eq!(phu, AutomatedPick::Allowed(&list[1]));
}

#[test]
fn the_same_product_id_at_the_other_store_is_a_different_product() {
    let list = vec![Candidate(Store::Coles, "a")];
    let book = DislikeBook::new(vec![dislike("phu", WOOLIES, "a")], vec![]);
    let pick = pick_for_automated_order(&list, &book, DislikeScope::Household);
    assert_eq!(pick, AutomatedPick::Allowed(&list[0]));
}

#[test]
fn an_override_for_this_order_means_no_skip() {
    let list = candidates();
    let book = DislikeBook::new(
        vec![dislike("phu", WOOLIES, "a")],
        vec![override_for(WOOLIES, "a")],
    );
    let pick = pick_for_automated_order(&list, &book, DislikeScope::Household);
    assert_eq!(pick, AutomatedPick::Allowed(&list[0]));
    // The dislike itself is still there.
    assert_eq!(
        book.disliked_by(WOOLIES, "a", DislikeScope::Household),
        ["phu name"]
    );
}

#[test]
fn when_every_candidate_is_disliked_the_best_is_bought_with_a_warning() {
    let list = candidates();
    let book = DislikeBook::new(
        vec![
            dislike("phu", WOOLIES, "a"),
            dislike("jesse", WOOLIES, "a"),
            dislike("jesse", Store::Coles, "b"),
        ],
        vec![],
    );
    let pick = pick_for_automated_order(&list, &book, DislikeScope::Household);
    assert_eq!(
        pick,
        AutomatedPick::OnlyDisliked {
            candidate: &list[0],
            disliked_by: vec!["phu name".to_string(), "jesse name".to_string()],
        }
    );
}

#[test]
fn no_candidates_means_nothing_to_pick() {
    let list: Vec<Candidate> = vec![];
    let pick = pick_for_automated_order(&list, &DislikeBook::default(), DislikeScope::Household);
    assert_eq!(pick, AutomatedPick::NoCandidates);
}
