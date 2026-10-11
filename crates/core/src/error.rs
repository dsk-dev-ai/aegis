use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Crypto(#[from] aegis_crypto::CryptoError),
    #[error("session not established for peer {0}")]
    NoSession(String),
    #[error("group error: {0}")]
    Group(String),
    #[error("malformed message: {0}")]
    Malformed(String),
    #[error("encoding error: {0}")]
    Encoding(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
