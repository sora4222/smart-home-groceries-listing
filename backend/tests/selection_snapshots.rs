//! Undo for an item's chosen products, through the real router.
//!
//! Before a change the web app saves the item's products as a snapshot.
//! Undo puts that snapshot back in one step: every product, count and
//! store, with the details saved then, and no store search.

mod common;

use axum::http::StatusCode;
use common::trolley::chosen_item;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

fn selection(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/selection")
}

async fn snapshot(app: &TestApp, item_id: &str) -> String {
    let (status, body) = app.post(&format!("{}/snapshot", selection(item_id))).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_string()
}

async fn restore(app: &TestApp, snapshot_id: &str) -> (StatusCode, Value) {
    app.post(&format!("/api/selection-snapshots/{snapshot_id}/restore"))
        .await
}

async fn choose(app: &TestApp, item_id: &str, store: &str, product_id: &str) {
    let (status, body) = app
        .put_json(
            &selection(item_id),
            &json!({ "store": store, "product_id": product_id }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn choices(app: &TestApp) -> Value {
    let (status, body) = app.get("/api/item-selections").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn undo_puts_back_counts_and_both_stores(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 3, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;
    choose(&app, &milk, "woolworths", "w-milk-3l").await;
    let before = choices(&app).await;
    let saved = snapshot(&app, &milk).await;

    app.put_json(
        &format!("{}/quantity", selection(&milk)),
        &json!({ "store": "woolworths", "product_id": "w-milk-3l", "quantity": 2 }),
    )
    .await;
    app.delete(&format!("{}?store=coles", selection(&milk)))
        .await;
    let (status, body) = restore(&app, &saved).await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(choices(&app).await, before);
}

#[sqlx::test]
async fn undo_of_a_first_choice_clears_it(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    let saved = snapshot(&app, &milk).await;
    choose(&app, &milk, "woolworths", "w-milk-2l").await;

    let (status, body) = restore(&app, &saved).await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(choices(&app).await, json!([]));
}

#[sqlx::test]
async fn undo_of_a_removal_puts_the_products_on_the_restored_item(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    let saved = snapshot(&app, &milk).await;
    app.delete(&format!("/api/grocery-items/{milk}")).await;
    let (_, back) = app
        .post_json(
            "/api/grocery-items/restore",
            &json!({ "name": "milk", "quantity": 2 }),
        )
        .await;
    let back = back["id"].as_str().unwrap();

    let (status, body) = app
        .post(&format!(
            "/api/selection-snapshots/{saved}/restore?grocery_item_id={back}"
        ))
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let now = choices(&app).await;
    assert_eq!(now[0]["grocery_item_id"], back);
    assert_eq!(now[0]["product_id"], "w-milk-2l");
}

#[sqlx::test]
async fn an_unknown_snapshot_or_item_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let missing = uuid::Uuid::new_v4();

    let (status, _) = restore(&app, &missing.to_string()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = app
        .post(&format!("{}/snapshot", selection(&missing.to_string())))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
