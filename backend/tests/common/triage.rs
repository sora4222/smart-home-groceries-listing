//! Helpers for the triage tests: finding where a request ended up once its
//! background check is stored.

use std::time::Duration;

use axum::http::StatusCode;
use serde_json::Value;

use super::TestApp;

/// The queue a request is shown in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shown {
    Pending,
    Held,
    Rejected,
}

impl TestApp {
    /// Where request `id` is shown right now, if anywhere.
    pub async fn shown_in(&self, id: &str) -> Option<(Shown, Value)> {
        for (place, uri) in [
            (Shown::Pending, "/api/voice-requests"),
            (Shown::Held, "/api/triage?tab=held"),
            (Shown::Rejected, "/api/triage?tab=rejected"),
        ] {
            let (status, list) = self.get(uri).await;
            assert_eq!(status, StatusCode::OK, "{uri}: {list}");
            if let Some(found) = list.as_array().unwrap().iter().find(|r| r["id"] == id) {
                return Some((place, found.clone()));
            }
        }
        None
    }

    /// Waits until the background check has put request `id` in a queue.
    ///
    /// The check runs after the intake answer is sent, so the test polls; it
    /// fails after five seconds rather than hanging.
    pub async fn triaged(&self, id: &str) -> (Shown, Value) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(found) = self.shown_in(id).await {
                return found;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "request {id} was not triaged within five seconds"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// Records a webhook item and waits for its triage.
    pub async fn webhook_item_triaged(&self, item: &str) -> (String, Shown, Value) {
        let id = self.create_pending(item, 1).await;
        let (shown, request) = self.triaged(&id).await;
        (id, shown, request)
    }
}
