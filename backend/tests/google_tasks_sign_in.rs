//! Signing in to Google Tasks and out again, against the fake account: the
//! one-use sign-in state, the stored (never returned) token, and the
//! encryption key it needs.

mod common;

use axum::http::StatusCode;
use common::google_tasks::state_in;
use common::{google_tasks_settings, triage_settings, TestApp};
use grocery_backend::config::{GoogleTasksMode, TriageProvider};
use serde_json::json;
use sqlx::PgPool;

fn app(pool: PgPool) -> TestApp {
    TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Fake, "http://127.0.0.1:9"),
        triage_settings(TriageProvider::Off, ""),
    )
}

#[sqlx::test]
async fn signing_in_stores_the_connection_but_never_returns_the_token(pool: PgPool) {
    let app = app(pool);

    let status = app.connect_google_tasks("fake-code").await;

    assert_eq!(status["connected"], true);
    assert!(status["connected_at"].is_string());
    let (_, settings) = app.get("/api/intake/settings").await;
    assert!(!settings.to_string().contains("fake-refresh-token"));
}

#[sqlx::test]
async fn a_sign_in_state_works_once(pool: PgPool) {
    let app = app(pool);
    let (_, started) = app.post("/api/intake/google-tasks/sign-in").await;
    let state = state_in(started["authorize_url"].as_str().unwrap());
    let body = json!({ "code": "fake-code", "state": state });

    let (first, _) = app
        .post_json("/api/intake/google-tasks/sign-in/finish", &body)
        .await;
    let (again, _) = app
        .post_json("/api/intake/google-tasks/sign-in/finish", &body)
        .await;

    assert_eq!(first, StatusCode::OK);
    assert_eq!(again, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn a_state_this_app_never_issued_is_refused(pool: PgPool) {
    let app = app(pool);

    let (status, body) = app
        .post_json(
            "/api/intake/google-tasks/sign-in/finish",
            &json!({ "code": "fake-code", "state": "made-up" }),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["detail"].as_str().unwrap().contains("expired"));
    let (_, settings) = app.get("/api/intake/settings").await;
    assert_eq!(settings["google_tasks"]["connected"], false);
}

#[sqlx::test]
async fn a_bad_code_from_google_stores_nothing(pool: PgPool) {
    let app = app(pool);
    let (_, started) = app.post("/api/intake/google-tasks/sign-in").await;
    let state = state_in(started["authorize_url"].as_str().unwrap());

    let (status, _) = app
        .post_json(
            "/api/intake/google-tasks/sign-in/finish",
            &json!({ "code": "wrong", "state": state }),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    let (_, settings) = app.get("/api/intake/settings").await;
    assert_eq!(settings["google_tasks"]["connected"], false);
}

#[sqlx::test]
async fn disconnecting_stops_polling(pool: PgPool) {
    let app = app(pool);
    app.connect_google_tasks("fake-code").await;
    app.watch_list("fake-groceries").await;

    let (status, _) = app.delete("/api/intake/google-tasks/sign-in").await;
    let (_, settings) = app.get("/api/intake/settings").await;
    let (poll, _) = app.poll_google_tasks().await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(settings["google_tasks"]["connected"], false);
    assert_eq!(settings["google_tasks"]["polling"], false);
    assert_eq!(
        settings["google_tasks"]["task_list"]["title"], "Groceries",
        "kept"
    );
    assert_eq!(poll, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn signing_in_needs_the_encryption_key(pool: PgPool) {
    // `TestApp::new` sets no CREDENTIAL_ENCRYPTION_KEY.
    let app = TestApp::new(pool);

    let (status, body) = app.post("/api/intake/google-tasks/sign-in").await;
    let (_, settings) = app.get("/api/intake/settings").await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["detail"], "Server configuration error");
    assert_eq!(settings["google_tasks"]["encryption_ready"], false);
}
