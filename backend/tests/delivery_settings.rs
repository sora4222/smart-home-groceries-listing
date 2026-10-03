//! Settings › Delivery through the real router: `GET` and
//! `PUT /api/delivery-settings`.

mod common;

use axum::http::{Method, StatusCode};
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

async fn read(app: &TestApp) -> Value {
    let (status, body) = app.get("/api/delivery-settings").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

fn woolworths(fee: Value, free_over: Value, minimum: Value) -> Value {
    json!({
        "store": "woolworths",
        "delivery_fee": fee,
        "free_delivery_over": free_over,
        "minimum_order": minimum,
    })
}

#[sqlx::test]
async fn nothing_saved_reads_as_unset_fees_and_the_default_mode(pool: PgPool) {
    let app = TestApp::new(pool);

    let body = read(&app).await;

    assert_eq!(body["mode"], "minimise_total");
    assert_eq!(body["max_delivery_spend"], Value::Null);
    let stores = body["stores"].as_array().unwrap();
    assert_eq!(stores.len(), 2);
    assert_eq!(stores[0]["store"], "woolworths");
    assert_eq!(stores[0]["store_name"], "Woolworths");
    assert_eq!(stores[0]["delivery_fee"], Value::Null);
    assert_eq!(stores[1]["store"], "coles");
}

#[sqlx::test]
async fn saved_settings_are_read_back(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, saved) = app
        .put_json(
            "/api/delivery-settings",
            &json!({
                "stores": [woolworths(json!("9.00"), json!("250"), json!("50"))],
                "mode": "minimise_delivery",
                "max_delivery_spend": "15",
            }),
        )
        .await;

    assert_eq!(status, StatusCode::OK, "{saved}");
    let body = read(&app).await;
    assert_eq!(body, saved);
    assert_eq!(body["mode"], "minimise_delivery");
    assert_eq!(body["max_delivery_spend"], "15");
    assert_eq!(body["stores"][0]["delivery_fee"], "9.00");
    assert_eq!(body["stores"][0]["free_delivery_over"], "250");
    assert_eq!(body["stores"][0]["minimum_order"], "50");
    // Coles was not sent, so it keeps its (unset) rules.
    assert_eq!(body["stores"][1]["delivery_fee"], Value::Null);
}

#[sqlx::test]
async fn saving_again_replaces_and_can_clear_a_value(pool: PgPool) {
    let app = TestApp::new(pool);
    let first = json!({
        "stores": [woolworths(json!("9"), json!("250"), Value::Null)],
        "mode": "manual",
        "max_delivery_spend": null,
    });
    app.put_json("/api/delivery-settings", &first).await;

    let second = json!({
        "stores": [woolworths(json!("12"), Value::Null, Value::Null)],
        "mode": "coles_only",
        "max_delivery_spend": null,
    });
    let (status, body) = app.put_json("/api/delivery-settings", &second).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["mode"], "coles_only");
    assert_eq!(body["stores"][0]["delivery_fee"], "12");
    assert_eq!(body["stores"][0]["free_delivery_over"], Value::Null);
}

#[sqlx::test]
async fn amounts_out_of_range_and_unknown_modes_are_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    for body in [
        json!({ "stores": [woolworths(json!("-1"), Value::Null, Value::Null)], "mode": "manual" }),
        json!({ "stores": [woolworths(json!("1000.01"), Value::Null, Value::Null)], "mode": "manual" }),
        json!({ "stores": [woolworths(json!("9.999"), Value::Null, Value::Null)], "mode": "manual" }),
        json!({ "stores": [], "mode": "cheapest_ever" }),
        json!({ "stores": [], "mode": "manual", "max_delivery_spend": "-5" }),
    ] {
        let (status, answer) = app.put_json("/api/delivery-settings", &body).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{body} → {answer}"
        );
    }
    assert_eq!(read(&app).await["mode"], "minimise_total");
}

#[sqlx::test]
async fn delivery_settings_need_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    for method in [Method::GET, Method::PUT] {
        let (status, _) = app
            .send(
                axum::http::Request::builder()
                    .method(method.clone())
                    .uri("/api/delivery-settings")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        json!({ "stores": [], "mode": "manual" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method}");
    }
}
