//! Encrypted key-value store.
//!
//! Every value is encrypted at rest with ChaCha20-Poly1305 under a master key
//! derived from the user's unlock secret (key derivation lives in the app
//! layer). On-disk format per record: `nonce(12) || ciphertext||tag`.

use crate::error::{map_err, Result, StorageError};
use aegis_crypto::{AeadKey, NONCE_LEN};
use redb::{Database, TableDefinition};
use std::path::Path;

const KV: TableDefinition<&str, &[u8]> = TableDefinition::new("aegis.kv");

/// An encrypted, embedded KV store suitable for messages, sessions, and
/// prekeys. Values are opaque bytes; callers serialize their own types.
pub struct EncryptedStore {
    db: Database,
    key: AeadKey,
    aad: Vec<u8>,
}

impl std::fmt::Debug for EncryptedStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedStore").finish_non_exhaustive()
    }
}

impl EncryptedStore {
    /// Open (or create) a store at `path`, encrypting all values under `key`.
    pub fn open(path: impl AsRef<Path>, key: AeadKey) -> Result<Self> {
        let db = Database::create(path).map_err(map_err)?;
        Ok(Self {
            db,
            key,
            aad: b"aegis-store-v1".to_vec(),
        })
    }

    pub fn put(&self, name: &str, value: &[u8]) -> Result<()> {
        let mut rng = rand::thread_rng();
        let (ct, nonce) = self.key.encrypt_random(&mut rng, &self.aad, value)?;
        let mut blob = Vec::with_capacity(NONCE_LEN + ct.len());
        blob.extend_from_slice(&nonce);
        blob.extend_from_slice(&ct);

        let txn = self.db.begin_write().map_err(map_err)?;
        {
            let mut table = txn.open_table(KV).map_err(map_err)?;
            table.insert(name, blob.as_slice()).map_err(map_err)?;
        }
        txn.commit().map_err(map_err)?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let txn = self.db.begin_read().map_err(map_err)?;
        let table = txn.open_table(KV).map_err(map_err)?;
        let Some(guard) = table.get(name).map_err(map_err)? else {
            return Ok(None);
        };
        let bytes = guard.value();
        if bytes.len() < NONCE_LEN {
            return Err(StorageError::Corrupt);
        }
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&bytes[..NONCE_LEN]);
        let plaintext = self.key.decrypt(&nonce, &self.aad, &bytes[NONCE_LEN..])?;
        Ok(Some(plaintext))
    }

    pub fn delete(&self, name: &str) -> Result<()> {
        let txn = self.db.begin_write().map_err(map_err)?;
        {
            let mut table = txn.open_table(KV).map_err(map_err)?;
            table.remove(name).map_err(map_err)?;
        }
        txn.commit().map_err(map_err)?;
        Ok(())
    }
}
