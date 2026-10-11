# Roadmap

| Phase | Goals | Status |
|---|---|---|
| 0 (Week 0–2) | Spec, threat model, architecture, repo skeleton, CI. | Done |
| 1 (MVP, 2–6 months) | Rust core complete, X3DH+Double Ratchet integrated (vetted lib), 1:1 text/media E2EE, contact QR + safety numbers, encrypted local storage, multi-device linking basics, Flutter mobile alpha. | In progress |
| 2 (Stability, 2–4 months) | MLS groups, voice/video (WebRTC + E2EE), disappearing messages, reactions, file transfer, desktop clients (Flutter). | Planned |
| 3 (Privacy, 1–3 months) | Sealed sender, usernames/discovery, invite links, push hardening, self-host docs, reproducible builds. | Planned |
| 4 (Hardening, 2–4 months) | External security audit, fuzzing, formal review of critical paths, post-quantum hybrid, federation/self-host production guides. | Planned |

## Immediate next steps

1. **Protocol libraries**: integrate `libsignal-client` (or equivalent) for 1:1; evaluate `openmls` for groups.
2. **Flutter bridge**: run `flutter_rust_bridge_codegen` to generate Dart bindings for `aegis-ffi`.
3. **Mobile UI**: build minimal chat list + conversation + QR scanner.
4. **Relay hardening**: TLS, rate limiting, message expiry/pruning, persistence (Postgres/ScyllaDB).
5. **Keystore integration**: platform-specific (Android Keystore, iOS Keychain) for master keys.
6. **CI/CD**: lint, test, build artifacts, reproducible builds.
7. **Security audit**: commission before public release.
