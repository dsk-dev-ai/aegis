# Veil Architecture

## Goals

- **Security-first**: E2EE by default, minimal trust in server, no homegrown crypto.
- **Portable**: share one Rust security core across mobile and desktop.
- **Lightweight & fast**: avoid bundling heavy runtimes (Rust core + Flutter/Tauri).
- **Open and self-hostable**: inspectable, reproducible, verifiable builds.

## System view

```text
+-------------------------------+     +-----------------------------+
|          Clients             |     |           Relay            |
|  Flutter (Android/iOS)       |     |   veil-server (axum, WS)   |
|  Flutter (Windows/macOS/Lin) |<--->|   opaque store-and-forward |
+-------------------------------+     +-----------------------------+
                ^                            | (push: APNs/FCM)
                |                            v
           Rust FFI (veil-ffi)            push providers
           (veil-crypto/core/storage/net)
```

## Crate responsibilities

| Crate | Purpose |
|---|---|
| `veil-crypto` | Primitives: ed25519/x25519, ChaCha20-Poly1305, HKDF, safety numbers, QR payloads. No network I/O. `#![forbid(unsafe_code)]`. |
| `veil-core` | Protocol types (AccountId, Endpoint, ConversationId, messages, RelayEnvelope) and seams: `Session` (1:1) and `GroupSession` (MLS). |
| `veil-net` | WebSocket transport client to the relay. No TLS in core crate (can be layered). |
| `veil-storage` | Encrypted local KV using `redb`. All writes encrypted with AEAD under the master key. |
| `veil-ffi` | Stable API surface exposed to Flutter. Zero-unsafe, small surface, testable standalone. |
| `veil-server` | Minimal store-and-forward relay. Stores only opaque envelopes; never sees plaintext. |

## Protocol plan

- **1:1**: Signal Protocol — X3DH for initial handshake, Double Ratchet for continuous forward secrecy and post-compromise security. Concrete implementation via a vetted library (`libsignal-client`) later, accessed through `veil-core`'s `Session` trait.
- **Groups**: MLS (RFC 9420). Epoch-based group keys with efficient updates. Prefer `openmls` with careful credential/identity binding.
- **Identity verification**: safety numbers + QR to compare fingerprints.
- **Multi-device**: device linking QR + E2EE key transfer.
- **Disappearing messages**: timer enforced by client; server drops expired envelopes (TTL). 

## Security considerations

- **Secrets**: private keys never leave the secure element/keystore; zeroized in memory on drop (where feasible).
- **Local encryption**: ChaCha20-Poly1305 with per-record nonce.
- **Replay/mitm**: handled by protocol layers (ratchet, MLS). Contact-add requires out-of-band verification of safety number.
- **Metadata**: relay deletes undelivered envelopes after TTL; sealed sender planned to hide sender identity.
- **Push**: uses APNs/FCM — unavoidable third party; minimize content sent.
- **No logging of plaintext**: audit logging stays metadata-free.

## Data flow (1:1, high level)

1. Peer scanned `veil://add` QR → decode prekey bundle.
2. Client initiates session (X3DH) → generates ciphertext.
3. Wrap in `RelayEnvelope(to, payload=E2EE, ttl_secs=...)` → send via `veil-net` to relay.
4. Relay stores until recipient connects (or TTL expires).
5. Recipient decrypts with established Double Ratchet session.
6. Delivery/read receipts encrypted same way.
