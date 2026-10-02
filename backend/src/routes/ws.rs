//! WebSocket endpoint pushing real-time events to open browser sessions.
//!
//! Needs a session like every household route; the token comes as
//! `?token=` because browsers cannot set headers on a WebSocket (see
//! `auth/socket.rs`).
//!
//! Currently only `voice_request_added` is emitted (see
//! `services/ws_hub.rs`). The web app uses it to update the Pending Requests
//! badge and raise a toast without polling.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use axum::routing::any;
use axum::Router;
use tokio::sync::broadcast::error::RecvError;

use crate::auth::socket::SocketUser;
use crate::services::ws_hub::ServerEvent;
use crate::state::AppState;

/// The `/ws` route.
pub fn router() -> Router<AppState> {
    Router::new().route("/ws", any(upgrade))
}

/// Accepts the upgrade from a signed-in household member (401 otherwise) and
/// hands the socket to [`run_socket`].
async fn upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    _user: SocketUser,
) -> Response {
    ws.on_upgrade(move |socket| run_socket(socket, state))
}

/// Forwards hub events to one browser session until it goes away.
///
/// The client sends nothing meaningful; reading from the socket is how a
/// disconnect is noticed. Both directions are polled together so a closed
/// socket is dropped promptly instead of only on the next event.
async fn run_socket(mut socket: WebSocket, state: AppState) {
    let mut events = state.hub.subscribe();

    loop {
        tokio::select! {
            received = events.recv() => match received {
                Ok(event) => {
                    if send_event(&mut socket, &event).await.is_err() {
                        return;
                    }
                }
                // The session fell behind the channel's buffer. The events
                // carry an absolute count, so the next one brings it back in
                // step — no need to close the socket.
                Err(RecvError::Lagged(skipped)) => {
                    tracing::debug!(skipped, "websocket session lagged behind");
                }
                Err(RecvError::Closed) => return,
            },
            incoming = socket.recv() => match incoming {
                // Anything the client sends is ignored; only closure matters.
                Some(Ok(_)) => continue,
                Some(Err(_)) | None => return,
            },
        }
    }
}

/// Serialises and sends one event.
async fn send_event(socket: &mut WebSocket, event: &ServerEvent) -> Result<(), ()> {
    let payload = match serde_json::to_string(event) {
        Ok(payload) => payload,
        Err(err) => {
            tracing::error!(error = ?err, "could not serialise a server event");
            return Ok(());
        }
    };
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(|_| ())
}
