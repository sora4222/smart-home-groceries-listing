//! Helpers for the Google Tasks tests: signing in, choosing a list and
//! polling, all through the real routes.

use axum::http::StatusCode;
use serde_json::{json, Value};

use super::TestApp;

/// The `state` carried in a sign-in URL.
pub fn state_in(authorize_url: &str) -> String {
    query_value(authorize_url, "state").expect("the sign-in URL carries a state")
}

/// One query value of a URL.
pub fn query_value(raw: &str, key: &str) -> Option<String> {
    url::Url::parse(raw)
        .expect("a URL")
        .query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

impl TestApp {
    /// Signs in to Google Tasks with `code` and returns the status.
    pub async fn connect_google_tasks(&self, code: &str) -> Value {
        let (status, started) = self.post("/api/intake/google-tasks/sign-in").await;
        assert_eq!(status, StatusCode::OK, "{started}");
        let state = state_in(started["authorize_url"].as_str().unwrap());
        let (status, finished) = self
            .post_json(
                "/api/intake/google-tasks/sign-in/finish",
                &json!({ "code": code, "state": state }),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{finished}");
        finished
    }

    /// Chooses `list_id` and switches polling on.
    pub async fn watch_list(&self, list_id: &str) -> Value {
        let (status, saved) = self
            .put_json(
                "/api/intake/google-tasks",
                &json!({ "task_list_id": list_id, "enabled": true, "poll_seconds": 60 }),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{saved}");
        saved
    }

    /// Adds a task to the fake account's Groceries list.
    pub async fn add_fake_task(&self, title: &str) {
        let (status, body) = self
            .post_json("/api/dev/google-tasks/tasks", &json!({ "title": title }))
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }

    /// "Check now"; returns the status and the poll report.
    pub async fn poll_google_tasks(&self) -> (StatusCode, Value) {
        self.post("/api/intake/google-tasks/poll").await
    }
}
