//! Purchase history through the real router: a trolley fill is saved as
//! bought, Undo takes it back, and the product picker reads past purchases.
//!
//! The stores are the fake catalogue: Woolworths "Full Cream Milk 2L"
//! (`w-milk-2l`) is $3.10 and "Complete Clean 3 Ply Toilet Paper"
//! (`w-tp-3ply`) is $12.00.

mod common;

use axum::http::StatusCode;
use common::purchases::{fill_trolley, orders_once_saved};
use common::trolley::chosen_item;
use common::{fake_store_settings, TestApp};
use grocery_backend::services::{purchases, stores};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

async fn list_names(app: &TestApp) -> Vec<String> {
    let (_, items) = app.get("/api/grocery-items").await;
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap().to_string())
        .collect()
}

async fn milk_history(app: &TestApp) -> Value {
    let (status, body) = app
        .post_json(
            "/api/purchase-history/products",
            &json!({ "products": [
                { "store": "woolworths", "product_id": "w-milk-2l" },
                { "store": "coles", "product_id": "never-bought" }
            ] }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn a_filled_trolley_is_saved_as_bought_and_leaves_the_list(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "toilet paper", 1, "woolworths", "w-tp-3ply").await;

    let handoff = fill_trolley(
        &app,
        json!([
            { "product_id": "w-milk-2l", "outcome": "added" },
            { "product_id": "w-tp-3ply", "outcome": "failed", "problem": "Sold out" }
        ]),
        "15",
    )
    .await;
    let orders = orders_once_saved(&app, 1).await;

    let order = &orders[0];
    assert_eq!(order["store"], "woolworths");
    assert_eq!(order["trolley_handoff_id"], handoff);
    assert_eq!(order["source"], "trolley_fill");
    assert_eq!(order["items_total"], "6.20");
    assert_eq!(order["delivery_fee"], "15");
    assert_eq!(order["total"], "21.20");
    assert_eq!(order["products"], 1);
    // Bought items leave the list; the one that failed stays to be bought.
    assert_eq!(list_names(&app).await, vec!["toilet paper"]);
}

#[sqlx::test]
async fn the_picker_sees_how_often_a_product_was_bought_and_its_prices(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "0",
    )
    .await;
    orders_once_saved(&app, 1).await;

    let body = milk_history(&app).await;

    let history = body.as_array().unwrap();
    assert_eq!(
        history.len(),
        1,
        "a product never bought is left out: {body}"
    );
    assert_eq!(history[0]["store"], "woolworths");
    assert_eq!(history[0]["product_id"], "w-milk-2l");
    assert_eq!(history[0]["times_bought"], 1);
    assert_eq!(history[0]["purchases"][0]["unit_price"], "3.10");
    assert_eq!(history[0]["purchases"][0]["quantity"], 2);
}

#[sqlx::test]
async fn an_empty_lookup_is_an_empty_answer(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, body) = app
        .post_json("/api/purchase-history/products", &json!({ "products": [] }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[sqlx::test]
async fn undo_forgets_the_shop_and_puts_items_back_as_they_were(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    app.post("/api/grocery-items/commit").await;
    fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "15",
    )
    .await;
    let orders = orders_once_saved(&app, 1).await;
    let order_id = orders[0]["id"].as_str().unwrap();

    let (status, body) = app
        .delete(&format!("/api/purchase-orders/{order_id}"))
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["items_restored"], 1);
    let (_, items) = app.get("/api/grocery-items").await;
    assert_eq!(items[0]["name"], "milk");
    assert_eq!(items[0]["status"], "committed", "back to how it was");
    let (_, orders) = app.get("/api/purchase-orders").await;
    assert_eq!(orders, json!([]));
    assert_eq!(milk_history(&app).await, json!([]));
}

#[sqlx::test]
async fn undoing_twice_is_a_404(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "0",
    )
    .await;
    let orders = orders_once_saved(&app, 1).await;
    let uri = format!("/api/purchase-orders/{}", orders[0]["id"].as_str().unwrap());

    app.delete(&uri).await;
    let (status, _) = app.delete(&uri).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn undo_keeps_an_item_deleted_since_deleted(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "0",
    )
    .await;
    let orders = orders_once_saved(&app, 1).await;
    sqlx::query("DELETE FROM grocery_items WHERE id = $1")
        .bind(Uuid::parse_str(&milk).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = app
        .delete(&format!(
            "/api/purchase-orders/{}",
            orders[0]["id"].as_str().unwrap()
        ))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items_restored"], 0);
    assert!(list_names(&app).await.is_empty());
}

#[sqlx::test]
async fn a_handoff_is_never_saved_twice(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let handoff = fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "0",
    )
    .await;
    orders_once_saved(&app, 1).await;
    let stores = stores::registry::build(&fake_store_settings());

    let again = purchases::record(&pool, &stores, Uuid::parse_str(&handoff).unwrap())
        .await
        .unwrap();

    assert!(again.is_none());
    let (_, orders) = app.get("/api/purchase-orders").await;
    assert_eq!(orders.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn a_fill_where_nothing_went_in_saves_nothing(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let handoff = fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "failed", "problem": "Sold out" }]),
        "0",
    )
    .await;
    let stores = stores::registry::build(&fake_store_settings());

    let saved = purchases::record(&pool, &stores, Uuid::parse_str(&handoff).unwrap())
        .await
        .unwrap();

    assert!(saved.is_none());
    assert_eq!(list_names(&app).await, vec!["milk"]);
}

#[sqlx::test]
async fn recategorise_applies_todays_keywords(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    fill_trolley(
        &app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "0",
    )
    .await;
    orders_once_saved(&app, 1).await;
    sqlx::query("UPDATE purchases SET category = 'Other', category_source = 'none'")
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = app.post("/api/purchase-history/recategorise").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["changed"], 1);
    let (_, spending) = app.get("/api/spending").await;
    assert_eq!(spending["categories"], json!(["Dairy & eggs"]));
}

#[sqlx::test]
async fn every_purchase_route_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let id = Uuid::new_v4();

    let (orders, _) = app.get("/api/purchase-orders").await;
    let (undo, _) = app.delete(&format!("/api/purchase-orders/{id}")).await;
    let (lookup, _) = app
        .post_json("/api/purchase-history/products", &json!({ "products": [] }))
        .await;
    let (recategorise, _) = app.post("/api/purchase-history/recategorise").await;

    for status in [orders, undo, lookup, recategorise] {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
