//! Encoding of identity/prekey material into shareable, QR-friendly payloads.
//!
//! A contact-add QR encodes a `aegis://add` URI carrying a [`PreKeyBundle`].
//! Scanning it lets a peer begin an encrypted session immediately, with no
//! phone number or central directory lookup.

use crate::error::{CryptoError, Result};
use crate::keys::PreKeyBundle;
use base64::Engine;

pub const ADD_URI_PREFIX: &str = "aegis://add?d=";

const URL_B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Serialize a prekey bundle into a compact `aegis://add?d=` URI.
pub fn encode_prekey_bundle(bundle: &PreKeyBundle) -> Result<String> {
    let json = serde_json::to_vec(bundle).map_err(|e| CryptoError::Encoding(e.to_string()))?;
    Ok(format!("{ADD_URI_PREFIX}{}", URL_B64.encode(json)))
}

/// Parse a `aegis://add?d=` URI back into a [`PreKeyBundle`].
pub fn decode_prekey_bundle(uri: &str) -> Result<PreKeyBundle> {
    let data = uri
        .strip_prefix(ADD_URI_PREFIX)
        .ok_or_else(|| CryptoError::Encoding("unrecognized QR payload".into()))?;
    let bytes = URL_B64
        .decode(data)
        .map_err(|e| CryptoError::Encoding(e.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|e| CryptoError::Encoding(e.to_string()))
}
