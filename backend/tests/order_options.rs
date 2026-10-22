//! The order planner through the real router: `GET /api/order-options`, and
//! using an option with `PUT /api/order-stores`.
//!
//! The stores are the fake catalogue (`STORE_CLIENTS=fake`): milk is $3.10
//! (2L) at Woolworths and $4.95 (3L) at Coles; bananas $0.80 and $0.83.

mod common;

use axum::http::StatusCode;
use common::trolley::chosen_item;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

async fn options(app: &TestApp, query: &str) -> Value {
    let (status, body) = app.get(&format!("/api/order-options{query}")).await;
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

async fn set_fees(app: &TestApp, fee: &str, mode: &str) {
    let rules = |store| json!({ "store": store, "delivery_fee": fee });
    let (status, body) = app
        .put_json(
            "/api/delivery-settings",
            &json!({ "stores": [rules("woolworths"), rules("coles")], "mode": mode }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// Milk and bananas, each chosen at Woolworths and then at Coles (so the
/// order buys both at Coles now), committed.
async fn both_stores_order(app: &TestApp) -> (String, String) {
    let milk = chosen_item(app, "milk", 1, "woolworths", "w-milk-2l").await;
    let bananas = chosen_item(app, "bananas", 1, "woolworths", "w-banana").await;
    choose(app, &milk, "coles", "c-milk-3l").await;
    choose(app, &bananas, "coles", "c-banana").await;
    app.post("/api/grocery-items/commit").await;
    (milk, bananas)
}

fn option<'a>(body: &'a Value, kind: &str) -> &'a Value {
    body["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["kind"] == kind)
        .unwrap_or_else(|| panic!("no {kind} option in {body}"))
}

#[sqlx::test]
async fn nothing_committed_has_no_options(pool: PgPool) {
    let app = TestApp::new(pool);

    let body = options(&app, "").await;

    assert_eq!(body["options"], json!([]));
    assert_eq!(body["unchosen"], json!([]));
    assert_eq!(body["mode"], "minimise_total");
}

#[sqlx::test]
async fn the_cheapest_store_with_delivery_is_recommended(pool: PgPool) {
    let app = TestApp::new(pool);
    both_stores_order(&app).await;
    set_fees(&app, "9", "minimise_total").await;

    let body = options(&app, "").await;

    let best = &body["options"][0];
    assert_eq!(best["kind"], "woolworths", "{body}");
    assert_eq!(best["label"], "All at Woolworths");
    assert_eq!(best["recommended"], true);
    assert_eq!(best["items_total"], "3.90");
    assert_eq!(best["delivery_total"], "9");
    assert_eq!(best["total"], "12.90");
    assert_eq!(best["fees_known"], true);
    assert_eq!(best["stores"][0]["lines"].as_array().unwrap().len(), 2);
    // The order buys both at Coles now.
    let coles = option(&body, "coles");
    assert_eq!(coles["is_current"], true);
    assert_eq!(coles["recommended"], false);
    assert_eq!(coles["total"], "14.78");
    // A split costs two deliveries, so it is never cheaper here and is
    // either absent or ranked after.
    assert_eq!(body["exact"], true);
}

#[sqlx::test]
async fn using_an_option_switches_the_stores_the_order_buys_from(pool: PgPool) {
    let app = TestApp::new(pool);
    both_stores_order(&app).await;
    set_fees(&app, "9", "minimise_total").await;
    let body = options(&app, "").await;
    let picks = body["options"][0]["picks"].clone();

    let (status, answer) = app
        .put_json("/api/order-stores", &json!({ "picks": picks }))
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "{answer}");
    let after = options(&app, "").await;
    assert_eq!(after["options"][0]["is_current"], true);
    let (_, review) = app.get("/api/order-review").await;
    assert_eq!(review["stores"][0]["store"], "woolworths");
    assert_eq!(review["stores"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn a_mode_in_the_query_overrides_the_saved_one(pool: PgPool) {
    let app = TestApp::new(pool);
    both_stores_order(&app).await;
    set_fees(&app, "9", "minimise_total").await;

    let body = options(&app, "?mode=coles_only").await;

    assert_eq!(body["mode"], "coles_only");
    assert_eq!(body["options"][0]["kind"], "coles");
    assert_eq!(body["options"][0]["recommended"], true);
}

#[sqlx::test]
async fn unset_fees_and_unchosen_items_are_reported(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    app.add_item(&json!({ "name": "bread" })).await;
    app.post("/api/grocery-items/commit").await;

    let body = options(&app, "").await;

    let best = &body["options"][0];
    assert_eq!(best["fees_known"], false);
    assert_eq!(best["stores"][0]["fee_known"], false);
    assert_eq!(best["delivery_total"], "0");
    assert_eq!(body["unchosen"][0]["name"], "bread");
}

#[sqlx::test]
async fn an_unknown_mode_is_unprocessable(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.get("/api/order-options?mode=cheapest_ever").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn order_options_need_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (status, _) = app.get("/api/order-options").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
