//! The delivery window part of a trolley handoff, through the real router:
//! what the web app asks for, what the store tab is told, and what it reports.

mod common;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use common::trolley::{chosen_item, claim, create, create_with_delivery, report};
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn by_default_the_store_tab_is_asked_for_the_next_day_any_time(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;

    let (status, created) = create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;

    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(
        created["delivery"]["requested"],
        json!({ "date": null, "time_of_day": "any" })
    );
    assert!(created["delivery"]["outcome"].is_null());
    // `date: null` tells the store tab "the day after the store's today".
    assert_eq!(
        claimed["delivery"],
        json!({ "date": null, "time_of_day": "any" })
    );
}

#[sqlx::test]
async fn a_chosen_day_and_time_of_day_reach_the_store_tab(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let day = (Utc::now() + Duration::days(3)).date_naive().to_string();

    let (status, _) =
        create_with_delivery(&app, &json!({ "date": day, "time_of_day": "evening" })).await;
    let (_, claimed, _) = claim(&app).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        claimed["delivery"],
        json!({ "date": day, "time_of_day": "evening" })
    );
}

#[sqlx::test]
async fn a_past_or_far_off_day_or_unknown_time_of_day_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let past = (Utc::now() - Duration::days(5)).date_naive().to_string();
    let far = (Utc::now() + Duration::days(40)).date_naive().to_string();

    for delivery in [
        json!({ "date": past }),
        json!({ "date": far }),
        json!({ "time_of_day": "midnight" }),
    ] {
        let (status, body) = create_with_delivery(&app, &delivery).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{delivery} → {body}"
        );
    }
}

#[sqlx::test]
async fn the_reserved_window_is_saved_and_shown_to_the_web_app(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();

    let (status, body, _) = report(
        &app,
        id,
        json!({
            "lines": [{ "product_id": "w-milk-2l", "outcome": "added" }],
            "delivery": {
                "outcome": "reserved",
                "window_label": "7:00am - 10:00am",
                "window_start": "2026-10-03T07:00:00",
                "window_end": "2026-10-03T10:00:00",
                "fee": "15"
            }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, seen) = app.get(&format!("/api/trolley-handoffs/{id}")).await;
    let delivery = &seen["delivery"];
    assert_eq!(delivery["outcome"], "reserved");
    assert_eq!(delivery["window_label"], "7:00am - 10:00am");
    assert_eq!(delivery["window_start"], "2026-10-03T07:00:00");
    assert_eq!(delivery["fee"], "15");
    assert_eq!(seen["status"], "filled");
}

#[sqlx::test]
async fn a_failed_window_is_saved_with_its_reason_and_products_still_count(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();

    let (status, _, _) = report(
        &app,
        id,
        json!({
            "lines": [{ "product_id": "w-milk-2l", "outcome": "added" }],
            "delivery": { "outcome": "failed", "problem": "No delivery windows this week" }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let (_, seen) = app.get(&format!("/api/trolley-handoffs/{id}")).await;
    assert_eq!(seen["delivery"]["outcome"], "failed");
    assert_eq!(seen["delivery"]["problem"], "No delivery windows this week");
}

#[sqlx::test]
async fn a_window_without_its_times_or_ending_before_it_starts_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();
    let lines = json!([{ "product_id": "w-milk-2l", "outcome": "added" }]);

    let no_times =
        json!({ "lines": lines, "delivery": { "outcome": "reserved", "window_label": "7am" } });
    let (status, _, _) = report(&app, id, no_times).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let backwards = json!({ "lines": lines, "delivery": {
        "outcome": "kept", "window_label": "7am",
        "window_start": "2026-10-03T10:00:00", "window_end": "2026-10-03T07:00:00" } });
    let (status, _, _) = report(&app, id, backwards).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
