//! An item's choices at both stores, and which one the order buys.
//!
//! An item may keep one product per store. The newest choice is the one the
//! order buys until `PUT /api/order-stores` switches it. Through the real
//! router over the fake catalogue (`STORE_CLIENTS=fake`).

mod common;

use axum::http::StatusCode;
use common::trolley::chosen_item;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

fn uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/selection")
}

async fn choose(app: &TestApp, item_id: &str, store: &str, product_id: &str) {
    let (status, body) = app
        .put_json(
            &uri(item_id),
            &json!({ "store": store, "product_id": product_id }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn selections(app: &TestApp) -> Vec<Value> {
    let (status, body) = app.get("/api/item-selections").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().expect("an array").clone()
}

async fn set_order_stores(app: &TestApp, picks: Value) -> (StatusCode, Value) {
    app.put_json("/api/order-stores", &json!({ "picks": picks }))
        .await
}

#[sqlx::test]
async fn choosing_at_the_other_store_keeps_the_first_choice_beside_it(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;

    choose(&app, &milk, "coles", "c-milk-3l").await;

    let saved = selections(&app).await;
    assert_eq!(saved.len(), 1, "{saved:?}");
    // The newest choice is the one the order buys.
    assert_eq!(saved[0]["store"], "coles");
    assert_eq!(saved[0]["product_id"], "c-milk-3l");
    let also = saved[0]["also_chosen"].as_array().unwrap();
    assert_eq!(also.len(), 1);
    assert_eq!(also[0]["store"], "woolworths");
    assert_eq!(also[0]["product_id"], "w-milk-2l");
}

#[sqlx::test]
async fn the_order_can_switch_an_item_to_its_other_stores_choice(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;

    let (status, body) = set_order_stores(
        &app,
        json!([{ "grocery_item_id": milk, "store": "woolworths" }]),
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let saved = selections(&app).await;
    assert_eq!(saved[0]["store"], "woolworths");
    assert_eq!(saved[0]["also_chosen"][0]["store"], "coles");
}

#[sqlx::test]
async fn switching_to_a_store_with_no_choice_changes_nothing(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;
    let bananas = chosen_item(&app, "bananas", 1, "woolworths", "w-banana").await;

    // Milk could move, bananas cannot: the whole request is refused.
    let (status, body) = set_order_stores(
        &app,
        json!([
            { "grocery_item_id": milk, "store": "woolworths" },
            { "grocery_item_id": bananas, "store": "coles" },
        ]),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(body["detail"].as_str().unwrap().contains("Coles"), "{body}");
    let milk_choice = selections(&app)
        .await
        .into_iter()
        .find(|s| s["grocery_item_id"] == milk)
        .unwrap();
    assert_eq!(milk_choice["store"], "coles");
}

#[sqlx::test]
async fn an_empty_or_malformed_switch_is_unprocessable(pool: PgPool) {
    let app = TestApp::new(pool);

    let (empty, _) = set_order_stores(&app, json!([])).await;
    let (bad_store, _) = set_order_stores(
        &app,
        json!([{ "grocery_item_id": "00000000-0000-0000-0000-000000000000", "store": "aldi" }]),
    )
    .await;

    assert_eq!(empty, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad_store, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn clearing_the_choice_to_buy_hands_over_to_the_other_store(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;

    let (status, _) = app.delete(&format!("{}?store=coles", uri(&milk))).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    let saved = selections(&app).await;
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0]["store"], "woolworths");
    assert_eq!(saved[0]["also_chosen"], json!([]));
}

#[sqlx::test]
async fn clearing_the_other_stores_choice_keeps_the_one_to_buy(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;

    app.delete(&format!("{}?store=woolworths", uri(&milk)))
        .await;

    let saved = selections(&app).await;
    assert_eq!(saved[0]["store"], "coles");
    assert_eq!(saved[0]["also_chosen"], json!([]));
}

#[sqlx::test]
async fn clearing_with_no_store_clears_both(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;

    let (status, _) = app.delete(&uri(&milk)).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(selections(&app).await.is_empty());
}

#[sqlx::test]
async fn the_order_review_and_the_trolley_use_only_the_choice_to_buy(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    choose(&app, &milk, "coles", "c-milk-3l").await;
    app.post("/api/grocery-items/commit").await;

    let (_, review) = app.get("/api/order-review").await;
    let stores = review["stores"].as_array().unwrap();
    assert_eq!(stores.len(), 1, "{review}");
    assert_eq!(stores[0]["store"], "coles");

    // Nothing at Woolworths is for the order, so there is nothing to send.
    let (status, _) = app
        .post_json("/api/trolley-handoffs", &json!({ "store": "woolworths" }))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn switching_stores_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (status, _) = set_order_stores(
        &app,
        json!([{ "grocery_item_id": "00000000-0000-0000-0000-000000000000", "store": "coles" }]),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
