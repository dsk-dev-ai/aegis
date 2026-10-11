//! The envelope the relay server actually sees.
//!
//! The server learns only routing information and opaque ciphertext. When
//! **sealed sender** is enabled, `from` is omitted and the sender's identity is
//! carried (encrypted) inside `payload`, so the server cannot build a social
//! graph.

use base64::Engine;
use serde::{Deserialize, Serialize};

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

const DEFAULT_TTL_SECS: u32 = 60 * 60 * 24 * 14; // 14 days undelivered

fn default_ttl() -> u32 {
    DEFAULT_TTL_SECS
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayEnvelope {
    /// Recipient routing token. Opaque to observers of the payload.
    pub to: String,
    /// Sender routing token; omitted when using sealed sender.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Base64 of the end-to-end ciphertext. The server cannot read this.
    pub payload: String,
    /// Seconds the server retains the message if undelivered.
    #[serde(default = "default_ttl")]
    pub ttl_secs: u32,
}

impl RelayEnvelope {
    pub fn new(to: impl Into<String>, ciphertext: &[u8]) -> Self {
        Self {
            to: to.into(),
            from: None,
            payload: B64.encode(ciphertext),
            ttl_secs: DEFAULT_TTL_SECS,
        }
    }

    pub fn ciphertext(&self) -> Result<Vec<u8>, veil_crypto::CryptoError> {
        B64.decode(&self.payload)
            .map_err(|e| veil_crypto::CryptoError::Encoding(e.to_string()))
    }
}
