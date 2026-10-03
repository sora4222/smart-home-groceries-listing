//! Google Tasks intake through the real router, against the fake account
//! (`GOOGLE_TASKS_CLIENT=fake`): signing in, choosing a list, and polling.

mod common;

use axum::http::StatusCode;
use common::{google_tasks_settings, triage_settings, TestApp};
use grocery_backend::config::{GoogleTasksMode, TriageProvider};
use serde_json::json;
use sqlx::PgPool;

const GROCERIES: &str = "fake-groceries";

fn app(pool: PgPool) -> TestApp {
    TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Fake, "http://127.0.0.1:9"),
        triage_settings(TriageProvider::Off, ""),
    )
}

/// Connected, watching the fake Groceries list.
async fn watching(pool: PgPool) -> TestApp {
    let app = app(pool);
    app.connect_google_tasks("fake-code").await;
    app.watch_list(GROCERIES).await;
    app
}

#[sqlx::test]
async fn the_settings_page_starts_not_connected(pool: PgPool) {
    let app = app(pool);

    let (status, settings) = app.get("/api/intake/settings").await;

    assert_eq!(status, StatusCode::OK);
    let tasks = &settings["google_tasks"];
    assert_eq!(tasks["configured"], true);
    assert_eq!(tasks["encryption_ready"], true);
    assert_eq!(tasks["connected"], false);
    assert_eq!(tasks["polling"], false);
    assert_eq!(tasks["poll_seconds"], 60);
    assert_eq!(settings["alexa"]["configured"], true);
    assert_eq!(settings["triage"]["provider"], "off");
}

#[sqlx::test]
async fn lists_need_a_connection_first(pool: PgPool) {
    let app = app(pool);

    let (before, _) = app.get("/api/intake/google-tasks/lists").await;
    app.connect_google_tasks("fake-code").await;
    let (after, lists) = app.get("/api/intake/google-tasks/lists").await;

    assert_eq!(before, StatusCode::CONFLICT);
    assert_eq!(after, StatusCode::OK);
    let titles: Vec<_> = lists
        .as_array()
        .unwrap()
        .iter()
        .map(|l| &l["title"])
        .collect();
    assert_eq!(titles, vec!["Groceries", "My Tasks"]);
}

#[sqlx::test]
async fn choosing_a_list_stores_its_title_and_turns_polling_on(pool: PgPool) {
    let app = app(pool);
    app.connect_google_tasks("fake-code").await;

    let saved = app.watch_list(GROCERIES).await;

    assert_eq!(saved["task_list"]["title"], "Groceries");
    assert_eq!(saved["enabled"], true);
    assert_eq!(saved["polling"], true);
}

#[sqlx::test]
async fn a_list_not_on_the_account_or_polling_without_a_list_is_refused(pool: PgPool) {
    let app = app(pool);
    app.connect_google_tasks("fake-code").await;

    let (unknown, _) = app
        .put_json(
            "/api/intake/google-tasks",
            &json!({ "task_list_id": "nope", "enabled": true, "poll_seconds": 60 }),
        )
        .await;
    let (no_list, _) = app
        .put_json(
            "/api/intake/google-tasks",
            &json!({ "task_list_id": null, "enabled": true, "poll_seconds": 60 }),
        )
        .await;
    let (too_fast, _) = app
        .put_json(
            "/api/intake/google-tasks",
            &json!({ "task_list_id": GROCERIES, "enabled": true, "poll_seconds": 5 }),
        )
        .await;

    assert_eq!(unknown, StatusCode::NOT_FOUND);
    assert_eq!(no_list, StatusCode::CONFLICT);
    assert_eq!(too_fast, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn a_poll_turns_each_task_into_a_pending_request_and_deletes_it(pool: PgPool) {
    let app = watching(pool).await;
    app.add_fake_task("2 oat milk").await;
    app.add_fake_task("bread").await;

    let (status, report) = app.poll_google_tasks().await;

    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["recorded"], 2);
    let (_, pending) = app.get("/api/voice-requests").await;
    let oat = pending
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["parsed_name"] == "oat milk")
        .expect("oat milk is pending");
    assert_eq!(oat["parsed_quantity"], 2);
    assert_eq!(oat["source"], "tasks");
    assert_eq!(oat["raw_text"], "2 oat milk");

    // Deleted from Google: the next poll finds nothing.
    let (_, again) = app.poll_google_tasks().await;
    assert_eq!(
        again,
        json!({ "recorded": 0, "already_seen": 0, "blank": 0, "not_deleted": 0 })
    );
}

#[sqlx::test]
async fn nothing_reaches_the_list_without_a_person(pool: PgPool) {
    let app = watching(pool).await;
    app.add_fake_task("eggs").await;

    app.poll_google_tasks().await;

    let (_, list) = app.get("/api/grocery-items").await;
    assert_eq!(list, json!([]));
}

#[sqlx::test]
async fn a_blank_task_is_left_alone(pool: PgPool) {
    let app = watching(pool).await;
    app.add_fake_task("   ").await;

    let (_, first) = app.poll_google_tasks().await;
    let (_, second) = app.poll_google_tasks().await;

    assert_eq!(first["blank"], 1);
    assert_eq!(second["blank"], 1, "still there, not deleted");
    let (_, pending) = app.get("/api/voice-requests").await;
    assert_eq!(pending, json!([]));
}

#[sqlx::test]
async fn a_poll_records_when_it_ran(pool: PgPool) {
    let app = watching(pool).await;

    app.poll_google_tasks().await;

    let (_, settings) = app.get("/api/intake/settings").await;
    assert!(settings["google_tasks"]["last_polled_at"].is_string());
    assert!(settings["google_tasks"]["last_poll_error"].is_null());
}

#[sqlx::test]
async fn check_now_needs_polling_set_up(pool: PgPool) {
    let app = app(pool);
    app.connect_google_tasks("fake-code").await;

    let (status, body) = app.poll_google_tasks().await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["detail"].as_str().unwrap().contains("pick a list"));
}

#[sqlx::test]
async fn every_route_needs_a_signed_in_member(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    for (method, uri) in [
        ("GET", "/api/intake/settings"),
        ("POST", "/api/intake/google-tasks/sign-in"),
        ("DELETE", "/api/intake/google-tasks/sign-in"),
        ("GET", "/api/intake/google-tasks/lists"),
        ("POST", "/api/intake/google-tasks/poll"),
        (
            "POST",
            "/api/triage/00000000-0000-0000-0000-000000000000/restore",
        ),
    ] {
        let status = match method {
            "GET" => app.get(uri).await.0,
            "DELETE" => app.delete(uri).await.0,
            _ => app.post(uri).await.0,
        };
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
    }
}

#[sqlx::test]
async fn the_fake_tasks_door_does_not_exist_against_the_real_google(pool: PgPool) {
    let app = TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Live, "http://127.0.0.1:9"),
        triage_settings(TriageProvider::Off, ""),
    );

    let (status, _) = app
        .post_json("/api/dev/google-tasks/tasks", &json!({ "title": "milk" }))
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
