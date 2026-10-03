//! The triage step through the real router, with the keyword classifier
//! (`INTAKE_LLM_PROVIDER=fake`): which queue each item lands in, and the
//! Triage view's accept and reject.

mod common;

use axum::http::StatusCode;
use common::settings::triage_settings;
use common::triage::Shown;
use common::TestApp;
use grocery_backend::config::TriageProvider;
use serde_json::json;
use sqlx::PgPool;

fn app(pool: PgPool) -> TestApp {
    TestApp::with_triage(pool, triage_settings(TriageProvider::Fake, ""))
}

#[sqlx::test]
async fn a_grocery_item_is_approved_into_pending_requests(pool: PgPool) {
    let app = app(pool);

    let (_, shown, request) = app.webhook_item_triaged("oat milk").await;

    assert_eq!(shown, Shown::Pending);
    assert_eq!(request["triage_status"], "approved");
    assert_eq!(request["status"], "pending");
    assert!(request["triage_confidence"].as_f64().unwrap() > 0.7);
    assert!(request["triage_reason"]
        .as_str()
        .unwrap()
        .contains("supermarket"));
}

#[sqlx::test]
async fn a_service_is_rejected_into_the_triage_view_not_pending(pool: PgPool) {
    let app = app(pool);

    let (_, shown, request) = app.webhook_item_triaged("car service").await;

    assert_eq!(shown, Shown::Rejected);
    assert_eq!(request["triage_status"], "rejected");
    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending, json!([]));
}

#[sqlx::test]
async fn an_alexa_item_is_triaged_too(pool: PgPool) {
    let app = app(pool);

    let (status, created) = app
        .post_alexa(&json!({ "item": "dentist appointment", "external_id": "amzn-7" }))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["triage_status"], "unchecked");

    let (shown, _) = app.triaged(created["id"].as_str().unwrap()).await;
    assert_eq!(shown, Shown::Rejected);
}

#[sqlx::test]
async fn accept_in_triage_moves_the_request_to_pending_not_onto_the_list(pool: PgPool) {
    let app = app(pool);
    let (id, _, _) = app.webhook_item_triaged("bike repair kit").await;

    let (status, moved) = app.post(&format!("/api/triage/{id}/accept")).await;

    assert_eq!(status, StatusCode::OK, "{moved}");
    assert_eq!(moved["triage_status"], "skipped");
    assert_eq!(moved["status"], "pending");
    assert!(moved["triage_reason"].as_str().unwrap().contains("service"));
    let (shown, _) = app.shown_in(&id).await.unwrap();
    assert_eq!(shown, Shown::Pending);
    let (_, list) = app.get("/api/grocery-items").await;
    assert_eq!(
        list,
        json!([]),
        "nothing reaches the list until a person accepts it"
    );
}

#[sqlx::test]
async fn accept_in_triage_twice_or_on_an_approved_item_is_a_conflict(pool: PgPool) {
    let app = app(pool);
    let (rejected, _, _) = app.webhook_item_triaged("haircut").await;
    let (approved, _, _) = app.webhook_item_triaged("bread").await;

    app.post(&format!("/api/triage/{rejected}/accept")).await;
    let (again, _) = app.post(&format!("/api/triage/{rejected}/accept")).await;
    let (on_approved, _) = app.post(&format!("/api/triage/{approved}/accept")).await;

    assert_eq!(again, StatusCode::CONFLICT);
    assert_eq!(on_approved, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn accept_in_triage_of_an_unknown_request_is_not_found(pool: PgPool) {
    let app = app(pool);

    let (status, _) = app
        .post("/api/triage/00000000-0000-0000-0000-000000000000/accept")
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn reject_in_triage_confirms_the_rejection(pool: PgPool) {
    let app = app(pool);
    let (id, _, _) = app.webhook_item_triaged("plumber").await;

    let (status, rejected) = app.post(&format!("/api/triage/{id}/reject")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(rejected["status"], "rejected");
    assert!(app.shown_in(&id).await.is_none());
}

#[sqlx::test]
async fn an_unknown_tab_is_refused(pool: PgPool) {
    let app = app(pool);

    let (status, _) = app.get("/api/triage?tab=everything").await;

    assert!(status.is_client_error());
}

#[sqlx::test]
async fn the_triage_view_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (list, _) = app.get("/api/triage?tab=held").await;
    let (accept, _) = app
        .post("/api/triage/00000000-0000-0000-0000-000000000000/accept")
        .await;

    assert_eq!(list, StatusCode::UNAUTHORIZED);
    assert_eq!(accept, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn with_triage_off_an_item_is_pending_at_once_and_marked_skipped(pool: PgPool) {
    let app = TestApp::new(pool);

    let id = app.create_pending("milk", 1).await;

    let (shown, request) = app.shown_in(&id).await.unwrap();
    assert_eq!(shown, Shown::Pending);
    assert_eq!(request["triage_status"], "skipped");
}

#[sqlx::test]
async fn a_request_left_unchecked_by_a_restart_is_checked_at_startup(pool: PgPool) {
    sqlx::query(
        "INSERT INTO voice_requests
             (id, source, raw_text, parsed_name, parsed_quantity, status, triage_status)
         VALUES ('11111111-1111-1111-1111-111111111111', 'webhook', 'rego', 'rego', 1,
                 'pending', 'unchecked')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = app(pool);

    let (shown, _) = app.triaged("11111111-1111-1111-1111-111111111111").await;
    assert_eq!(shown, Shown::Rejected);
}
