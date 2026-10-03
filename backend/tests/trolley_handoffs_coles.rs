//! Coles trolley handoffs through the real router: the same handoff as
//! Woolworths, claimed by the "Fill Coles trolley" bookmark on coles.com.au.
//!
//! The Coles bookmark does not reserve a delivery time (the Coles delivery
//! calls are not known yet), so it reports the delivery as not chosen.

mod common;

use axum::http::StatusCode;
use common::trolley::{chosen_item, create, store_tab_post_from, COLES_ORIGIN};
use common::{TestApp, TEST_STORE_TAB_SECRET};
use serde_json::{json, Value};
use sqlx::PgPool;

/// Claims the newest Coles handoff, as the bookmark on coles.com.au does.
async fn claim_coles(app: &TestApp) -> (StatusCode, Value, axum::http::HeaderMap) {
    store_tab_post_from(
        app,
        COLES_ORIGIN,
        "/api/store-tab/trolley-handoffs/claim",
        &json!({ "secret": TEST_STORE_TAB_SECRET, "store": "coles" }),
    )
    .await
}

#[sqlx::test]
async fn a_coles_handoff_holds_only_coles_choices(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 2, "woolworths", "w-milk-2l").await;
    let oat = chosen_item(&app, "oat milk", 3, "coles", "c-oat-1l").await;

    let (status, body) = create(&app, "coles").await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["store"], "coles");
    assert_eq!(body["store_name"], "Coles");
    let lines = body["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["grocery_item_id"], oat);
    assert_eq!(lines[0]["product_id"], "c-oat-1l");
    assert_eq!(lines[0]["quantity"], 3);
}

#[sqlx::test]
async fn the_coles_tab_claims_only_the_coles_handoff(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "oat milk", 1, "coles", "c-oat-1l").await;
    let (_, woolworths) = create(&app, "woolworths").await;
    let (_, coles) = create(&app, "coles").await;

    let (status, body, headers) = claim_coles(&app).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["handoff_id"], coles["id"]);
    assert_eq!(body["lines"][0]["product_id"], "c-oat-1l");
    assert_eq!(headers["access-control-allow-origin"], COLES_ORIGIN);

    let (status, _, _) = claim_coles(&app).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "claimed once only");
    let (_, still) = app
        .get(&format!(
            "/api/trolley-handoffs/{}",
            woolworths["id"].as_str().unwrap()
        ))
        .await;
    assert_eq!(
        still["status"], "waiting_for_store_tab",
        "a Coles handoff never replaces or claims a Woolworths one"
    );
}

#[sqlx::test]
async fn a_coles_report_without_a_delivery_time_is_saved(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "oat milk", 1, "coles", "c-oat-1l").await;
    create(&app, "coles").await;
    let (_, claimed, _) = claim_coles(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();

    let (status, body, headers) = store_tab_post_from(
        &app,
        COLES_ORIGIN,
        &format!("/api/store-tab/trolley-handoffs/{id}/report"),
        &json!({
            "secret": TEST_STORE_TAB_SECRET,
            "delivery": {
                "outcome": "failed",
                "problem": "Choose a delivery time on Coles."
            },
            "lines": [{ "product_id": "c-oat-1l", "outcome": "added" }]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(headers["access-control-allow-origin"], COLES_ORIGIN);
    let (_, seen) = app.get(&format!("/api/trolley-handoffs/{id}")).await;
    assert_eq!(seen["status"], "filled");
    assert_eq!(seen["lines"][0]["outcome"], "added");
    assert_eq!(seen["delivery"]["outcome"], "failed");
    assert_eq!(
        seen["delivery"]["problem"],
        "Choose a delivery time on Coles."
    );
}
