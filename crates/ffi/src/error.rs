use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfiError {
    #[error(transparent)]
    Crypto(#[from] aegis_crypto::CryptoError),
    #[error(transparent)]
    Core(#[from] aegis_core::CoreError),
    #[error(transparent)]
    Storage(#[from] aegis_storage::StorageError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, FfiError>;
