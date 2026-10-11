# Threat model

## Assets

- **Private keys** (identity, exchange, prekeys) — confidentiality and integrity critical.
- **Session keys** — forward secrecy and PCS.
- **Local messages & contacts** — encrypted at rest.
- **Metadata**: who communicates with whom, when, IP, device info.
- **Group membership**.

## Adversaries

| Adversary | Capabilities | Mitigations |
|---|---|---|
| Local attacker (unprivileged) | Read files/cache if unlocked | OS sandbox, encrypted local DB, keystore protected, no plaintext on disk. |
| Compromised OS/root | Full control of device | Cannot recover keys from hardware keystore without auth; still need to mitigate via PCS + rotation. |
| Network attacker (MITM) | Intercept/modify traffic between client and relay | TLS for transport; E2EE authenticates endpoints; safety number verification. |
| Malicious relay | See all envelopes, connection metadata, timing | Envelopes are opaque ciphertext. Planned: sealed sender to hide sender. Minimize logs, short TTL. No contact lists on server. |
| Active collusion (relay + observer) | Correlate timing/IPs | Hard problem; avoid server-side identity of sender; consider transport rotation. |
| Malicious peer | Abusive messages, key spoofing | Safety number verification; block/report; key rotation in groups (MLS). |
| Forensic recovery | Disk images of device | Encrypted storage; keys in secure enclave; ensure no swap of plaintext. |

## Assumptions

- OS keystore (KeyStore/Keychain/TPM) provides reasonable protection.
- Users verify safety numbers on first contact.
- The relay is honest-but-curious as default threat model.
- TLS termination at reverse proxy (or native TLS in client later).

## In-scope

- E2EE confidentiality/integrity/authenticity.
- Forward secrecy and post-compromise security (via ratchet/MLS).
- Minimal metadata collection.
- Secure local storage.

## Out-of-scope

- Side-channels (power, acoustic) — general hardening later.
- Nation-state coercion of cloud push providers — documented trade-off.
- Denial of service — mitigation via rate-limits, planned.
- Social engineering — UX cannot fix everything.

## Security requirements

- No plaintext log of messages/keys.
- Never persist keys unencrypted outside keystore.
- AEAD for all stored/cached ciphertext.
- Public keys bound to identities, verified out-of-band.
- Reproducible builds + open source (AGPL).
- External audit before real user adoption.
