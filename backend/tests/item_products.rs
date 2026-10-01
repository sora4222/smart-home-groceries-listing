//! `GET /api/grocery-items/{id}/products` through the real router.
//!
//! The stores are the fake catalogue (`STORE_CLIENTS=fake`): an integration
//! test never reaches Woolworths or Coles.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

fn uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/products")
}

fn store<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["stores"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["store"] == name)
        .unwrap_or_else(|| panic!("no {name} entry in {body}"))
}

fn product_ids(store: &Value) -> Vec<&str> {
    store["products"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["product_id"].as_str().unwrap())
        .collect()
}

#[sqlx::test]
async fn searches_both_stores_cheapest_per_unit_first(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "Milk" })).await;

    let (status, body) = app.get(&uri(&id)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["query"], "Milk");
    assert_eq!(body["item"]["name"], "Milk");
    let woolworths = store(&body, "woolworths");
    assert_eq!(woolworths["status"], "ok");
    assert_eq!(woolworths["store_name"], "Woolworths");
    // $1.55/L beats $1.80/L even though the 3L is the bigger pack.
    assert_eq!(product_ids(woolworths), ["w-milk-2l", "w-milk-3l"]);
    let first = &woolworths["products"][0];
    assert_eq!(first["price"], "3.10");
    assert_eq!(first["unit_price"]["amount"], "0.155");
    assert_eq!(first["unit_price"]["per"], "100mL");
    assert_eq!(first["unit_price_note"], "unit price calculated from 1L");
    assert_eq!(store(&body, "coles")["status"], "ok");
}

#[sqlx::test]
async fn the_items_chips_narrow_the_results(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "toilet paper", "filter_terms": ["3 ply"] }))
        .await;

    let (status, body) = app.get(&uri(&id)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["query"], "toilet paper 3 ply");
    assert_eq!(product_ids(store(&body, "woolworths")), ["w-tp-3ply"]);
    assert_eq!(product_ids(store(&body, "coles")), ["c-tp-3ply"]);
}

#[sqlx::test]
async fn prices_the_items_quantity_with_deals(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "oat milk", "quantity": 2 }))
        .await;

    let (_, body) = app.get(&uri(&id)).await;

    let coles = store(&body, "coles");
    let oat = coles["products"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["product_id"] == "c-oat-1l")
        .unwrap();
    assert_eq!(oat["total_price"], "6.50");
    assert_eq!(oat["deal_applied"], true);
    assert_eq!(oat["deals"][0]["min_quantity"], 2);
    assert_eq!(oat["deals"][0]["unit_price"], "3.25");
}

#[sqlx::test]
async fn one_store_failing_still_shows_the_other(pool: PgPool) {
    let app = TestApp::new(pool);
    // The fake Coles fails any query containing "outage".
    let id = app.add_item(&json!({ "name": "milk outage" })).await;

    let (status, body) = app.get(&uri(&id)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let coles = store(&body, "coles");
    assert_eq!(coles["status"], "unreachable");
    assert_eq!(
        coles["message"],
        "Coles could not be reached. Try again later."
    );
    assert_eq!(coles["products"], json!([]));
    assert_eq!(store(&body, "woolworths")["status"], "ok");
    assert!(!product_ids(store(&body, "woolworths")).is_empty());
}

#[sqlx::test]
async fn a_committed_item_can_still_be_compared(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "bananas" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, body) = app.get(&uri(&id)).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(product_ids(store(&body, "woolworths")), ["w-banana"]);
}

#[sqlx::test]
async fn an_unknown_item_is_404(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get(&uri("00000000-0000-0000-0000-000000000000")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["detail"], "Grocery item not found");
}

#[sqlx::test]
async fn a_malformed_id_is_rejected(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.get(&uri("not-a-uuid")).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn needs_a_signed_in_household_member(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (status, _) = app
        .send(
            Request::builder()
                .method(Method::GET)
                .uri(uri("00000000-0000-0000-0000-000000000000"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
