use crate::conversation::ConversationId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type MessageId = Uuid;

/// A decrypted, application-level message. This never leaves the device in
/// plaintext form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlainMessage {
    pub id: MessageId,
    pub conversation: ConversationId,
    pub sent_at_ms: u64,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Content {
    Text {
        body: String,
    },
    /// Reference to out-of-band encrypted media (blob stored on a CDN or
    /// self-hosted object store). `key` is the per-file AEAD key, itself
    /// transported inside this encrypted message.
    Attachment {
        media_type: String,
        key: String,
        digest: String,
        size: u64,
    },
    Reaction {
        target: MessageId,
        emoji: String,
    },
    Receipt {
        target: MessageId,
        state: ReceiptState,
    },
    Typing {
        active: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptState {
    Delivered,
    Read,
}
