# Aegis

Aegis is a private, end-to-end encrypted messenger built to be fast, lightweight, open-source, and secure. Users are identified by **username + QR** (no phone number required). Encryption defaults to E2EE with **Signal Protocol for 1:1** and **MLS (RFC 9420) for groups**.

## Principles

- **E2EE by default** — no plaintext fallback.
- **Don't roll your own crypto** — built on audited primitives; protocol seams (MLS/X3DH) allow swapping in vetted libraries.
- **Minimal metadata** — design assumes an adversarial relay. Sealed sender is the target.
- **One security core** — all critical logic lives in Rust (`aegis-crypto`, `aegis-core`, `aegis-ffi`) shared across Flutter mobile and Tauri/Flutter desktop.
- **Self-hostable** — the relay (`aegis-server`) is a small store-and-forward service for opaque ciphertext.

## Repository layout

```text
/aegis
├─ Cargo.toml          # Rust workspace
├─ crates/
│  ├─ crypto/          # Symmetric/asymmetric, AEAD, KDF, safety numbers, QR
│  ├─ core/            # Identity, conversations, relay envelope, protocol seams
│  ├─ net/             # WebSocket relay client
│  ├─ storage/         # Encrypted local KV (redb)
│  ├─ ffi/             # Stable UI-facing API for Flutter
│  └─ server/          # Opaque store-and-forward relay (axum)
├─ apps/
│  ├─ mobile/          # Flutter app (Android/iOS)
│  └─ desktop/         # Flutter desktop (Windows/macOS/Linux)
├─ docs/               # Architecture, threat model, crypto, roadmap
├─ docker/             # Dev/prod containers
└─ .github/workflows/  # CI
```

## Building

### Prerequisites

- Rust 1.75+ (`rustup toolchain stable`)
- Flutter 3.24+ (`flutter doctor`)
- Docker (optional, for relay/container dev)

### Rust workspace

```bash
cd /path/to/aegis
cargo build            # build all crates
cargo test             # run unit tests (10+ passing)
cargo run -p aegis-server  # run relay on http://127.0.0.1:8080
```

### Flutter apps

```bash
cd apps/mobile
flutter pub get
flutter run -d android  # or ios

cd apps/desktop
flutter pub get
flutter run -d linux    # or windows/macos
```

## Security model (high level)

- **Identity**: Ed25519 (signing), X25519 (key agreement). Username+random account ID; no phone numbers.
- **Local storage**: all persistent data encrypted with ChaCha20-Poly1305 under a master key derived from the user's unlock secret; keys live in the OS keystore.
- **1:1**: X3DH + Double Ratchet (protocol seam in `aegis-core`).
- **Groups**: MLS (RFC 9420).
- **QR sharing**: `aegis://add?d=...` encodes a compact prekey bundle; safety numbers compare fingerprints out-of-band.
- **Relay**: sees only routing and opaque ciphertext; TTL-based store-and-forward; sealed sender design to minimize metadata (planned).

## Current status

**MVP skeleton** is scaffolded: crypto primitives, protocol types, encrypted storage, WebSocket transport, FFI API surface, relay server, and Flutter app shells. The workspace compiles and unit tests pass. The next phase is implementing a concrete X3DH+Double Ratchet and integrating MLS (e.g., `libsignal-client` via FFI or `openmls`), followed by an external security audit before any real users.

## License

AGPL-3.0-or-later. See `LICENSE`.
