//! # aegis-core
//!
//! Protocol-level building blocks for Aegis: identity, conversations, message
//! model, the encrypted relay envelope, and the seams for the 1:1 (X3DH +
//! Double Ratchet) and group (MLS) session implementations.
//!
//! This crate is transport-agnostic and platform-agnostic. The Flutter apps
//! and the relay server both build on the same types.

#![forbid(unsafe_code)]

pub mod conversation;
pub mod error;
pub mod group;
pub mod identity;
pub mod message;
pub mod relay;
pub mod session;

pub use conversation::{ConversationId, ConversationKind};
pub use error::{CoreError, Result};
pub use identity::{AccountId, DeviceId, Endpoint, Username};
pub use message::{Content, MessageId, PlainMessage, ReceiptState};
pub use relay::RelayEnvelope;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_validation() {
        assert!(Username::parse("Alice").unwrap().as_str() == "alice");
        assert!(Username::parse("ab").is_err());
        assert!(Username::parse("has space").is_err());
        assert!(Username::parse("nice_user_1").is_ok());
    }

    #[test]
    fn message_roundtrips_through_json() {
        let msg = PlainMessage {
            id: uuid::Uuid::new_v4(),
            conversation: ConversationId::random(),
            sent_at_ms: 1_700_000_000_000,
            content: Content::Text {
                body: "hello".into(),
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: PlainMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn relay_envelope_hides_plaintext() {
        let ct = b"opaque-ciphertext";
        let env = RelayEnvelope::new("routing-token", ct);
        assert_eq!(env.from, None);
        assert_eq!(env.ciphertext().unwrap(), ct);
        assert!(!env.payload.contains("opaque"));
    }
}
