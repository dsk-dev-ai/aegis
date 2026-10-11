//! # veil-server
//!
//! A deliberately minimal relay. It performs **store-and-forward of opaque
//! ciphertext** and nothing else:
//!
//! * it cannot read message contents (payloads are end-to-end encrypted),
//! * it stores no contact lists or social graph,
//! * undelivered messages expire after their TTL.
//!
//! This crate is the seam for a horizontally scalable, self-hostable relay.

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
use serde::Serialize;
use veil_core::RelayEnvelope;

/// Shared, in-memory relay state. Replace with a durable, sharded store for
/// production; the interface should stay the same.
#[derive(Clone, Default)]
pub struct AppState {
    inboxes: Arc<DashMap<String, Vec<RelayEnvelope>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue an envelope for later delivery.
    pub fn enqueue(&self, envelope: RelayEnvelope) {
        self.inboxes
            .entry(envelope.to.clone())
            .or_default()
            .push(envelope);
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
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("recipients", &self.inboxes.len())
            .finish()
    }
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
}

/// Build the relay's HTTP/WebSocket router.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/ws", get(ws))
        .with_state(state)
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn ws(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                if let Ok(envelope) = serde_json::from_str::<RelayEnvelope>(&text) {
                    tracing::debug!(to = %envelope.to, "relay: storing envelope");
                    state.enqueue(envelope);
                } else {
                    tracing::warn!("relay: received malformed envelope");
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
