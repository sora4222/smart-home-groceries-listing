//! What editing or removing a list item does to its chosen product.
//!
//! A product is chosen for what the item is — its name and chips — so a
//! rename or a change of chips drops the choice, while a new quantity or note
//! keeps it. Removing the item removes its choice.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

fn uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/selection")
}

fn choice(store: &str, product_id: &str) -> Value {
    json!({ "store": store, "product_id": product_id })
}

async fn selections(app: &TestApp) -> Vec<Value> {
    let (status, body) = app.get("/api/item-selections").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().expect("an array").clone()
}

#[sqlx::test]
async fn renaming_an_item_clears_its_choice(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json(&uri(&id), &choice("coles", "c-milk-3l")).await;

    let (status, _) = app
        .patch_json(
            &format!("/api/grocery-items/{id}"),
            &json!({ "name": "oat milk" }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(selections(&app).await.is_empty());
}

#[sqlx::test]
async fn changing_an_items_chips_clears_its_choice(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "toilet paper" })).await;
    app.put_json(&uri(&id), &choice("woolworths", "w-tp-2ply"))
        .await;

    app.patch_json(
        &format!("/api/grocery-items/{id}"),
        &json!({ "filter_terms": ["3 ply"] }),
    )
    .await;

    assert!(selections(&app).await.is_empty());
}

#[sqlx::test]
async fn changing_only_the_quantity_or_note_keeps_the_choice(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json(&uri(&id), &choice("coles", "c-milk-3l")).await;

    app.patch_json(
        &format!("/api/grocery-items/{id}"),
        // The same name in another case is not a rename of the product.
        &json!({ "name": "Milk", "quantity": 3, "note": "for the week" }),
    )
    .await;

    let saved = selections(&app).await;
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0]["product_id"], "c-milk-3l");
}

#[sqlx::test]
async fn removing_the_item_removes_its_choice(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json(&uri(&id), &choice("coles", "c-milk-3l")).await;

    let (status, _) = app.delete(&format!("/api/grocery-items/{id}")).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(selections(&app).await.is_empty());
}
