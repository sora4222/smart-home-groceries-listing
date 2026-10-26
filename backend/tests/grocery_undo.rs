//! What the list's Undo buttons need from the backend: committing or
//! releasing only the items one press changed, and putting a removed item
//! back exactly as it was.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

async fn status_of(app: &TestApp, name: &str) -> String {
    let (_, listed) = app.get("/api/grocery-items").await;
    listed
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .map(|item| item["status"].as_str().unwrap().to_string())
        .unwrap_or_default()
}

#[sqlx::test]
async fn releasing_named_items_leaves_the_others_committed(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;
    let bread = app.add_item(&json!({ "name": "bread" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, released) = app
        .post(&format!("/api/grocery-items/release?ids={bread}"))
        .await;

    assert_eq!(status, StatusCode::OK, "{released}");
    assert_eq!(released.as_array().unwrap().len(), 1);
    assert_eq!(status_of(&app, "bread").await, "active");
    assert_eq!(status_of(&app, "milk").await, "committed");
}

#[sqlx::test]
async fn committing_named_items_leaves_the_others_active(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    app.add_item(&json!({ "name": "bread" })).await;

    let (status, _) = app
        .post(&format!("/api/grocery-items/commit?ids={milk}"))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(status_of(&app, "milk").await, "committed");
    assert_eq!(status_of(&app, "bread").await, "active");
}

#[sqlx::test]
async fn a_restored_item_keeps_its_chips_and_skips_the_rules(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, rule) = app
        .post_json(
            "/api/item-rules",
            &json!({ "triggers": ["milk"], "filter_terms": ["full cream"], "apply_to_manual": true }),
        )
        .await;
    assert!(status.is_success(), "{rule}");
    app.add_item(&json!({ "name": "milk" })).await;

    let (status, restored): (StatusCode, Value) = app
        .post_json(
            "/api/grocery-items/restore",
            &json!({ "name": "milk", "quantity": 2, "filter_terms": ["lactose free"] }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED, "{restored}");
    assert_eq!(restored["filter_terms"], json!(["lactose free"]));
    assert_eq!(restored["quantity"], 2);
}
