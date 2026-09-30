//! What a rule does: putting its filter chips on the items it matches.
//!
//! The spec's "Item Rules > Runtime behaviour": an accepted voice item always
//! gets the chips of every rule whose trigger matches its name; an item typed
//! into the web app only gets them from rules with "apply to manual additions"
//! switched on. The chips are the item's own from then on.
//!
//! Managing the rules themselves is `tests/item_rules.rs`.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

/// Stores a rule, failing the test if that did not work, and returns its id.
async fn rule(app: &TestApp, triggers: &[&str], terms: &[&str], manual: bool) -> String {
    let (status, created) = app
        .post_json(
            "/api/item-rules",
            &json!({ "triggers": triggers, "filter_terms": terms, "apply_to_manual": manual }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "setup failed: {created}");
    created["id"].as_str().unwrap().to_string()
}

/// Says an item by voice and accepts it, returning the resulting list item.
async fn say_and_accept(app: &TestApp, item: &str) -> Value {
    let id = app.create_pending(item, 1).await;
    let (status, body) = app.post(&format!("/api/voice-requests/{id}/accept")).await;
    assert_eq!(status, StatusCode::OK, "accept failed: {body}");
    body["grocery_item"].clone()
}

#[sqlx::test]
async fn an_accepted_voice_item_gets_the_rules_chips(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["toilet paper"], &["3 ply"], false).await;

    let item = say_and_accept(&app, "toilet paper").await;

    assert_eq!(item["filter_terms"], json!(["3 ply"]));
    let (_, list) = app.get("/api/grocery-items").await;
    assert_eq!(
        list[0]["filter_terms"],
        json!(["3 ply"]),
        "stored, not just echoed"
    );
}

#[sqlx::test]
async fn a_trigger_matches_whole_words_in_any_case(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["toilet paper"], &["3 ply"], false).await;
    rule(&app, &["milk"], &["a2"], false).await;

    assert_eq!(
        say_and_accept(&app, "Quilton Toilet  Paper").await["filter_terms"],
        json!(["3 ply"])
    );
    assert_eq!(
        say_and_accept(&app, "milkshake").await["filter_terms"],
        json!([]),
        "part of a word is not a match"
    );
    assert_eq!(
        say_and_accept(&app, "paper towel").await["filter_terms"],
        json!([]),
        "part of the phrase is not a match"
    );
}

#[sqlx::test]
async fn any_of_a_rules_triggers_will_do(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["toilet paper", "loo roll"], &["3 ply"], false).await;

    let item = say_and_accept(&app, "loo roll").await;

    assert_eq!(item["filter_terms"], json!(["3 ply"]));
}

#[sqlx::test]
async fn every_matching_rule_contributes_without_repeats(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["milk"], &["full cream", "2 litre"], false).await;
    rule(&app, &["cream"], &["Full Cream", "australian"], false).await;

    let item = say_and_accept(&app, "full cream milk").await;

    assert_eq!(
        item["filter_terms"],
        json!(["full cream", "2 litre", "australian"])
    );
}

#[sqlx::test]
async fn the_corrected_name_is_the_one_matched(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["toilet paper"], &["3 ply"], false).await;
    let id = app.create_pending("toy lit paper", 1).await;

    let (_, body) = app
        .post_json(
            &format!("/api/voice-requests/{id}/accept"),
            &json!({ "name": "toilet paper" }),
        )
        .await;

    assert_eq!(body["grocery_item"]["filter_terms"], json!(["3 ply"]));
}

#[sqlx::test]
async fn merging_into_an_item_does_not_bring_back_a_removed_chip(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["toilet paper"], &["3 ply"], false).await;
    let first = say_and_accept(&app, "toilet paper").await;
    let item_id = first["id"].as_str().unwrap();
    app.patch_json(
        &format!("/api/grocery-items/{item_id}"),
        &json!({ "filter_terms": [] }),
    )
    .await;

    let second = app.create_pending("toilet paper", 2).await;
    let (status, body) = app
        .post(&format!("/api/voice-requests/{second}/accept?merge=true"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["grocery_item"]["quantity"], 3);
    assert_eq!(body["grocery_item"]["filter_terms"], json!([]));
}

#[sqlx::test]
async fn removing_a_chip_from_an_item_leaves_the_rule_alone(pool: PgPool) {
    let app = TestApp::new(pool);
    rule(&app, &["toilet paper"], &["3 ply"], false).await;
    let item = say_and_accept(&app, "toilet paper").await;

    app.patch_json(
        &format!("/api/grocery-items/{}", item["id"].as_str().unwrap()),
        &json!({ "filter_terms": [] }),
    )
    .await;

    let (_, rules) = app.get("/api/item-rules").await;
    assert_eq!(rules[0]["filter_terms"], json!(["3 ply"]));
}
