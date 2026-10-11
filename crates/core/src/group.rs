//! Group session abstraction backed by **MLS (RFC 9420)**.
//!
//! MLS provides efficient group key agreement with forward secrecy and
//! post-compromise security, scalable to large groups. The production
//! implementation will use a vetted MLS library (e.g. `openmls`); this trait
//! is the integration seam.

use crate::error::Result;

/// MLS epoch (group state version). Bumps on every membership change.
pub type Epoch = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMember {
    /// Serialized MLS credential / identity public key.
    pub credential: Vec<u8>,
}

/// A stateful group session. Membership changes are commits that advance the
/// epoch and produce a Welcome message for newly added members.
pub trait GroupSession {
    fn epoch(&self) -> Epoch;

    fn members(&self) -> Vec<GroupMember>;

    /// Add a member; returns the commit payload to broadcast.
    fn add_member(&mut self, member: GroupMember) -> Result<Vec<u8>>;

    /// Remove a member by credential; returns the commit payload.
    fn remove_member(&mut self, credential: &[u8]) -> Result<Vec<u8>>;

    fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;

    fn decrypt(&mut self, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
}

/// An encrypted group message (MLS `PrivateMessage`) plus the epoch it belongs
/// to. During key changes, clients must keep old epochs to decrypt in-flight
/// messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMessage {
    pub epoch: Epoch,
    pub ciphertext: Vec<u8>,
}
