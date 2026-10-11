//! # aegis-net
//!
//! Transport to the Aegis relay over WebSocket, carrying opaque
//! [`aegis_core::RelayEnvelope`]s. The client never sends plaintext.

#![forbid(unsafe_code)]

pub mod client;
pub mod error;
pub mod transport;

pub use client::{ClientConfig, RelayClient};
pub use error::{NetError, Result};
pub use transport::Transport;
