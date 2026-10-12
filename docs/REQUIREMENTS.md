# Requirements

Tracked requirements for Aegis. **FR** = functional, **NFR** = non-functional.
Status: `done`, `wip` (in progress), `todo`.

## v1.0.0 scope (definition of the first stable release)

### Functional

| ID | Requirement | Status |
|---|---|---|
| FR-1 | Passwordless identity: username + device keys, no phone/email | done |
| FR-2 | Contact discovery via QR / `aegis://add` invite code | done |
| FR-3 | 60-digit safety-number verification (MITM defence) | done |
| FR-4 | 1:1 end-to-end encrypted text messaging | done |
| FR-5 | Message delivery through a relay (queue when recipient offline) | done |
| FR-6 | Encrypted local storage of identity, contacts, messages | wip |
| FR-7 | Android app | wip |
| FR-8 | Web app | done |
| FR-9 | Desktop app (Linux/Windows/macOS) | todo |
| FR-10 | Host verification / relay setting (self-host) | done |
| FR-11 | Attachments (images, files) | todo |
| FR-12 | Group messaging (MLS, RFC 9420) | todo |
| FR-13 | Voice/video calls (WebRTC + E2EE) | todo |
| FR-14 | Disappearing messages | todo |
| FR-15 | Multi-device linking | todo |
| FR-16 | Push notifications | todo |
| FR-17 | Account/contact backup & restore | todo |

### Non-functional

| ID | Requirement | Status |
|---|---|---|
| NFR-1 | E2EE on every message; no plaintext to the relay | done |
| NFR-2 | Rust core for crypto/protocol; thin UI layer | wip |
| NFR-3 | Pure-Rust deps where possible (no C toolchain needed) | done |
| NFR-4 | Runnable on web, mobile, desktop from one Flutter codebase | wip |
| NFR-5 | Deterministic crypto test vectors shared Rust ↔ Dart | done |
| NFR-6 | CI: fmt, clippy, test, analyze (informational) | done |
| NFR-7 | Reproducible builds | todo |
| NFR-8 | External security audit before 1.0 | todo |
| NFR-9 | Forward secrecy (X3DH + Double Ratchet) | todo |
| NFR-10 | Post-quantum hybrid KEM (roadmap) | todo |

## Security requirements

- Identity: Ed25519 signing + X25519 key agreement per device.
- Session key: HKDF-SHA256 bound to both identities.
- AEAD: ChaCha20-Poly1305 with AAD binding identities.
- Wire format: opaque ciphertext to the relay; recipient routing by random token.
- Local secrets: platform keystore (Android Keystore / iOS Keychain / OS keyring).
- Relay: TLS (wss://), rate limiting, message TTL/pruning.

## Platform matrix (target)

| Platform | Client | Status |
|---|---|---|
| Android | Flutter | wip (emulator) |
| iOS | Flutter | todo |
| Web | Flutter | done |
| Linux | Flutter desktop | todo |
| Windows | Flutter desktop | todo |
| macOS | Flutter desktop | todo |
| Self-host relay | Rust (axum) | done |

## Out of scope for v1.0.0

- Federation between independent relays.
- Anonymous payments / monetisation.
- Bots / public channels.
