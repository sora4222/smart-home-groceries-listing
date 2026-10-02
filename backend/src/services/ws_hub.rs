//! Fan-out of real-time events to connected browser sessions.
//!
//! A `tokio::sync::broadcast` channel replaces the previous per-socket
//! registry: each WebSocket task holds a receiver, so a slow or dead client
//! cannot block a publisher, and no lock is held across an await.
//!
//! Events are process-local, which suits the single-machine home-server
//! deployment this app targets. Fronting several backend processes would need
//! an external bus (Postgres `LISTEN/NOTIFY` would be the smallest change).

use serde::Serialize;
use tokio::sync::broadcast;

/// How many undelivered events a connection may fall behind by before it is
/// dropped from the channel. Events are small and clients only need the latest
/// count, so a short buffer is enough.
const CHANNEL_CAPACITY: usize = 64;

/// An event pushed to every open browser session.
///
/// The wire shape is unchanged from the previous implementation, so the web
/// app's WebSocket client needs no changes.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    /// The pending-request count changed (an item arrived, or one was decided).
    VoiceRequestAdded { count: i64 },
    /// The held-for-review count changed (triage held an item, or a person
    /// moved or rejected one). Drives the Triage nav badge and its toast.
    TriageHeld { count: i64 },
}

/// Publishes [`ServerEvent`]s to all subscribed connections.
#[derive(Debug, Clone)]
pub struct WsHub {
    sender: broadcast::Sender<ServerEvent>,
}

impl WsHub {
    /// Creates a hub with no subscribers.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Returns a receiver for a newly connected session.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }

    /// Sends an event to every subscriber.
    ///
    /// Having no subscribers is not an error — a broadcast with nobody
    /// listening is the normal state when no browser tab is open.
    pub fn broadcast(&self, event: ServerEvent) {
        let _ = self.sender.send(event);
    }

    /// Number of currently subscribed sessions. Used by tests and diagnostics.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialises_to_the_shape_the_web_app_parses() {
        let json = serde_json::to_value(ServerEvent::VoiceRequestAdded { count: 3 }).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "voice_request_added", "count": 3 })
        );
    }

    #[test]
    fn triage_held_serialises_with_its_own_type() {
        let json = serde_json::to_value(ServerEvent::TriageHeld { count: 2 }).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "triage_held", "count": 2 })
        );
    }

    #[tokio::test]
    async fn broadcast_reaches_every_subscriber() {
        let hub = WsHub::new();
        let mut first = hub.subscribe();
        let mut second = hub.subscribe();
        assert_eq!(hub.subscriber_count(), 2);

        hub.broadcast(ServerEvent::VoiceRequestAdded { count: 1 });

        assert_eq!(
            first.recv().await.unwrap(),
            ServerEvent::VoiceRequestAdded { count: 1 }
        );
        assert_eq!(
            second.recv().await.unwrap(),
            ServerEvent::VoiceRequestAdded { count: 1 }
        );
    }

    #[tokio::test]
    async fn broadcast_with_no_subscribers_is_not_an_error() {
        let hub = WsHub::new();
        hub.broadcast(ServerEvent::VoiceRequestAdded { count: 0 });
        assert_eq!(hub.subscriber_count(), 0);
    }
}
