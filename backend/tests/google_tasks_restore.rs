//! Triage's "Restore to source" for Google Tasks items, with the keyword
//! classifier and the fake Tasks account: the item goes back on the list,
//! and the next poll brings it straight to Pending Requests.

mod common;

use axum::http::StatusCode;
use common::triage::Shown;
use common::{google_tasks_settings, triage_settings, TestApp};
use grocery_backend::config::{GoogleTasksMode, TriageProvider};
use serde_json::Value;
use sqlx::PgPool;

async fn watching(pool: PgPool) -> TestApp {
    let app = TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Fake, "http://127.0.0.1:9"),
        triage_settings(TriageProvider::Fake, ""),
    );
    app.connect_google_tasks("fake-code").await;
    app.watch_list("fake-groceries").await;
    app
}

/// Adds a task, polls it in, and waits for triage. Returns the request.
async fn polled_and_triaged(app: &TestApp, title: &str) -> (Shown, Value) {
    app.add_fake_task(title).await;
    let (status, report) = app.poll_google_tasks().await;
    assert_eq!(report["recorded"], 1, "{status} {report}");
    let id = newest_request_id(app, title).await;
    app.triaged(&id).await
}

/// The id of the newest request with this raw text, in any queue.
async fn newest_request_id(app: &TestApp, raw_text: &str) -> String {
    for _ in 0..250 {
        for uri in [
            "/api/voice-requests",
            "/api/triage?tab=held",
            "/api/triage?tab=rejected",
        ] {
            let (_, list) = app.get(uri).await;
            if let Some(found) = list
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["raw_text"] == raw_text)
            {
                return found["id"].as_str().unwrap().to_string();
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("no request for {raw_text}");
}

#[sqlx::test]
async fn a_restored_item_skips_triage_on_the_next_poll(pool: PgPool) {
    let app = watching(pool).await;
    let (shown, request) = polled_and_triaged(&app, "car service").await;
    assert_eq!(shown, Shown::Rejected);
    let id = request["id"].as_str().unwrap();

    let (status, restored) = app.post(&format!("/api/triage/{id}/restore")).await;

    assert_eq!(status, StatusCode::OK, "{restored}");
    assert_eq!(restored["status"], "rejected");
    assert!(app.shown_in(id).await.is_none(), "it left the Triage view");

    let (_, report) = app.poll_google_tasks().await;
    assert_eq!(report["recorded"], 1);
    let (_, pending) = app.get("/api/voice-requests").await;
    let back = &pending.as_array().unwrap()[0];
    assert_eq!(back["parsed_name"], "car service");
    assert_eq!(back["triage_status"], "skipped");
    assert_ne!(back["id"], request["id"]);

    // Still a person's decision: nothing is on the list.
    let (_, list) = app.get("/api/grocery-items").await;
    assert_eq!(list, serde_json::json!([]));
}

#[sqlx::test]
async fn the_restored_task_keeps_what_was_typed(pool: PgPool) {
    let app = watching(pool).await;
    let (_, request) = polled_and_triaged(&app, "2 x car service").await;

    app.post(&format!(
        "/api/triage/{}/restore",
        request["id"].as_str().unwrap()
    ))
    .await;
    app.poll_google_tasks().await;

    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending[0]["raw_text"], "2 x car service");
    assert_eq!(pending[0]["parsed_quantity"], 2);
}

#[sqlx::test]
async fn only_an_undecided_triaged_request_can_be_restored(pool: PgPool) {
    let app = watching(pool).await;
    let (shown, approved) = polled_and_triaged(&app, "oat milk").await;
    assert_eq!(shown, Shown::Pending);

    let (status, _) = app
        .post(&format!(
            "/api/triage/{}/restore",
            approved["id"].as_str().unwrap()
        ))
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn restoring_twice_is_a_conflict(pool: PgPool) {
    let app = watching(pool).await;
    let (_, request) = polled_and_triaged(&app, "haircut").await;
    let uri = format!("/api/triage/{}/restore", request["id"].as_str().unwrap());

    let (first, _) = app.post(&uri).await;
    let (second, _) = app.post(&uri).await;

    assert_eq!(first, StatusCode::OK);
    assert_eq!(second, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn an_item_from_another_channel_cannot_be_restored(pool: PgPool) {
    let app = watching(pool).await;
    let (id, shown, _) = app.webhook_item_triaged("car service").await;
    assert_eq!(shown, Shown::Rejected);

    let (status, body) = app.post(&format!("/api/triage/{id}/restore")).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["detail"].as_str().unwrap().contains("Google Tasks"));
    assert_eq!(
        app.shown_in(&id).await.unwrap().0,
        Shown::Rejected,
        "unchanged"
    );
}

#[sqlx::test]
async fn restoring_needs_google_tasks_connected(pool: PgPool) {
    let app = watching(pool).await;
    let (_, request) = polled_and_triaged(&app, "dentist").await;
    let id = request["id"].as_str().unwrap();
    app.delete("/api/intake/google-tasks/sign-in").await;

    let (status, _) = app.post(&format!("/api/triage/{id}/restore")).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        app.shown_in(id).await.unwrap().0,
        Shown::Rejected,
        "unchanged"
    );
}

#[sqlx::test]
async fn an_unknown_request_is_404(pool: PgPool) {
    let app = watching(pool).await;

    let (status, _) = app
        .post("/api/triage/00000000-0000-0000-0000-000000000000/restore")
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
