//! # veil-net
//!
//! Transport to the Veil relay over WebSocket, carrying opaque
//! [`veil_core::RelayEnvelope`]s. The client never sends plaintext.

#![forbid(unsafe_code)]

pub mod client;
pub mod error;
pub mod transport;

pub use client::{ClientConfig, RelayClient};
pub use error::{NetError, Result};
pub use transport::Transport;
