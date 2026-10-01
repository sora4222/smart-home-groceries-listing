//! Handing chosen products to a store's trolley, through the real router.
//!
//! The web app's side (`/api/trolley-handoffs`) and the store tab's side
//! (`/api/store-tab/trolley-handoffs`, the bookmarklet on the store's
//! website). The stores are the fake catalogue; nothing reaches a real store.

mod common;

use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use common::{TestApp, TEST_STORE_TAB_SECRET};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The Woolworths "website" in the test settings (`fake_store_settings`).
const WOOLWORTHS_ORIGIN: &str = "http://127.0.0.1:9";

/// Adds an item and chooses a product for it.
async fn chosen_item(
    app: &TestApp,
    name: &str,
    quantity: i64,
    store: &str,
    product_id: &str,
) -> String {
    let id = app
        .add_item(&json!({ "name": name, "quantity": quantity }))
        .await;
    let (status, body) = app
        .put_json(
            &format!("/api/grocery-items/{id}/selection"),
            &json!({ "store": store, "product_id": product_id }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    id
}

/// A store-tab request: JSON sent as `text/plain` from the store's origin.
async fn store_tab_post(app: &TestApp, uri: &str, body: &Value) -> (StatusCode, Value, HeaderMap) {
    app.send_with_headers(
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("origin", WOOLWORTHS_ORIGIN)
            .header("content-type", "text/plain;charset=UTF-8")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
}

async fn claim(app: &TestApp) -> (StatusCode, Value, HeaderMap) {
    store_tab_post(
        app,
        "/api/store-tab/trolley-handoffs/claim",
        &json!({ "secret": TEST_STORE_TAB_SECRET, "store": "woolworths" }),
    )
    .await
}

async fn create(app: &TestApp, store: &str) -> (StatusCode, Value) {
    app.post_json("/api/trolley-handoffs", &json!({ "store": store }))
        .await
}

#[sqlx::test]
async fn a_handoff_holds_every_item_chosen_at_that_store(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "oat milk", 1, "coles", "c-oat-1l").await;
    app.add_item(&json!({ "name": "bread" })).await; // nothing chosen

    let (status, body) = create(&app, "woolworths").await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["store"], "woolworths");
    assert_eq!(body["status"], "waiting_for_store_tab");
    let lines = body["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["grocery_item_id"], milk);
    assert_eq!(lines[0]["product_id"], "w-milk-2l");
    assert_eq!(lines[0]["quantity"], 2);
    assert!(lines[0]["outcome"].is_null());
}

#[sqlx::test]
async fn nothing_chosen_at_the_store_is_a_422(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "oat milk", 1, "coles", "c-oat-1l").await;

    let (status, body) = create(&app, "woolworths").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[sqlx::test]
async fn the_store_tab_claims_the_newest_handoff_once(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let (_, older) = create(&app, "woolworths").await;
    let (_, newer) = create(&app, "woolworths").await;

    let (status, body, headers) = claim(&app).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["handoff_id"], newer["id"]);
    assert_eq!(
        body["lines"][0],
        json!({ "product_id": "w-milk-2l", "name": "Full Cream Milk", "quantity": 1 })
    );
    assert_eq!(headers["access-control-allow-origin"], WOOLWORTHS_ORIGIN);

    let (status, _, _) = claim(&app).await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "a claimed handoff is not handed out twice"
    );

    let (_, older) = app
        .get(&format!(
            "/api/trolley-handoffs/{}",
            older["id"].as_str().unwrap()
        ))
        .await;
    assert_eq!(older["status"], "replaced");
}

#[sqlx::test]
async fn one_product_chosen_for_two_items_is_claimed_as_one_line(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "full cream milk", 3, "woolworths", "w-milk-2l").await;
    create(&app, "woolworths").await;

    let (_, body, _) = claim(&app).await;

    assert_eq!(body["lines"].as_array().unwrap().len(), 1);
    assert_eq!(body["lines"][0]["quantity"], 5);
}

#[sqlx::test]
async fn the_report_is_saved_and_the_web_app_sees_it(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "toilet paper", 1, "woolworths", "w-tp-3ply").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();

    let (status, body, headers) = store_tab_post(
        &app,
        &format!("/api/store-tab/trolley-handoffs/{id}/report"),
        &json!({ "secret": TEST_STORE_TAB_SECRET, "lines": [
            { "product_id": "w-milk-2l", "outcome": "added" },
            { "product_id": "w-tp-3ply", "outcome": "failed", "problem": "Out of stock" }
        ]}),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(headers["access-control-allow-origin"], WOOLWORTHS_ORIGIN);
    let (_, seen) = app.get(&format!("/api/trolley-handoffs/{id}")).await;
    assert_eq!(seen["status"], "filled_with_problems");
    assert_eq!(seen["lines"][0]["outcome"], "added");
    assert_eq!(seen["lines"][1]["outcome"], "failed");
    assert_eq!(seen["lines"][1]["problem"], "Out of stock");
    assert!(seen["reported_at"].as_str().is_some());
}

#[sqlx::test]
async fn an_incomplete_report_or_a_second_report_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "toilet paper", 1, "woolworths", "w-tp-3ply").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let uri = format!(
        "/api/store-tab/trolley-handoffs/{}/report",
        claimed["handoff_id"].as_str().unwrap()
    );

    let only_milk = json!({ "secret": TEST_STORE_TAB_SECRET, "lines": [{ "product_id": "w-milk-2l", "outcome": "added" }] });
    let (status, _, _) = store_tab_post(&app, &uri, &only_milk).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let both = json!({ "secret": TEST_STORE_TAB_SECRET, "lines": [
        { "product_id": "w-milk-2l", "outcome": "added" },
        { "product_id": "w-tp-3ply", "outcome": "added" }
    ]});
    let (status, _, _) = store_tab_post(&app, &uri, &both).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _, _) = store_tab_post(&app, &uri, &both).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn a_wrong_secret_is_refused_and_other_origins_are_not_allowed(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _, headers) = store_tab_post(
        &app,
        "/api/store-tab/trolley-handoffs/claim",
        &json!({ "secret": "wrong", "store": "woolworths" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // The bookmarklet can still read the refusal.
    assert_eq!(headers["access-control-allow-origin"], WOOLWORTHS_ORIGIN);

    let (_, _, headers) = app
        .send_with_headers(
            Request::builder()
                .method(Method::POST)
                .uri("/api/store-tab/trolley-handoffs/claim")
                .header("origin", "https://evil.example")
                .body(Body::from(
                    json!({ "secret": "x", "store": "woolworths" }).to_string(),
                ))
                .unwrap(),
        )
        .await;
    assert!(headers.get("access-control-allow-origin").is_none());
}

#[sqlx::test]
async fn the_web_app_routes_need_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (status, _) = create(&app, "woolworths").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app.get("/api/trolley-handoffs/store-tab-secret").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn the_web_app_reads_the_store_tab_secret_to_build_the_bookmarklet(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get("/api/trolley-handoffs/store-tab-secret").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["secret"], TEST_STORE_TAB_SECRET);
}
