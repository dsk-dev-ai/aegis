use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid key material: {0}")]
    InvalidKey(String),
    #[error("invalid or corrupted ciphertext")]
    Decrypt,
    #[error("signature verification failed")]
    Signature,
    #[error("encoding error: {0}")]
    Encoding(String),
    #[error("random number generator failure")]
    Rng,
}

pub type Result<T> = std::result::Result<T, CryptoError>;
