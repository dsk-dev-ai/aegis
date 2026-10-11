use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("failed to connect: {0}")]
    Connect(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("core error: {0}")]
    Core(#[from] veil_core::CoreError),
}

pub type Result<T> = std::result::Result<T, NetError>;
