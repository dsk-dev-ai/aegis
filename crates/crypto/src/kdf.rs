//! Key-derivation helpers (HKDF-SHA256).

use crate::error::{CryptoError, Result};
use hkdf::Hkdf;
use sha2::Sha256;

/// Expand input keying material into `length` bytes under the given salt and
/// domain-separating `info` label.
pub fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
    if length == 0 || length > 255 * 32 {
        return Err(CryptoError::InvalidKey(format!(
            "invalid hkdf output length: {length}"
        )));
    }
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = vec![0u8; length];
    hk.expand(info, &mut okm)
        .map_err(|_| CryptoError::InvalidKey("hkdf expand failed".into()))?;
    Ok(okm)
}

/// Derive a 32-byte AEAD key from a shared secret and a domain label.
pub fn derive_key(shared_secret: &[u8], salt: &[u8], label: &[u8]) -> Result<[u8; 32]> {
    let out = hkdf_sha256(shared_secret, salt, label, 32)?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&out);
    Ok(key)
}
