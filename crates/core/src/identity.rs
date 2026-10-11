//! Account and device identity.
//!
//! Aegis does not use phone numbers. A user is identified by a **username**
//! plus a **random per-account identifier**, and each device has its own
//! key material. Devices are linked via a QR pairing flow.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A human-chosen handle, e.g. `alice`. Normalised to lowercase.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    /// Validate and normalise a username: 3..=32 chars, `[a-z0-9_]`.
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let normalised = raw.trim().to_ascii_lowercase();
        let valid_len = (3..=32).contains(&normalised.len());
        let valid_chars = normalised
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_len || !valid_chars {
            return Err(CoreError::Malformed(format!("invalid username: {raw:?}")));
        }
        Ok(Self(normalised))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An opaque, random account identifier. Stable across username changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(Uuid);

impl AccountId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// Identifies a specific device belonging to an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(Uuid);

impl DeviceId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

/// A fully-qualified recipient: account + device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Endpoint {
    pub account: AccountId,
    pub device: DeviceId,
}
