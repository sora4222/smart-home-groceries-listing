//! The order review, through the real router: `GET /api/order-review`.
//!
//! The stores are the fake catalogue (`STORE_CLIENTS=fake`). A choice the
//! catalogue cannot produce (an old price, a product it no longer sells) is
//! written straight into `item_selections`, the way time would leave it.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

async fn review(app: &TestApp) -> Value {
    let (status, body) = app.get("/api/order-review").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

async fn choose(app: &TestApp, item_id: &str, store: &str, product_id: &str) {
    let (status, body) = app
        .put_json(
            &format!("/api/grocery-items/{item_id}/selection"),
            &json!({ "store": store, "product_id": product_id }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// Saves a choice as if it had been made long ago, at `price`.
async fn old_choice(pool: &PgPool, item_id: &str, store: &str, product_id: &str, price: &str) {
    sqlx::query(
        "INSERT INTO item_selections (id, grocery_item_id, store, product_id, product_name,
                                      price, total_price, priced_quantity, url, selected_by)
         VALUES ($1, $2, $3, $4, 'Old product', $5::numeric, $5::numeric, 1, '', 'dev-user')",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::parse_str(item_id).unwrap())
    .bind(store)
    .bind(product_id)
    .bind(price)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn nothing_committed_is_an_empty_order(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    choose(&app, &milk, "coles", "c-milk-3l").await;

    let body = review(&app).await;

    // Active items are still being reviewed on the list: not in the order.
    assert_eq!(body["stores"], json!([]));
    assert_eq!(body["unchosen"], json!([]));
    assert_eq!(body["total"], "0");
    assert_eq!(body["complete"], true);
}

#[sqlx::test]
async fn committed_choices_are_grouped_by_store_with_subtotals(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    let bananas = app
        .add_item(&json!({ "name": "bananas", "quantity": 5 }))
        .await;
    let oat = app.add_item(&json!({ "name": "oat milk" })).await;
    choose(&app, &milk, "coles", "c-milk-3l").await;
    choose(&app, &bananas, "woolworths", "w-banana").await;
    choose(&app, &oat, "coles", "c-oat-1l").await;
    app.post("/api/grocery-items/commit").await;

    let body = review(&app).await;

    let stores = body["stores"].as_array().unwrap();
    assert_eq!(stores.len(), 2, "{body}");
    assert_eq!(stores[0]["store"], "woolworths");
    assert_eq!(stores[0]["store_name"], "Woolworths");
    assert_eq!(stores[0]["subtotal"], "4.00");
    assert_eq!(stores[1]["store"], "coles");
    assert_eq!(stores[1]["lines"].as_array().unwrap().len(), 2);
    assert_eq!(stores[1]["subtotal"], "8.85");
    assert_eq!(body["total"], "12.85");
    assert_eq!(body["complete"], true);

    let line = &stores[0]["lines"][0];
    assert_eq!(line["grocery_item_id"], bananas);
    assert_eq!(line["item_name"], "bananas");
    assert_eq!(line["quantity"], 5);
    assert_eq!(line["product_id"], "w-banana");
    assert_eq!(line["status"], "priced");
    assert_eq!(line["price"], "0.80");
    assert_eq!(line["price_change"], "same");
    assert_eq!(line["problem"], Value::Null);
}

#[sqlx::test]
async fn the_items_current_quantity_is_priced_not_the_chosen_one(pool: PgPool) {
    let app = TestApp::new(pool);
    let oat = app.add_item(&json!({ "name": "oat milk" })).await;
    choose(&app, &oat, "coles", "c-oat-1l").await;
    app.patch_json(
        &format!("/api/grocery-items/{oat}"),
        &json!({ "quantity": 2 }),
    )
    .await;
    app.post("/api/grocery-items/commit").await;

    let line = review(&app).await["stores"][0]["lines"][0].clone();

    assert_eq!(line["quantity"], 2);
    assert_eq!(line["chosen_priced_quantity"], 1);
    assert_eq!(line["chosen_total_price"], "3.90");
    // 2 for $6.50 is reached at the new quantity.
    assert_eq!(line["total_price"], "6.50");
    assert_eq!(line["deal_applied"], true);
}

#[sqlx::test]
async fn items_with_no_product_are_listed_and_the_order_is_incomplete(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    let bread = app
        .add_item(&json!({ "name": "bread", "quantity": 2 }))
        .await;
    choose(&app, &milk, "coles", "c-milk-3l").await;
    app.post("/api/grocery-items/commit").await;

    let body = review(&app).await;

    assert_eq!(
        body["unchosen"],
        json!([{ "grocery_item_id": bread, "name": "bread", "quantity": 2 }])
    );
    assert_eq!(body["total"], "4.95");
    assert_eq!(body["complete"], false);
}

#[sqlx::test]
async fn a_price_change_since_choosing_is_shown(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    old_choice(&pool, &milk, "woolworths", "w-milk-2l", "2.80").await;
    app.post("/api/grocery-items/commit").await;

    let line = review(&app).await["stores"][0]["lines"][0].clone();

    assert_eq!(line["chosen_price"], "2.80");
    assert_eq!(line["price"], "3.10");
    assert_eq!(line["price_change"], "up");
}

#[sqlx::test]
async fn a_product_the_store_stopped_selling_needs_a_new_choice(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    let bananas = app.add_item(&json!({ "name": "bananas" })).await;
    old_choice(&pool, &milk, "woolworths", "w-milk-retired", "2.80").await;
    choose(&app, &bananas, "woolworths", "w-banana").await;
    app.post("/api/grocery-items/commit").await;

    let body = review(&app).await;

    let store = &body["stores"][0];
    let gone = store["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|line| line["product_id"] == "w-milk-retired")
        .unwrap();
    assert_eq!(gone["status"], "not_offered");
    assert_eq!(gone["total_price"], Value::Null);
    assert!(gone["problem"]
        .as_str()
        .unwrap()
        .contains("no longer offers"));
    assert_eq!(store["subtotal"], "0.80");
    assert_eq!(store["complete"], false);
    assert_eq!(body["complete"], false);
}

#[sqlx::test]
async fn a_store_that_cannot_be_asked_keeps_the_choice_without_a_price(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    // The fake Coles fails any search containing "outage".
    let milk = app.add_item(&json!({ "name": "outage milk" })).await;
    old_choice(&pool, &milk, "coles", "c-milk-3l", "4.95").await;
    app.post("/api/grocery-items/commit").await;

    let line = review(&app).await["stores"][0]["lines"][0].clone();

    assert_eq!(line["status"], "store_failed");
    assert_eq!(line["product_name"], "Old product");
    assert!(
        line["problem"].as_str().unwrap().contains("Coles"),
        "{line}"
    );
    // Nothing about the upstream failure leaks.
    assert!(!line["problem"].as_str().unwrap().contains("fake outage"));
}

#[sqlx::test]
async fn the_review_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (status, _) = app.get("/api/order-review").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
