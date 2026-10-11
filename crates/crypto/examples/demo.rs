//! End-to-end demonstration of the Veil crypto primitives.
//!
//! Run with: `cargo run -p veil-crypto --example demo`

use veil_crypto::{
    decode_prekey_bundle, encode_prekey_bundle, kdf, safety_number, AeadKey, ExchangeKeyPair,
    IdentityKeyPair, PreKeyBundle,
};

fn main() {
    let mut rng = rand::thread_rng();

    // --- Two users each generate an identity + exchange key pair ---
    let alice_id = IdentityKeyPair::generate(&mut rng);
    let alice_ex = ExchangeKeyPair::generate(&mut rng);
    let bob_id = IdentityKeyPair::generate(&mut rng);
    let bob_ex = ExchangeKeyPair::generate(&mut rng);

    println!("Alice identity : {}", alice_id.public());
    println!("Bob   identity : {}", bob_id.public());
    println!();

    // --- Safety number: both sides must compute the SAME value ---
    let a_side = safety_number(&alice_id.public(), &bob_id.public());
    let b_side = safety_number(&bob_id.public(), &alice_id.public());
    println!("Alice sees safety number : {a_side}");
    println!("Bob   sees safety number : {b_side}");
    println!("Match (MITM check): {}\n", a_side == b_side);

    // --- Alice shares a QR prekey bundle; Bob scans it ---
    let bundle = PreKeyBundle {
        identity_key: alice_id.public().to_base64(),
        exchange_key: base64(&alice_ex.public().to_bytes()),
        signed_prekey: None,
        signed_prekey_signature: None,
    };
    let qr = encode_prekey_bundle(&bundle).unwrap();
    println!("Alice's QR payload ({} chars):", qr.len());
    println!("  {qr}\n");

    let scanned = decode_prekey_bundle(&qr).unwrap();
    println!("Bob scanned Alice's identity: {}\n", scanned.identity_key);

    // --- Derive a shared secret via X25519 + HKDF (both sides independently) ---
    let alice_secret = alice_ex.diffie_hellman(&bob_ex.public());
    let bob_secret = bob_ex.diffie_hellman(&alice_ex.public());
    let alice_key =
        AeadKey::from_bytes(kdf::derive_key(&alice_secret, b"salt", b"veil-demo").unwrap());
    let bob_key = AeadKey::from_bytes(kdf::derive_key(&bob_secret, b"salt", b"veil-demo").unwrap());
    println!("Shared secrets agree: {}\n", alice_secret == bob_secret);

    // --- Alice encrypts a message; Bob decrypts it ---
    let plaintext = b"hey Bob, this is end-to-end encrypted";
    let (ciphertext, nonce) = alice_key
        .encrypt_random(&mut rng, b"conversation-header", plaintext)
        .unwrap();
    println!("Ciphertext on the wire : {}", hex::encode(&ciphertext));

    let decrypted = bob_key
        .decrypt(&nonce, b"conversation-header", &ciphertext)
        .unwrap();
    println!(
        "Bob decrypted          : {}\n",
        String::from_utf8_lossy(&decrypted)
    );

    // --- Tampering is detected ---
    let mut tampered = ciphertext.clone();
    tampered[0] ^= 0x01;
    let caught = bob_key
        .decrypt(&nonce, b"conversation-header", &tampered)
        .is_err();
    println!("Tampered message rejected: {caught}");
}

fn base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}
