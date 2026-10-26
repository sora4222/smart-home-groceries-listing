//! A trolley fill that added some of an item's products but not others: the
//! bought part is saved, and the rest stays on the list to buy.
//!
//! The stores are the fake catalogue: two Woolworths milks, `w-milk-2l` and
//! `w-milk-3l`.

mod common;

use axum::http::StatusCode;
use common::purchases::{fill_trolley, orders_once_saved};
use common::trolley::chosen_item;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

/// Milk wanting 3: two of `w-milk-2l` and one of `w-milk-3l`.
async fn milk_from_two_products(app: &TestApp) -> String {
    let milk = chosen_item(app, "milk", 3, "woolworths", "w-milk-2l").await;
    let (status, body) = app
        .put_json(
            &format!("/api/grocery-items/{milk}/selection"),
            &json!({ "store": "woolworths", "product_id": "w-milk-3l" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    milk
}

async fn fill_only_the_2l(app: &TestApp) -> Value {
    fill_trolley(
        app,
        json!([
            { "product_id": "w-milk-2l", "outcome": "added" },
            { "product_id": "w-milk-3l", "outcome": "failed", "problem": "Sold out" }
        ]),
        "0",
    )
    .await;
    orders_once_saved(app, 1).await[0].clone()
}

async fn milk(app: &TestApp) -> Option<Value> {
    let (_, items) = app.get("/api/grocery-items").await;
    items.as_array().unwrap().first().cloned()
}

#[sqlx::test]
async fn the_product_not_added_stays_on_the_list(pool: PgPool) {
    let app = TestApp::new(pool);
    milk_from_two_products(&app).await;

    let order = fill_only_the_2l(&app).await;

    assert_eq!(order["products"], 1);
    let milk = milk(&app).await.expect("milk stays on the list");
    assert_eq!(milk["status"], "active");
    assert_eq!(milk["quantity"], 1);
    let (_, selections) = app.get("/api/item-selections").await;
    assert_eq!(selections[0]["product_id"], "w-milk-3l");
    assert_eq!(selections[0]["quantity"], 1);
    assert_eq!(selections[0]["bought_with"], json!([]));
}

#[sqlx::test]
async fn undo_puts_the_bought_quantity_back(pool: PgPool) {
    let app = TestApp::new(pool);
    milk_from_two_products(&app).await;
    let order = fill_only_the_2l(&app).await;

    let (status, _) = app
        .delete(&format!(
            "/api/purchase-orders/{}",
            order["id"].as_str().unwrap()
        ))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(milk(&app).await.unwrap()["quantity"], 3);
}
