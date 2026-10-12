import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:cryptography/cryptography.dart';

/// Core cryptographic operations for Aegis.
///
/// The wire format (invite payload, relay envelope, safety number) mirrors the
/// Rust core in `crates/`, so the native FFI path and this web/mobile path
/// interoperate.
///
/// NOTE: this is a working, genuine X25519 + HKDF + ChaCha20-Poly1305 scheme.
/// The production roadmap upgrades 1:1 to X3DH + Double Ratchet (forward
/// secrecy) and groups to MLS; see docs/CRYPTO.md.
class AegisCrypto {
  AegisCrypto._();

  static final Ed25519 _ed = Ed25519();
  static final X25519 _x25519 = X25519();
  static final Sha256 _sha = Sha256();

  static const String _kdfInfo = 'aegis-direct-v1';
  static const String _safetyPrefix = 'aegis-safety-number-v1';
  static const String _invitePrefix = 'aegis://add?d=';

  // --- random / encoding helpers ---

  static final Random _rng = Random.secure();

  static List<int> randomBytes(int n) =>
      Uint8List.fromList(List<int>.generate(n, (_) => _rng.nextInt(256)));

  static String b64(List<int> bytes) => base64Url.encode(bytes).replaceAll('=', '');

  static List<int> b64d(String input) {
    var s = input.trim().replaceAll('-', '+').replaceAll('_', '/');
    while (s.length % 4 != 0) {
      s += '=';
    }
    return base64.decode(s);
  }

  static String hex(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static int _cmpBytes(List<int> a, List<int> b) {
    final n = a.length < b.length ? a.length : b.length;
    for (var i = 0; i < n; i++) {
      final d = a[i] - b[i];
      if (d != 0) return d;
    }
    return a.length - b.length;
  }

  static List<int> _u32be(int value) => [
        (value >> 24) & 0xff,
        (value >> 16) & 0xff,
        (value >> 8) & 0xff,
        value & 0xff,
      ];

  // --- key generation / derivation ---

  static Future<List<int>> edPublicFromSeed(List<int> seed) async {
    final kp = await _ed.newKeyPairFromSeed(seed);
    return (await kp.extractPublicKey()).bytes;
  }

  static Future<List<int>> xPublicFromSeed(List<int> seed) async {
    final kp = await _x25519.newKeyPairFromSeed(seed);
    return (await kp.extractPublicKey()).bytes;
  }

  static Future<List<int>> sign(List<int> edSeed, List<int> message) async {
    final kp = await _ed.newKeyPairFromSeed(edSeed);
    final sig = await _ed.sign(message, keyPair: kp);
    return sig.bytes;
  }

  /// Derive the shared conversation key. Both peers compute the same value:
  /// X25519(my seed, peer public) → HKDF bound to both identity keys.
  static Future<SecretKey> deriveConversationKey({
    required List<int> myDhSeed,
    required List<int> peerDhPublic,
    required List<int> myIdentityPublic,
    required List<int> peerIdentityPublic,
  }) async {
    final kp = await _x25519.newKeyPairFromSeed(myDhSeed);
    final shared = await _x25519.sharedSecretKey(
      keyPair: kp,
      remotePublicKey: SimplePublicKey(peerDhPublic, type: KeyPairType.x25519),
    );
    final secretBytes = await shared.extractBytes();

    final ids = [myIdentityPublic, peerIdentityPublic]..sort(_cmpBytes);
    final info = <int>[
      ...utf8.encode(_kdfInfo),
      ...ids[0],
      ...ids[1],
    ];

    final hkdf = Hkdf(hmac: Hmac.sha256(), outputLength: 32);
    return hkdf.deriveKey(
      secretKey: SecretKey(secretBytes),
      nonce: utf8.encode(_kdfInfo),
      info: info,
    );
  }

  /// AAD binding the ciphertext to the pair of identities.
  static List<int> conversationAad(List<int> a, List<int> b) {
    final ids = [a, b]..sort(_cmpBytes);
    return <int>[...utf8.encode('aegis-aad-v1'), ...ids[0], ...ids[1]];
  }

  // --- AEAD ---

  static Future<String> encryptJson(
    SecretKey key,
    List<int> aad,
    Map<String, dynamic> payload,
  ) async {
    final nonce = randomBytes(12);
    final clear = utf8.encode(jsonEncode(payload));
    final box = await Chacha20.poly1305Aead()
        .encrypt(clear, secretKey: key, nonce: nonce, aad: aad);
    final blob = <int>[...nonce, ...box.cipherText, ...box.mac.bytes];
    return b64(blob);
  }

  static Future<Map<String, dynamic>> decryptJson(
    SecretKey key,
    List<int> aad,
    String blobB64,
  ) async {
    final blob = b64d(blobB64);
    if (blob.length < 28) {
      throw const FormatException('ciphertext too short');
    }
    final nonce = blob.sublist(0, 12);
    final mac = blob.sublist(blob.length - 16);
    final ct = blob.sublist(12, blob.length - 16);
    final clear = await Chacha20.poly1305Aead().decrypt(
      SecretBox(ct, nonce: nonce, mac: Mac(mac)),
      secretKey: key,
      aad: aad,
    );
    return jsonDecode(utf8.decode(clear)) as Map<String, dynamic>;
  }

  // --- safety number ---

  /// 60-digit, 12-group fingerprint both peers compute identically.
  static Future<String> safetyNumber(List<int> a, List<int> b) async {
    final ids = [a, b]..sort(_cmpBytes);
    final seed = <int>[
      ...utf8.encode(_safetyPrefix),
      ...ids[0],
      ...ids[1],
    ];
    final digits = StringBuffer();
    var counter = 0;
    while (digits.length < 60) {
      final digest = await _sha.hash(<int>[...seed, ..._u32be(counter)]);
      for (final byte in digest.bytes) {
        if (digits.length >= 60) break;
        digits.write(byte % 10);
      }
      counter++;
    }
    final s = digits.toString();
    return [
      for (var i = 0; i < 60; i += 5) s.substring(i, i + 5),
    ].join(' ');
  }

  // --- invite / QR payload ---

  static String encodeInvite({
    required String username,
    required List<int> identityKey,
    required List<int> exchangeKey,
    required String routingToken,
  }) {
    final json = {
      'username': username,
      'identity_key': b64(identityKey),
      'exchange_key': b64(exchangeKey),
      'routing_token': routingToken,
    };
    final data = base64Url.encode(utf8.encode(jsonEncode(json))).replaceAll('=', '');
    return '$_invitePrefix$data';
  }

  static Map<String, dynamic> decodeInvite(String raw) {
    final text = raw.trim();
    final marker = text.indexOf(_invitePrefix);
    final data = marker >= 0 ? text.substring(marker + _invitePrefix.length) : text;
    // Tolerate surrounding whitespace/newlines from copy-paste.
    final cleaned = data.split(RegExp(r'\s')).first;
    final decoded = jsonDecode(utf8.decode(b64d(cleaned))) as Map<String, dynamic>;
    for (final key in ['username', 'identity_key', 'exchange_key', 'routing_token']) {
      if (!decoded.containsKey(key)) {
        throw FormatException('invite missing "$key"');
      }
    }
    return decoded;
  }
}
