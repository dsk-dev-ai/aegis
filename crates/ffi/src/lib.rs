//! # veil-ffi
//!
//! The single, stable API surface that the Flutter apps (mobile and desktop)
//! link against. It composes the crypto, core, and storage crates into
//! UI-friendly operations and is the only crate the UI layer touches.
//!
//! ## Enabling the Flutter bridge
//! This crate currently exposes a plain Rust API so the workspace builds
//! without code generation. To wire it to Dart:
//!
//! ```text
//! cargo install flutter_rust_bridge_codegen
//! flutter_rust_bridge_codegen generate    # reads flutter_rust_bridge.yaml
//! ```
//!
//! That generates `src/frb_generated.rs`; add `mod frb_generated;` below and
//! annotate `api` items with `#[flutter_rust_bridge::frb]` as needed.

#![forbid(unsafe_code)]

pub mod api;
pub mod error;

pub use api::{Account, DeviceInfo, Vault};
pub use error::{FfiError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_qr_roundtrip_and_safety_number() {
        let a = Account::generate();
        let b = Account::generate();

        let uri = a.contact_qr_uri().unwrap();
        let peer_key = Account::parse_contact_qr(&uri).unwrap();
        assert_eq!(peer_key, a.identity_key());

        let n_a = a.safety_number_with(&b.identity_key()).unwrap();
        let n_b = b.safety_number_with(&a.identity_key()).unwrap();
        assert_eq!(n_a, n_b);
    }

    #[test]
    fn vault_roundtrip() {
        let dir = std::env::temp_dir().join(format!("veil-ffi-{}", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let vault = Vault::open(dir.to_str().unwrap(), [7u8; 32]).unwrap();
        vault.put("session:peer", b"state").unwrap();
        assert_eq!(vault.get("session:peer").unwrap().unwrap(), b"state");
        let _ = std::fs::remove_file(&dir);
    }
}
