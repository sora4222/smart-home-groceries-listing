//! Managing item rules: the settings page's list, add, edit and delete.
//!
//! What a rule does once it exists — putting chips on an item — is
//! `tests/item_rules_applied.rs`.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

/// Creates a rule and returns its body, failing the test if that did not work.
async fn create_rule(app: &TestApp, body: &Value) -> Value {
    let (status, created) = app.post_json("/api/item-rules", body).await;
    assert_eq!(status, StatusCode::CREATED, "setup failed: {created}");
    created
}

#[sqlx::test]
async fn a_new_rule_is_stored_and_listed(pool: PgPool) {
    let app = TestApp::new(pool);

    let created = create_rule(
        &app,
        &json!({ "triggers": ["toilet paper"], "filter_terms": ["3 ply"] }),
    )
    .await;

    assert_eq!(created["triggers"], json!(["toilet paper"]));
    assert_eq!(created["filter_terms"], json!(["3 ply"]));
    assert_eq!(created["apply_to_manual"], false, "off by default");
    assert!(created["id"].is_string());

    let (status, listed) = app.get("/api/item-rules").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["id"], created["id"]);
}

#[sqlx::test]
async fn rules_are_listed_in_the_order_they_were_made(pool: PgPool) {
    let app = TestApp::new(pool);
    create_rule(
        &app,
        &json!({ "triggers": ["milk"], "filter_terms": ["a2"] }),
    )
    .await;
    create_rule(
        &app,
        &json!({ "triggers": ["eggs"], "filter_terms": ["free range"] }),
    )
    .await;

    let (_, listed) = app.get("/api/item-rules").await;

    assert_eq!(listed[0]["triggers"], json!(["milk"]));
    assert_eq!(listed[1]["triggers"], json!(["eggs"]));
}

#[sqlx::test]
async fn triggers_and_terms_are_tidied(pool: PgPool) {
    let app = TestApp::new(pool);

    let created = create_rule(
        &app,
        &json!({
            "triggers": ["  toilet   paper ", "Toilet Paper", "loo roll", " "],
            "filter_terms": ["3 ply", " 3 PLY ", "recycled"],
            "apply_to_manual": true
        }),
    )
    .await;

    assert_eq!(created["triggers"], json!(["toilet paper", "loo roll"]));
    assert_eq!(created["filter_terms"], json!(["3 ply", "recycled"]));
    assert_eq!(created["apply_to_manual"], true);
}

#[sqlx::test]
async fn a_rule_needs_a_trigger_and_a_filter(pool: PgPool) {
    let app = TestApp::new(pool);

    for body in [
        json!({ "triggers": [], "filter_terms": ["3 ply"] }),
        json!({ "triggers": ["   "], "filter_terms": ["3 ply"] }),
        json!({ "triggers": ["toilet paper"], "filter_terms": [] }),
        json!({ "triggers": ["toilet paper"], "filter_terms": ["  "] }),
        json!({ "filter_terms": ["3 ply"] }),
    ] {
        let (status, _) = app.post_json("/api/item-rules", &body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "for {body}");
    }

    let (_, listed) = app.get("/api/item-rules").await;
    assert_eq!(listed, json!([]), "nothing invalid was stored");
}

#[sqlx::test]
async fn a_rule_is_bounded_like_the_columns(pool: PgPool) {
    let app = TestApp::new(pool);
    let eleven: Vec<String> = (0..11).map(|n| format!("term {n}")).collect();

    for body in [
        json!({ "triggers": eleven, "filter_terms": ["3 ply"] }),
        json!({ "triggers": ["milk"], "filter_terms": eleven }),
        json!({ "triggers": ["x".repeat(101)], "filter_terms": ["3 ply"] }),
        json!({ "triggers": ["milk"], "filter_terms": ["x".repeat(61)] }),
    ] {
        let (status, _) = app.post_json("/api/item-rules", &body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[sqlx::test]
async fn an_edit_changes_only_what_it_mentions(pool: PgPool) {
    let app = TestApp::new(pool);
    let rule = create_rule(
        &app,
        &json!({ "triggers": ["toilet paper"], "filter_terms": ["3 ply"] }),
    )
    .await;
    let id = rule["id"].as_str().unwrap();

    let (status, edited) = app
        .patch_json(
            &format!("/api/item-rules/{id}"),
            &json!({ "apply_to_manual": true, "filter_terms": ["4 ply"] }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(edited["triggers"], json!(["toilet paper"]));
    assert_eq!(edited["filter_terms"], json!(["4 ply"]));
    assert_eq!(edited["apply_to_manual"], true);
}

#[sqlx::test]
async fn an_edit_cannot_empty_a_rule(pool: PgPool) {
    let app = TestApp::new(pool);
    let rule = create_rule(
        &app,
        &json!({ "triggers": ["milk"], "filter_terms": ["a2"] }),
    )
    .await;
    let id = rule["id"].as_str().unwrap();

    let (status, _) = app
        .patch_json(
            &format!("/api/item-rules/{id}"),
            &json!({ "triggers": [" "] }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn editing_or_deleting_a_missing_rule_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let missing = "/api/item-rules/00000000-0000-0000-0000-000000000000";

    let (status, _) = app
        .patch_json(missing, &json!({ "apply_to_manual": true }))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = app.delete(missing).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_deleted_rule_is_gone(pool: PgPool) {
    let app = TestApp::new(pool);
    let rule = create_rule(
        &app,
        &json!({ "triggers": ["milk"], "filter_terms": ["a2"] }),
    )
    .await;
    let id = rule["id"].as_str().unwrap();

    let (status, _) = app.delete(&format!("/api/item-rules/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, listed) = app.get("/api/item-rules").await;
    assert_eq!(listed, json!([]));
}

#[sqlx::test]
async fn every_rule_route_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let id = "00000000-0000-0000-0000-000000000000";

    let (status, _) = app.get("/api/item-rules").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app
        .post_json(
            "/api/item-rules",
            &json!({ "triggers": ["milk"], "filter_terms": ["a2"] }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app
        .patch_json(&format!("/api/item-rules/{id}"), &json!({}))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app.delete(&format!("/api/item-rules/{id}")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
