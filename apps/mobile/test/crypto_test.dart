import 'package:aegis/src/crypto/aegis_crypto.dart';
import 'package:aegis/src/models/models.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('safety number matches the Rust core vector', () async {
    // Produced by `cargo run -p aegis-crypto --example vector` from the Rust
    // implementation (crates/crypto/src/keys.rs).
    final a = AegisCrypto.b64d('iojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1w');
    final b = AegisCrypto.b64d('gTl3Dqh9F19Wo1Rmw0x+zMuNipG07jeiXfYPW4/Js5Q');

    final sn = await AegisCrypto.safetyNumber(a, b);
    expect(sn, '70101 73425 51421 44814 54795 13519 44831 53279 31992 80336 39215 66507');
    // Order-independent.
    expect(await AegisCrypto.safetyNumber(b, a), sn);
  });

  test('both peers derive the same conversation key and can exchange', () async {
    final alice = await Identity.generate('alice');
    final bob = await Identity.generate('bob');

    final aliceKey = await AegisCrypto.deriveConversationKey(
      myDhSeed: alice.xSeed,
      peerDhPublic: bob.xPublic,
      myIdentityPublic: alice.edPublic,
      peerIdentityPublic: bob.edPublic,
    );
    final bobKey = await AegisCrypto.deriveConversationKey(
      myDhSeed: bob.xSeed,
      peerDhPublic: alice.xPublic,
      myIdentityPublic: bob.edPublic,
      peerIdentityPublic: alice.edPublic,
    );

    expect(await aliceKey.extractBytes(), await bobKey.extractBytes());

    final aad = AegisCrypto.conversationAad(alice.edPublic, bob.edPublic);
    final blob = await AegisCrypto.encryptJson(aliceKey, aad, {'body': 'hi bob'});
    final decoded = await AegisCrypto.decryptJson(bobKey, aad, blob);
    expect(decoded['body'], 'hi bob');
  });

  test('tampered ciphertext is rejected', () async {
    final alice = await Identity.generate('alice');
    final bob = await Identity.generate('bob');
    final key = await AegisCrypto.deriveConversationKey(
      myDhSeed: alice.xSeed,
      peerDhPublic: bob.xPublic,
      myIdentityPublic: alice.edPublic,
      peerIdentityPublic: bob.edPublic,
    );
    final aad = AegisCrypto.conversationAad(alice.edPublic, bob.edPublic);
    final blob = await AegisCrypto.encryptJson(key, aad, {'body': 'secret'});

    final bytes = AegisCrypto.b64d(blob);
    bytes[15] ^= 0x01; // flip a ciphertext bit
    final tampered = AegisCrypto.b64(bytes);

    expect(
      () => AegisCrypto.decryptJson(key, aad, tampered),
      throwsA(isA<Exception>()),
    );
  });

  test('invite code round-trips', () async {
    final alice = await Identity.generate('alice');
    final code = AegisCrypto.encodeInvite(
      username: alice.username,
      identityKey: alice.edPublic,
      exchangeKey: alice.xPublic,
      routingToken: alice.routingToken,
    );
    expect(code.startsWith('aegis://add?d='), isTrue);

    final decoded = AegisCrypto.decodeInvite(code);
    expect(decoded['username'], 'alice');
    expect(decoded['routing_token'], alice.routingToken);
    expect(AegisCrypto.b64d(decoded['identity_key'] as String), alice.edPublic);
    expect(AegisCrypto.b64d(decoded['exchange_key'] as String), alice.xPublic);
  });

  test('identity json round-trips', () async {
    final alice = await Identity.generate('alice');
    final restored = Identity.fromJson(alice.toJson());
    expect(restored.username, alice.username);
    expect(restored.edSeed, alice.edSeed);
    expect(restored.xPublic, alice.xPublic);
    expect(restored.routingToken, alice.routingToken);
  });
}
