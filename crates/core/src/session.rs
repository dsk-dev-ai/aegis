//! 1:1 session abstraction.
//!
//! The production implementation will be **X3DH + Double Ratchet** (via a
//! vetted library such as `libsignal-client`) providing forward secrecy and
//! post-compromise security. This trait is the seam the rest of the app codes
//! against so the concrete protocol can be swapped without touching UI or
//! storage code.
//!
//! NOTE: no home-grown ratchet is provided here on purpose. Rolling your own
//! key schedule is a common and severe failure mode.

use crate::error::Result;
use std::collections::HashMap;

/// Per-message metadata needed to decrypt, sent alongside the ciphertext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionHeader {
    /// Public key / ratchet step identifier.
    pub ratchet_key: Vec<u8>,
    /// Message counter within the current chain.
    pub counter: u32,
    /// Previous chain length (for out-of-order handling).
    pub previous_chain_len: u32,
}

/// An established, stateful encrypted session with a single peer device.
pub trait Session {
    /// Encrypt a plaintext with associated data; returns ciphertext + header.
    fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, SessionHeader)>;

    /// Decrypt a received message.
    fn decrypt(&mut self, header: &SessionHeader, ciphertext: &[u8], aad: &[u8])
        -> Result<Vec<u8>>;
}

/// In-memory session registry keyed by peer device id. The persistent,
/// encrypted implementation lives in `veil-storage`.
#[derive(Default)]
pub struct SessionStore {
    sessions: HashMap<[u8; 16], Box<dyn Session>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, device: [u8; 16], session: Box<dyn Session>) {
        self.sessions.insert(device, session);
    }

    pub fn get_mut(&mut self, device: &[u8; 16]) -> Option<&mut Box<dyn Session>> {
        self.sessions.get_mut(device)
    }

    pub fn contains(&self, device: &[u8; 16]) -> bool {
        self.sessions.contains_key(device)
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

impl std::fmt::Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionStore")
            .field("sessions", &self.sessions.len())
            .finish()
    }
}
