import 'dart:async';
import 'dart:io';

import 'package:aegis/src/crypto/aegis_crypto.dart';
import 'package:aegis/src/models/models.dart';
import 'package:aegis/src/net/relay_client.dart';
import 'package:flutter_test/flutter_test.dart';

/// End-to-end test against a running Aegis relay.
///
/// Run with the relay listening, e.g.:
///   cargo run -p aegis-server &
///   AEGIS_RELAY_URL=ws://127.0.0.1:8080/v1/ws flutter test test/relay_e2e_test.dart
void main() {
  final relayUrl = Platform.environment['AEGIS_RELAY_URL'];

  test('two clients exchange an encrypted message via the relay', () async {
    final alice = await Identity.generate('alice');
    final bob = await Identity.generate('bob');

    final bobReceived = Completer<Map<String, dynamic>>();
    final bobClient = RelayClient(
      url: relayUrl!,
      onEnvelope: (env) {
        if (!bobReceived.isCompleted) bobReceived.complete(env);
      },
      onStatus: (_) {},
    )..connect(bob.routingToken);

    final aliceClient = RelayClient(
      url: relayUrl,
      onEnvelope: (_) {},
      onStatus: (_) {},
    )..connect(alice.routingToken);

    // Give both sockets a moment to register with the relay.
    await Future<void>.delayed(const Duration(milliseconds: 500));

    final aliceKey = await AegisCrypto.deriveConversationKey(
      myDhSeed: alice.xSeed,
      peerDhPublic: bob.xPublic,
      myIdentityPublic: alice.edPublic,
      peerIdentityPublic: bob.edPublic,
    );
    final aad = AegisCrypto.conversationAad(alice.edPublic, bob.edPublic);
    final ciphertext = await AegisCrypto.encryptJson(aliceKey, aad, {
      'type': 'text',
      'id': 'm1',
      'body': 'hello bob',
      'ts': DateTime.now().millisecondsSinceEpoch,
      'sender': 'alice',
    });

    aliceClient.send({
      'to': bob.routingToken,
      'from': alice.routingToken,
      'payload': ciphertext,
      'ttl_secs': 1209600,
    });

    final envelope = await bobReceived.future.timeout(const Duration(seconds: 5));
    expect(envelope['from'], alice.routingToken);

    final bobKey = await AegisCrypto.deriveConversationKey(
      myDhSeed: bob.xSeed,
      peerDhPublic: alice.xPublic,
      myIdentityPublic: bob.edPublic,
      peerIdentityPublic: alice.edPublic,
    );
    final decoded = await AegisCrypto.decryptJson(
      bobKey,
      aad,
      envelope['payload'] as String,
    );
    expect(decoded['body'], 'hello bob');

    aliceClient.close();
    bobClient.close();
  }, skip: relayUrl == null ? 'set AEGIS_RELAY_URL to run' : false);
}
