//! The stable API the Flutter apps call into.
//!
//! Every function here is intentionally free of platform types so it can be
//! surfaced through `flutter_rust_bridge` (see `flutter_rust_bridge.yaml`)
//! without changes. Keep this surface small and well-documented.

use aegis_core::{DeviceId, Username};
use aegis_crypto::{
    decode_prekey_bundle, encode_prekey_bundle, safety_number, ExchangeKeyPair, IdentityKeyPair,
    IdentityPublicKey, PreKeyBundle,
};
use aegis_storage::EncryptedStore;
use base64::Engine;

use crate::error::{FfiError, Result};

const B64: base64::engine::general_purpose::GeneralPurpose =
    base64::engine::general_purpose::STANDARD_NO_PAD;

/// A local account: the long-term identity + exchange keys for this device.
pub struct Account {
    identity: IdentityKeyPair,
    exchange: ExchangeKeyPair,
}

impl Account {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            identity: IdentityKeyPair::generate(&mut rng),
            exchange: ExchangeKeyPair::generate(&mut rng),
        }
    }

    /// Recreate an account from a 32-byte secret seed (store this in the
    /// platform keystore; never in plain files).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            identity: IdentityKeyPair::from_seed(&seed),
            exchange: ExchangeKeyPair::generate(&mut rng),
        }
    }

    /// Public identity key, base64.
    pub fn identity_key(&self) -> String {
        self.identity.public().to_base64()
    }

    /// Build a `aegis://add?...` URI to render as a QR code for adding this
    /// account as a contact.
    pub fn contact_qr_uri(&self) -> Result<String> {
        let bundle = PreKeyBundle {
            identity_key: self.identity.public().to_base64(),
            exchange_key: B64.encode(self.exchange.public().to_bytes()),
            signed_prekey: None,
            signed_prekey_signature: None,
        };
        Ok(encode_prekey_bundle(&bundle)?)
    }

    /// Decode a scanned contact QR and return the peer's identity key.
    pub fn parse_contact_qr(uri: &str) -> Result<String> {
        let bundle =
            decode_prekey_bundle(uri).map_err(|e| FfiError::InvalidInput(e.to_string()))?;
        Ok(bundle.identity_key)
    }

    /// Human-comparable safety number against a peer's base64 identity key.
    pub fn safety_number_with(&self, peer_identity_key_b64: &str) -> Result<String> {
        let bytes = B64
            .decode(peer_identity_key_b64.trim())
            .map_err(|e| FfiError::InvalidInput(e.to_string()))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| FfiError::InvalidInput("identity key must be 32 bytes".into()))?;
        let peer = IdentityPublicKey::from_bytes(&arr)?;
        Ok(safety_number(&self.identity.public(), &peer))
    }

    /// Validate a username against Aegis's rules.
    pub fn validate_username(raw: &str) -> Result<String> {
        Ok(Username::parse(raw)?.as_str().to_string())
    }
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("identity", &self.identity.public())
            .finish_non_exhaustive()
    }
}

/// Encrypted, local persistence exposed to the UI.
pub struct Vault {
    store: EncryptedStore,
}

impl Vault {
    pub fn open(path: &str, master_key: [u8; 32]) -> Result<Self> {
        let key = aegis_crypto::AeadKey::from_bytes(master_key);
        Ok(Self {
            store: EncryptedStore::open(path, key)?,
        })
    }

    pub fn put(&self, name: &str, value: &[u8]) -> Result<()> {
        Ok(self.store.put(name, value)?)
    }

    pub fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.store.get(name)?)
    }

    pub fn delete(&self, name: &str) -> Result<()> {
        Ok(self.store.delete(name)?)
    }
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Vault(<encrypted>)")
    }
}

/// Information identifying this device to the relay.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device: DeviceId,
    pub username: Option<String>,
}

impl DeviceInfo {
    pub fn new() -> Self {
        Self {
            device: DeviceId::random(),
            username: None,
        }
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self::new()
    }
}
