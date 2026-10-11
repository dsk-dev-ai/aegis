use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfiError {
    #[error(transparent)]
    Crypto(#[from] veil_crypto::CryptoError),
    #[error(transparent)]
    Core(#[from] veil_core::CoreError),
    #[error(transparent)]
    Storage(#[from] veil_storage::StorageError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, FfiError>;
