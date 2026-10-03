//! Integration tests for removing and reducing list items by Alexa.
//!
//! The bridge sidecar forwards "remove milk" or "remove two milk" to
//! `POST /api/intake/alexa/remove`. These check which item is matched, what
//! happens to it, and that retries and locked lists are handled.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{TestApp, TEST_BRIDGE_SECRET};
use grocery_backend::routes::alexa::BRIDGE_SECRET_HEADER;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// `POST /api/intake/alexa/remove` with `secret` as the bridge secret.
async fn remove_with(app: &TestApp, body: &Value, secret: Option<&str>) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/api/intake/alexa/remove")
        .header("content-type", "application/json");
    if let Some(secret) = secret {
        request = request.header(BRIDGE_SECRET_HEADER, secret);
    }
    app.send(request.body(Body::from(body.to_string())).unwrap())
        .await
}

/// `POST /api/intake/alexa/remove` as the bridge sidecar.
async fn remove(app: &TestApp, body: &Value) -> (StatusCode, Value) {
    remove_with(app, body, Some(TEST_BRIDGE_SECRET)).await
}

fn request_id() -> String {
    format!("amzn1.echo-api.request.{}", Uuid::new_v4())
}

/// The list as `(name, quantity)` pairs.
async fn listed(app: &TestApp) -> Vec<(String, i64)> {
    let (_, items) = app.get("/api/grocery-items").await;
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["name"].as_str().unwrap().to_string(),
                item["quantity"].as_i64().unwrap(),
            )
        })
        .collect()
}

#[sqlx::test]
async fn remove_takes_the_whole_item_off_the_list(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "Milk", "quantity": 3 }))
        .await;

    let (status, body) = remove(
        &app,
        &json!({ "item": "milk", "external_id": request_id() }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["kind"], "removed");
    assert_eq!(body["item_name"], "Milk");
    assert_eq!(body["quantity_before"], 3);
    assert_eq!(body["quantity_after"], 0);
    assert_eq!(body["undone"], false);
    assert!(listed(&app).await.is_empty());
}

#[sqlx::test]
async fn a_quantity_lowers_the_item_and_keeps_it(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "eggs", "quantity": 3 }))
        .await;

    let (status, body) = remove(&app, &json!({ "item": "eggs", "quantity": 2 })).await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["kind"], "reduced");
    assert_eq!(body["quantity_after"], 1);
    assert_eq!(listed(&app).await, vec![("eggs".to_string(), 1)]);
}

#[sqlx::test]
async fn reducing_to_zero_or_below_removes_the_item(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "bread", "quantity": 2 }))
        .await;

    let (_, body) = remove(&app, &json!({ "item": "bread", "quantity": 5 })).await;

    assert_eq!(body["kind"], "removed");
    assert!(listed(&app).await.is_empty());
}

#[sqlx::test]
async fn a_plural_matches_a_singular_item(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "Banana", "quantity": 6 }))
        .await;

    let (status, body) = remove(&app, &json!({ "item": "bananas", "quantity": 1 })).await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(listed(&app).await, vec![("Banana".to_string(), 5)]);
}

#[sqlx::test]
async fn an_exact_name_wins_over_a_plural(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "egg", "quantity": 1 })).await;
    app.add_item(&json!({ "name": "eggs", "quantity": 12 }))
        .await;

    remove(&app, &json!({ "item": "eggs" })).await;

    assert_eq!(listed(&app).await, vec![("egg".to_string(), 1)]);
}

#[sqlx::test]
async fn an_unknown_item_is_a_404_and_changes_nothing(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "oat milk" })).await;

    // "milk" is part of "oat milk" but not the same item: no guessing.
    let (status, body) = remove(&app, &json!({ "item": "milk" })).await;

    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(listed(&app).await, vec![("oat milk".to_string(), 1)]);
}

#[sqlx::test]
async fn a_committed_item_is_locked(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, body) = remove(&app, &json!({ "item": "rice" })).await;

    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(listed(&app).await, vec![("rice".to_string(), 1)]);
}

#[sqlx::test]
async fn a_pending_request_is_not_on_the_list_yet(pool: PgPool) {
    // Only accepted items can be removed; a pending card is the web app's to
    // reject.
    let app = TestApp::new(pool);
    app.create_pending("butter", 1).await;

    let (status, _) = remove(&app, &json!({ "item": "butter" })).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_retried_delivery_changes_the_list_once(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "apples", "quantity": 5 }))
        .await;
    let body = json!({ "item": "apples", "quantity": 1, "external_id": request_id() });

    let (first_status, first) = remove(&app, &body).await;
    let (retry_status, retry) = remove(&app, &body).await;

    assert_eq!(first_status, StatusCode::CREATED);
    assert_eq!(retry_status, StatusCode::OK, "a retry is not a new change");
    assert_eq!(first["id"], retry["id"]);
    assert_eq!(listed(&app).await, vec![("apples".to_string(), 4)]);
}

#[sqlx::test]
async fn requires_the_bridge_secret(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice" })).await;
    let body = json!({ "item": "rice" });

    let (missing, _) = remove_with(&app, &body, None).await;
    let (wrong, _) = remove_with(&app, &body, Some("not-the-secret")).await;
    let (webhook, _) = remove_with(&app, &body, Some(common::TEST_WEBHOOK_SECRET)).await;

    assert_eq!(missing, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong, StatusCode::UNAUTHORIZED);
    assert_eq!(webhook, StatusCode::UNAUTHORIZED);
    assert_eq!(listed(&app).await, vec![("rice".to_string(), 1)]);
}

#[sqlx::test]
async fn a_zero_quantity_is_rejected(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "rice" })).await;

    let (status, _) = remove(&app, &json!({ "item": "rice", "quantity": 0 })).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
