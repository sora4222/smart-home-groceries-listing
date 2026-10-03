//! Choosing a product for a list item, through the real router.
//!
//! `PUT /api/grocery-items/{id}/selection`, `DELETE` on the same path and
//! `GET /api/item-selections`. The stores are the fake catalogue
//! (`STORE_CLIENTS=fake`): an integration test never reaches a real store.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

fn uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/selection")
}

fn choice(store: &str, product_id: &str) -> Value {
    json!({ "store": store, "product_id": product_id })
}

/// Every saved choice, from the endpoint the order screen reads.
async fn selections(app: &TestApp) -> Vec<Value> {
    let (status, body) = app.get("/api/item-selections").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().expect("an array").clone()
}

#[sqlx::test]
async fn choosing_a_product_saves_it_with_the_stores_own_details(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    let (status, body) = app
        .put_json(&uri(&id), &choice("woolworths", "w-milk-2l"))
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["grocery_item_id"], id);
    assert_eq!(body["store"], "woolworths");
    assert_eq!(body["store_name"], "Woolworths");
    assert_eq!(body["product_id"], "w-milk-2l");
    assert_eq!(body["name"], "Full Cream Milk");
    assert_eq!(body["brand"], "Woolworths");
    assert_eq!(body["package_size"], "2L");
    // Prices come from the store's answer, never from the request.
    assert_eq!(body["price"], "3.10");
    assert_eq!(body["unit_price"]["amount"], "0.155");
    assert_eq!(body["unit_price"]["per"], "100mL");
    assert_eq!(body["total_price"], "3.10");
    assert_eq!(body["priced_quantity"], 1);
    assert_eq!(body["selected_by"], "dev-user");
    assert!(body["url"].as_str().is_some());
    assert!(body["selected_at"].as_str().is_some());

    let saved = selections(&app).await;
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0]["product_id"], "w-milk-2l");
}

#[sqlx::test]
async fn the_total_is_priced_for_the_items_quantity_with_deals(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "oat milk", "quantity": 2 }))
        .await;

    let (status, body) = app.put_json(&uri(&id), &choice("coles", "c-oat-1l")).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["price"], "3.90");
    assert_eq!(body["total_price"], "6.50");
    assert_eq!(body["priced_quantity"], 2);
}

#[sqlx::test]
async fn choosing_again_at_the_same_store_replaces_it(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json(&uri(&id), &choice("woolworths", "w-milk-2l"))
        .await;

    let (status, body) = app
        .put_json(&uri(&id), &choice("woolworths", "w-milk-3l"))
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let saved = selections(&app).await;
    assert_eq!(saved.len(), 1, "{saved:?}");
    assert_eq!(saved[0]["product_id"], "w-milk-3l");
    assert_eq!(saved[0]["also_chosen"], json!([]));
}

#[sqlx::test]
async fn each_item_keeps_its_own_choice(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = app.add_item(&json!({ "name": "milk" })).await;
    let bananas = app.add_item(&json!({ "name": "bananas" })).await;

    app.put_json(&uri(&milk), &choice("coles", "c-milk-3l"))
        .await;
    app.put_json(&uri(&bananas), &choice("woolworths", "w-banana"))
        .await;

    let saved = selections(&app).await;
    assert_eq!(saved.len(), 2);
    let for_item = |id: &str| {
        saved
            .iter()
            .find(|s| s["grocery_item_id"] == id)
            .unwrap_or_else(|| panic!("no choice for {id}"))["product_id"]
            .clone()
    };
    assert_eq!(for_item(&milk), "c-milk-3l");
    assert_eq!(for_item(&bananas), "w-banana");
}

#[sqlx::test]
async fn a_product_the_store_does_not_offer_for_the_item_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    // A Coles id asked for at Woolworths, and an id nobody sells.
    for body in [choice("woolworths", "c-milk-3l"), choice("coles", "nope")] {
        let (status, answer) = app.put_json(&uri(&id), &body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{answer}");
    }
    assert!(selections(&app).await.is_empty());
}

#[sqlx::test]
async fn a_product_the_items_chips_exclude_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "toilet paper", "filter_terms": ["3 ply"] }))
        .await;

    let (status, body) = app
        .put_json(&uri(&id), &choice("woolworths", "w-tp-2ply"))
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[sqlx::test]
async fn a_store_that_cannot_be_searched_is_reported_as_unavailable(pool: PgPool) {
    let app = TestApp::new(pool);
    // The fake Coles fails any search containing "outage".
    let id = app.add_item(&json!({ "name": "outage milk" })).await;

    let (status, body) = app.put_json(&uri(&id), &choice("coles", "c-milk-3l")).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    assert!(body["detail"].as_str().unwrap().contains("Coles"), "{body}");
    assert!(selections(&app).await.is_empty());
}

#[sqlx::test]
async fn an_unknown_store_or_a_missing_product_id_is_unprocessable(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    for body in [
        json!({ "store": "aldi", "product_id": "w-milk-2l" }),
        json!({ "store": "woolworths" }),
        json!({ "store": "woolworths", "product_id": "" }),
    ] {
        let (status, answer) = app.put_json(&uri(&id), &body).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{body} → {answer}"
        );
    }
}

#[sqlx::test]
async fn choosing_for_an_unknown_item_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let missing = "00000000-0000-0000-0000-000000000000";

    let (status, _) = app
        .put_json(&uri(missing), &choice("woolworths", "w-milk-2l"))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = app.delete(&uri(missing)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_committed_item_can_still_have_its_product_chosen(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, body) = app
        .put_json(&uri(&id), &choice("woolworths", "w-milk-3l"))
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
}

#[sqlx::test]
async fn clearing_a_choice_removes_it_and_is_harmless_twice(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;
    app.put_json(&uri(&id), &choice("woolworths", "w-milk-2l"))
        .await;

    let (first, _) = app.delete(&uri(&id)).await;
    let (second, _) = app.delete(&uri(&id)).await;

    assert_eq!(first, StatusCode::NO_CONTENT);
    assert_eq!(second, StatusCode::NO_CONTENT);
    assert!(selections(&app).await.is_empty());
}

#[sqlx::test]
async fn every_selection_route_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);
    let item = "00000000-0000-0000-0000-000000000000";

    for (method, path) in [
        (Method::GET, "/api/item-selections".to_string()),
        (Method::PUT, uri(item)),
        (Method::DELETE, uri(item)),
    ] {
        let (status, _) = app
            .send(
                Request::builder()
                    .method(method.clone())
                    .uri(&path)
                    .header("content-type", "application/json")
                    .body(Body::from(choice("coles", "c-milk-3l").to_string()))
                    .unwrap(),
            )
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
}
