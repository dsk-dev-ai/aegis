# Crypto design

## Primitives

| Purpose | Algorithm | Rationale |
|---|---|---|
| Identity (signing) | Ed25519 | Modern, simple, small keys, widely vetted. |
| Key agreement | X25519 | Constant-time, efficient, pair with Ed25519. |
| AEAD | ChaCha20-Poly1305 | No AES hardware assumptions, robust against timing; used for local storage and as message cipher base. |
| KDF | HKDF-SHA256 | Extract-and-expand, standard. |
| Hash | SHA256 | Ubiquitous. |

All randomness is caller-supplied (CSPRNG). `#![forbid(unsafe_code)]` in critical crates; zeroization of secrets where feasible.

## Key hierarchy

- **Identity keypair** (Ed25519): long-term, authenticates owner.
- **Exchange keypair** (X25519): long-term DH key (Signal-style). Can be rotated; prekeys reduce online requirement.
- **Session keys**: derived per ratchet step (ephemeral), never persisted in plaintext.
- **Master unlock key**: derived from user passphrase/biometric (e.g., Argon2id) to encrypt the local database key in the keystore. The database key (`AeadKey`) is stored in platform keystore or wrapped; not on disk in plaintext.

## Safety numbers

Canonical fingerprint derived from both identity keys: HMAC/SHA256 over the pair in sorted order, expanded to 60 digits in 12 groups of 5. Displayed and compared out-of-band (or via QR).

## QR payload

`aegis://add?d=` followed by URL-safe base64 of compact JSON:
```json
{
  "identity_key": "<b64>",
  "exchange_key": "<b64>",
  "signed_prekey": "<b64?>",
  "signed_prekey_signature": "<b64?>"
}
```

Scanning initiates session setup; verification still requires safety number check.

## 1:1 sessions

- **Initial handshake**: X3DH (triple DH). Bundles include signed prekeys for offline peers.
- **Ongoing**: Double Ratchet — root + sending/receiving chains, message keys per message, forward secrecy + post-compromise security.

Implementation: use a vetted library (e.g., [`libsignal-client`](https://github.com/signalapp/libsignal)) via FFI or re-export its types; `aegis-core` provides the `Session` trait seam to keep it swappable.

## Group sessions

- **MLS (RFC 9420)**: ratcheting trees, epoch keys, HPKE. Efficient for large groups; PCS on membership changes.
- Implementation: [`openmls`](https://github.com/openmls/openmls) (Rust). Credentials bound to device identity keys.

## Secure transport

- WebSocket over TLS (WSS). Client currently connects ws:// for dev; production terminates TLS at reverse proxy.
- All application-layer payloads are E2EE; no plaintext in `RelayEnvelope.payload`.

## Local storage format

Each KV record: `nonce(12 bytes) || ciphertext || tag (16 bytes)` from ChaCha20-Poly1305. AAD includes record name/version.

## Post-quantum roadmap

Hybrid KEMs: X25519 + Kyber768 (PQXDH-style) and MLS with hybrid KEMs as they standardize. Designed to be additive (protocol versioning).
