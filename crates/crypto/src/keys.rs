//! Identity and key-agreement key material.
//!
//! Aegis uses two long-term key pairs per device:
//! * an **identity key** (Ed25519) used to sign and authenticate,
//! * an **exchange key** (X25519) used for Diffie-Hellman key agreement.
//!
//! 1:1 sessions will be established with X3DH + the Double Ratchet and group
//! sessions with MLS (RFC 9420). This module only provides the primitives and
//! the serialisable key types those layers are built on.

use crate::error::{CryptoError, Result};
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey as ExchangePublicKey, StaticSecret};

const ED25519_PUBLIC_LEN: usize = 32;
const ED25519_SIGNATURE_LEN: usize = 64;

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

/// A long-term Ed25519 signing key pair.
pub struct IdentityKeyPair {
    signing: SigningKey,
}

impl IdentityKeyPair {
    pub fn generate(rng: &mut impl CryptoRngCore) -> Self {
        Self {
            signing: SigningKey::generate(rng),
        }
    }

    pub fn from_seed(seed: &[u8; ED25519_PUBLIC_LEN]) -> Self {
        Self {
            signing: SigningKey::from_bytes(seed),
        }
    }

    pub fn to_seed(&self) -> [u8; ED25519_PUBLIC_LEN] {
        self.signing.to_bytes()
    }

    pub fn public(&self) -> IdentityPublicKey {
        IdentityPublicKey(self.signing.verifying_key())
    }

    pub fn sign(&self, message: &[u8]) -> [u8; ED25519_SIGNATURE_LEN] {
        self.signing.sign(message).to_bytes()
    }
}

impl std::fmt::Debug for IdentityKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityKeyPair")
            .field("public", &self.public())
            .finish_non_exhaustive()
    }
}

/// The public half of an [`IdentityKeyPair`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IdentityPublicKey(pub(crate) VerifyingKey);

impl IdentityPublicKey {
    pub fn from_bytes(bytes: &[u8; ED25519_PUBLIC_LEN]) -> Result<Self> {
        let vk =
            VerifyingKey::from_bytes(bytes).map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
        Ok(Self(vk))
    }

    pub fn as_bytes(&self) -> [u8; ED25519_PUBLIC_LEN] {
        self.0.to_bytes()
    }

    pub fn verify(&self, message: &[u8], signature: &[u8; ED25519_SIGNATURE_LEN]) -> Result<()> {
        let sig = Signature::from_bytes(signature);
        self.0
            .verify(message, &sig)
            .map_err(|_| CryptoError::Signature)
    }
}

impl std::fmt::Debug for IdentityPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityPublicKey({})", self.to_base64())
    }
}

impl IdentityPublicKey {
    pub fn to_base64(&self) -> String {
        B64.encode(self.0.to_bytes())
    }
}

impl std::fmt::Display for IdentityPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base64())
    }
}

/// A long-term X25519 key pair for Diffie-Hellman key agreement.
pub struct ExchangeKeyPair {
    secret: StaticSecret,
}

impl ExchangeKeyPair {
    pub fn generate(rng: &mut impl CryptoRngCore) -> Self {
        Self {
            secret: StaticSecret::random_from_rng(rng),
        }
    }

    pub fn from_secret(secret: [u8; 32]) -> Self {
        Self {
            secret: StaticSecret::from(secret),
        }
    }

    pub fn to_secret(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    pub fn public(&self) -> ExchangePublicKey {
        ExchangePublicKey::from(&self.secret)
    }

    /// Perform X25519 Diffie-Hellman with a peer's public key, returning the
    /// raw 32-byte shared secret. Feed this through HKDF before use as a key.
    pub fn diffie_hellman(&self, peer: &ExchangePublicKey) -> [u8; 32] {
        self.secret.diffie_hellman(peer).to_bytes()
    }
}

impl std::fmt::Debug for ExchangeKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExchangeKeyPair(<redacted>)")
    }
}

/// Derive a short, human-comparable "safety number" from two identity keys.
///
/// Users compare this out-of-band (or via QR) to defeat man-in-the-middle
/// attacks during first contact. It is intentionally derived from both
/// identities in a canonical order so both sides compute the same value.
pub fn safety_number(a: &IdentityPublicKey, b: &IdentityPublicKey) -> String {
    let (x, y) = {
        let (ab, bb) = (a.as_bytes(), b.as_bytes());
        if ab <= bb {
            (ab, bb)
        } else {
            (bb, ab)
        }
    };

    let mut seed = Vec::with_capacity(64 + 32);
    seed.extend_from_slice(b"aegis-safety-number-v1");
    seed.extend_from_slice(&x);
    seed.extend_from_slice(&y);

    // Expand to 60 digits (12 groups of 5), Signal-style, by hashing with a
    // counter until enough digits are produced.
    let mut digits = String::with_capacity(60);
    let mut counter: u32 = 0;
    while digits.len() < 60 {
        let mut h = Sha256::new();
        h.update(&seed);
        h.update(counter.to_be_bytes());
        for byte in h.finalize() {
            if digits.len() >= 60 {
                break;
            }
            digits.push(char::from(b'0' + (byte % 10)));
        }
        counter += 1;
    }

    digits
        .as_bytes()
        .chunks(5)
        .map(|c| std::str::from_utf8(c).expect("ascii digits"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A public prekey bundle published to the relay so a peer can start a session
/// without you being online. Fits comfortably in a QR code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreKeyBundle {
    pub identity_key: String,
    pub exchange_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_prekey: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_prekey_signature: Option<String>,
}
