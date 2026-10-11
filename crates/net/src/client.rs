//! High-level relay client: connects, authenticates, and provides a simple
//! send/receive API. Reconnection and push-notification wake-ups are layered
//! on top later.

use crate::error::Result;
use crate::transport::Transport;
use veil_core::RelayEnvelope;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub relay_url: String,
    /// Opaque routing token for this device, assigned at registration.
    pub routing_token: String,
}

pub struct RelayClient {
    config: ClientConfig,
    transport: Transport,
}

impl std::fmt::Debug for RelayClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelayClient")
            .field("relay_url", &self.config.relay_url)
            .finish_non_exhaustive()
    }
}

impl RelayClient {
    /// Connect to the relay and complete the (stub) handshake.
    pub async fn connect(config: ClientConfig) -> Result<Self> {
        let transport = Transport::connect(&config.relay_url).await?;
        Ok(Self { config, transport })
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Send an already-encrypted envelope to the relay.
    pub async fn send(&mut self, envelope: &RelayEnvelope) -> Result<()> {
        self.transport.send(envelope).await
    }

    /// Await the next inbound envelope.
    pub async fn next(&mut self) -> Result<Option<RelayEnvelope>> {
        self.transport.recv().await
    }

    pub async fn close(&mut self) -> Result<()> {
        self.transport.close().await
    }
}
