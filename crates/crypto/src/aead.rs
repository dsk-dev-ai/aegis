//! Authenticated encryption (ChaCha20-Poly1305).
//!
//! ChaCha20-Poly1305 is chosen over AES-GCM for constant-time software
//! performance across all target platforms and resistance to cache-timing
//! attacks. Nonces must never repeat for a given key; callers should let
//! [`AeadKey::encrypt_random`] generate them.

use crate::error::{CryptoError, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use rand_core::CryptoRngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

/// A symmetric key for authenticated encryption.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct AeadKey([u8; KEY_LEN]);

impl AeadKey {
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(bytes)
    }

    pub fn generate(rng: &mut impl CryptoRngCore) -> Self {
        let mut key = [0u8; KEY_LEN];
        rng.fill_bytes(&mut key);
        Self(key)
    }

    fn cipher(&self) -> ChaCha20Poly1305 {
        ChaCha20Poly1305::new(Key::from_slice(&self.0))
    }

    /// Encrypt with an explicit nonce. The caller MUST guarantee nonce
    /// uniqueness per key.
    pub fn encrypt(
        &self,
        nonce: &[u8; NONCE_LEN],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        self.cipher()
            .encrypt(
                Nonce::from_slice(nonce),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::Decrypt)
    }

    pub fn decrypt(
        &self,
        nonce: &[u8; NONCE_LEN],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        self.cipher()
            .decrypt(
                Nonce::from_slice(nonce),
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::Decrypt)
    }

    /// Encrypt with a freshly generated random nonce, returned alongside the
    /// ciphertext so it can travel with the message.
    pub fn encrypt_random(
        &self,
        rng: &mut impl CryptoRngCore,
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, [u8; NONCE_LEN])> {
        let mut nonce = [0u8; NONCE_LEN];
        rng.fill_bytes(&mut nonce);
        let ct = self.encrypt(&nonce, aad, plaintext)?;
        Ok((ct, nonce))
    }
}

impl std::fmt::Debug for AeadKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AeadKey(<redacted>)")
    }
}
