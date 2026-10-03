//! Helpers for the trolley handoff tests: choosing products, and talking to
//! the API the way the store tab (the bookmarklet) does.

use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use serde_json::{json, Value};

use super::{TestApp, TEST_STORE_TAB_SECRET};

/// The Woolworths "website" in the test settings (`fake_store_settings`).
pub const WOOLWORTHS_ORIGIN: &str = "http://127.0.0.1:9";

/// The Coles "website" in the test settings (`fake_store_settings`).
pub const COLES_ORIGIN: &str = "http://127.0.0.1:8";

/// Adds an item and chooses a product for it.
pub async fn chosen_item(
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

/// A store-tab request: JSON sent as `text/plain` from the Woolworths website.
pub async fn store_tab_post(
    app: &TestApp,
    uri: &str,
    body: &Value,
) -> (StatusCode, Value, HeaderMap) {
    store_tab_post_from(app, WOOLWORTHS_ORIGIN, uri, body).await
}

/// A store-tab request: JSON sent as `text/plain` from `origin`.
pub async fn store_tab_post_from(
    app: &TestApp,
    origin: &str,
    uri: &str,
    body: &Value,
) -> (StatusCode, Value, HeaderMap) {
    app.send_with_headers(
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("origin", origin)
            .header("content-type", "text/plain;charset=UTF-8")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
}

pub async fn claim(app: &TestApp) -> (StatusCode, Value, HeaderMap) {
    store_tab_post(
        app,
        "/api/store-tab/trolley-handoffs/claim",
        &json!({ "secret": TEST_STORE_TAB_SECRET, "store": "woolworths" }),
    )
    .await
}

pub async fn create(app: &TestApp, store: &str) -> (StatusCode, Value) {
    app.post_json("/api/trolley-handoffs", &json!({ "store": store }))
        .await
}

/// Creates a handoff with a delivery request.
pub async fn create_with_delivery(app: &TestApp, delivery: &Value) -> (StatusCode, Value) {
    app.post_json(
        "/api/trolley-handoffs",
        &json!({ "store": "woolworths", "delivery": delivery }),
    )
    .await
}

/// Reports for a claimed handoff, as the store tab does.
pub async fn report(
    app: &TestApp,
    handoff_id: &str,
    body: Value,
) -> (StatusCode, Value, HeaderMap) {
    let mut body = body;
    body["secret"] = json!(TEST_STORE_TAB_SECRET);
    store_tab_post(
        app,
        &format!("/api/store-tab/trolley-handoffs/{handoff_id}/report"),
        &body,
    )
    .await
}
