use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage backend error: {0}")]
    Backend(String),
    #[error("stored record is corrupt")]
    Corrupt,
    #[error(transparent)]
    Crypto(#[from] aegis_crypto::CryptoError),
}

pub type Result<T> = std::result::Result<T, StorageError>;

pub(crate) fn map_err<E: std::fmt::Display>(e: E) -> StorageError {
    StorageError::Backend(e.to_string())
}
