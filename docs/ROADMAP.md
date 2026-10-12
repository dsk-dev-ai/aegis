# Roadmap

| Phase | Goals | Status |
|---|---|---|
| 0 (Week 0–2) | Spec, threat model, architecture, repo skeleton, CI. | Done |
| 1 (MVP, 2–6 months) | Rust core complete, X3DH+Double Ratchet integrated (vetted lib), 1:1 text/media E2EE, contact QR + safety numbers, encrypted local storage, multi-device linking basics, Flutter mobile alpha. | In progress |
| 2 (Stability, 2–4 months) | MLS groups, voice/video (WebRTC + E2EE), disappearing messages, reactions, file transfer, desktop clients (Flutter). | Planned |
| 3 (Privacy, 1–3 months) | Sealed sender, usernames/discovery, invite links, push hardening, self-host docs, reproducible builds. | Planned |
| 4 (Hardening, 2–4 months) | External security audit, fuzzing, formal review of critical paths, post-quantum hybrid, federation/self-host production guides. | Planned |

See also: [REQUIREMENTS.md](REQUIREMENTS.md) (tracked requirements) and
[WORKFLOW.md](WORKFLOW.md) (branch/PR process).

## v1.0.0 exit criteria

Tracked as requirement IDs in [REQUIREMENTS.md](REQUIREMENTS.md). v1.0.0 ships
when FR-1..FR-10 and NFR-1..NFR-6, NFR-9 are `done` on Android, iOS and Web.

## Progress notes

- Mobile/web Flutter client with real X25519 + HKDF + ChaCha20-Poly1305 E2EE,
  QR invites, safety numbers, and relay transport: **working slice shipped**
  (web verified end-to-end; Android via emulator).
- Relay routing with offline queue/flush: **done**.

## Immediate next steps

1. **Android bring-up**: build + install on the Pixel API 30 emulator; verify
   messaging emulator ↔ web through the relay.
2. **Protocol libraries**: integrate `libsignal-client` (or equivalent) for 1:1
   (X3DH + Double Ratchet); evaluate `openmls` for groups.
3. **Flutter bridge**: run `flutter_rust_bridge_codegen` to generate Dart
   bindings for `aegis-ffi` (replace the parallel Dart crypto path).
4. **Encrypted local storage**: swap `SharedPreferences` for the encrypted store.
5. **Relay hardening**: TLS, rate limiting, message expiry/pruning, persistence.
6. **Keystore integration**: platform-specific for master keys.
7. **Desktop clients**: enable Linux/Windows/macOS builds.
8. **Security audit**: commission before public release.
