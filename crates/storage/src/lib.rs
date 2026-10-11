//! # aegis-storage
//!
//! Encrypted, local-only persistence. Data is encrypted before it is written
//! to disk, so copying the database file reveals nothing without the master
//! key.

#![forbid(unsafe_code)]

pub mod error;
pub mod store;

pub use error::{Result, StorageError};
pub use store::EncryptedStore;

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_crypto::AeadKey;

    #[test]
    fn put_get_delete_roundtrip() {
        let dir = std::env::temp_dir().join(format!("aegis-store-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let mut rng = rand::thread_rng();
        let key = AeadKey::generate(&mut rng);

        let store = EncryptedStore::open(&dir, key).unwrap();
        store.put("msg:1", b"top secret").unwrap();
        assert_eq!(store.get("msg:1").unwrap().unwrap(), b"top secret");
        assert!(store.get("missing").unwrap().is_none());

        store.delete("msg:1").unwrap();
        assert!(store.get("msg:1").unwrap().is_none());

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn wrong_key_cannot_decrypt() {
        let dir = std::env::temp_dir().join(format!("aegis-store-wrong-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let mut rng = rand::thread_rng();

        {
            let store = EncryptedStore::open(&dir, AeadKey::generate(&mut rng)).unwrap();
            store.put("k", b"v").unwrap();
        }
        {
            let store = EncryptedStore::open(&dir, AeadKey::generate(&mut rng)).unwrap();
            assert!(store.get("k").is_err());
        }
        let _ = std::fs::remove_file(&dir);
    }
}
