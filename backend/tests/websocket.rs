//! Integration tests for the `/ws` real-time event stream.
//!
//! The web app's pending badge and toast depend on this, so it is tested over
//! a real socket against a real server rather than in process.

mod common;

use std::time::Duration;

use common::TestApp;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message;

/// Waits for the next text frame, failing the test rather than hanging if the
/// server never sends one.
async fn next_event<S>(socket: &mut S) -> Value
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    let frame = tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("an event should arrive within five seconds")
        .expect("the socket should still be open")
        .expect("the frame should be readable");

    match frame {
        Message::Text(text) => serde_json::from_str(&text).expect("the frame should be JSON"),
        other => panic!("expected a text frame, got {other:?}"),
    }
}

#[sqlx::test]
async fn a_new_intake_request_pushes_the_pending_count(pool: PgPool) {
    let app = TestApp::new(pool);
    let address = app.serve().await;

    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .expect("the websocket should accept the upgrade");

    app.post_webhook(&json!({ "item": "milk", "quantity": 1 }))
        .await;

    assert_eq!(
        next_event(&mut socket).await,
        json!({ "type": "voice_request_added", "count": 1 })
    );
}

#[sqlx::test]
async fn every_open_session_receives_the_same_event(pool: PgPool) {
    let app = TestApp::new(pool);
    let address = app.serve().await;
    let url = format!("ws://{address}/ws");

    let (mut first, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let (mut second, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

    app.post_webhook(&json!({ "item": "bread" })).await;

    let expected = json!({ "type": "voice_request_added", "count": 1 });
    assert_eq!(next_event(&mut first).await, expected);
    assert_eq!(next_event(&mut second).await, expected);
}

#[sqlx::test]
async fn accepting_and_rejecting_also_push_the_new_count(pool: PgPool) {
    let app = TestApp::new(pool);
    let address = app.serve().await;

    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();

    let first = app.create_pending("milk", 1).await;
    let second = app.create_pending("bread", 1).await;
    assert_eq!(next_event(&mut socket).await["count"], 1);
    assert_eq!(next_event(&mut socket).await["count"], 2);

    app.post(&format!("/api/voice-requests/{first}/accept"))
        .await;
    assert_eq!(next_event(&mut socket).await["count"], 1);

    app.post(&format!("/api/voice-requests/{second}/reject"))
        .await;
    assert_eq!(next_event(&mut socket).await["count"], 0);
}

#[sqlx::test]
async fn an_alexa_item_pushes_an_event_too(pool: PgPool) {
    let app = TestApp::new(pool);
    let address = app.serve().await;

    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();

    app.post_alexa(&json!({ "item": "rice", "external_id": "amzn-1" }))
        .await;

    assert_eq!(next_event(&mut socket).await["count"], 1);
}

#[sqlx::test]
async fn client_messages_are_ignored_and_keep_the_socket_open(pool: PgPool) {
    // The web app's client sends nothing meaningful; anything it does send
    // must not close the stream.
    let app = TestApp::new(pool);
    let address = app.serve().await;

    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();
    socket.send(Message::Text("ping".into())).await.unwrap();

    app.post_webhook(&json!({ "item": "milk" })).await;

    assert_eq!(next_event(&mut socket).await["count"], 1);
}

#[sqlx::test]
async fn a_dropped_session_does_not_stop_the_others(pool: PgPool) {
    let app = TestApp::new(pool);
    let address = app.serve().await;
    let url = format!("ws://{address}/ws");

    let (survivor_socket, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let mut survivor = survivor_socket;
    let (leaver, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    drop(leaver);

    app.post_webhook(&json!({ "item": "milk" })).await;

    assert_eq!(next_event(&mut survivor).await["count"], 1);
}
