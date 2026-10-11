//! WebSocket transport to the relay.
//!
//! TLS is terminated by a reverse proxy in front of the relay for now; enabling
//! native TLS in the client is a later step once a pure-Rust or system TLS
//! backend is selected.

use crate::error::{NetError, Result};
use aegis_core::RelayEnvelope;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// A framed, JSON-over-WebSocket connection carrying [`RelayEnvelope`]s.
pub struct Transport {
    ws: Ws,
}

impl std::fmt::Debug for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Transport(connected)")
    }
}

impl Transport {
    /// Connect to a `ws://` or `wss://` relay URL.
    pub async fn connect(url: &str) -> Result<Self> {
        let (ws, _resp) = connect_async(url)
            .await
            .map_err(|e| NetError::Connect(e.to_string()))?;
        Ok(Self { ws })
    }

    /// Send an envelope. Returns an error if the socket is closed.
    pub async fn send(&mut self, envelope: &RelayEnvelope) -> Result<()> {
        let text =
            serde_json::to_string(envelope).map_err(|e| NetError::Protocol(e.to_string()))?;
        self.ws
            .send(Message::Text(text))
            .await
            .map_err(|e| NetError::Transport(e.to_string()))
    }

    /// Receive the next envelope. `Ok(None)` means the peer closed cleanly.
    pub async fn recv(&mut self) -> Result<Option<RelayEnvelope>> {
        loop {
            let msg = match self.ws.next().await {
                Some(Ok(msg)) => msg,
                Some(Err(e)) => return Err(NetError::Transport(e.to_string())),
                None => return Ok(None),
            };
            match msg {
                Message::Text(text) => {
                    let env = serde_json::from_str(&text)
                        .map_err(|e| NetError::Protocol(e.to_string()))?;
                    return Ok(Some(env));
                }
                Message::Binary(bytes) => {
                    let env = serde_json::from_slice(&bytes)
                        .map_err(|e| NetError::Protocol(e.to_string()))?;
                    return Ok(Some(env));
                }
                Message::Close(_) => return Ok(None),
                _ => continue,
            }
        }
    }

    /// Send a WebSocket close frame.
    pub async fn close(&mut self) -> Result<()> {
        self.ws
            .close(None)
            .await
            .map_err(|e| NetError::Transport(e.to_string()))
    }
}
