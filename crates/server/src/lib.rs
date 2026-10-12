//! # aegis-server
//!
//! A deliberately minimal relay. It performs **store-and-forward of opaque
//! ciphertext** and routes envelopes between connected clients:
//!
//! * it cannot read message contents (payloads are end-to-end encrypted),
//! * it stores no contact lists or social graph,
//! * undelivered messages are held in memory and flushed when the recipient
//!   connects (or dropped — expiry/TTL is enforced by clients/policy).
//!
//! ## Wire protocol (over WebSocket)
//! * Client → Server, first frame: `{"hello": "<routing_token>"}`
//! * Client → Server, thereafter: `RelayEnvelope` JSON
//! * Server → Client: `RelayEnvelope` JSON (forwarded or flushed)

#![forbid(unsafe_code)]

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::mpsc;
use aegis_core::RelayEnvelope;

/// Shared, in-memory relay state.
#[derive(Clone, Default)]
pub struct AppState {
    /// Undelivered envelopes per recipient routing token.
    inboxes: Arc<DashMap<String, Vec<RelayEnvelope>>>,
    /// Live connections per routing token.
    peers: Arc<DashMap<String, mpsc::UnboundedSender<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    fn register(&self, token: &str, tx: mpsc::UnboundedSender<String>) {
        self.peers.insert(token.to_string(), tx);
    }

    fn unregister(&self, token: &str) {
        self.peers.remove(token);
    }

    /// Route an envelope to a live peer, or queue it for later.
    pub fn deliver(&self, envelope: RelayEnvelope) {
        if let Some(tx) = self.peers.get(&envelope.to) {
            if let Ok(json) = serde_json::to_string(&envelope) {
                if tx.send(json).is_ok() {
                    return;
                }
            }
        }
        self.enqueue(envelope);
    }

    /// Queue an envelope for later delivery.
    pub fn enqueue(&self, envelope: RelayEnvelope) {
        let mut entry = self.inboxes.entry(envelope.to.clone()).or_default();
        if entry.len() < 1000 {
            entry.push(envelope);
        }
    }

    /// Drain all pending envelopes for a recipient.
    pub fn drain(&self, recipient: &str) -> Vec<RelayEnvelope> {
        self.inboxes
            .remove(recipient)
            .map(|(_, v)| v)
            .unwrap_or_default()
    }

    pub fn pending(&self, recipient: &str) -> usize {
        self.inboxes.get(recipient).map(|v| v.len()).unwrap_or(0)
    }

    pub fn online(&self) -> usize {
        self.peers.len()
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("recipients", &self.inboxes.len())
            .field("peers", &self.peers.len())
            .finish()
    }
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    peers: usize,
}

/// Build the relay's HTTP/WebSocket router.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/ws", get(ws))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        peers: state.online(),
    })
}

async fn ws(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();

    // First frame must be the hello handshake carrying a routing token.
    let token = loop {
        match stream.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(token) = value.get("hello").and_then(|v| v.as_str()) {
                        break token.to_string();
                    }
                }
            }
            Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return,
            _ => {}
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    state.register(&token, tx);
    tracing::info!(%token, "peer connected");

    // Flush anything queued while the peer was offline.
    for envelope in state.drain(&token) {
        if let Ok(json) = serde_json::to_string(&envelope) {
            if sink.send(Message::Text(json)).await.is_err() {
                state.unregister(&token);
                return;
            }
        }
    }

    // Outbound pump: forwards delivered envelopes to this socket.
    let send_task = tokio::spawn(async move {
        while let Some(json) = rx.recv().await {
            if sink.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Inbound loop.
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(envelope) = serde_json::from_str::<RelayEnvelope>(&text) {
                    state.deliver(envelope);
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    state.unregister(&token);
    send_task.abort();
    tracing::info!(%token, "peer disconnected");
}
