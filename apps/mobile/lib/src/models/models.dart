import 'dart:convert';

import '../crypto/aegis_crypto.dart';

/// The local device identity: Ed25519 (signing) + X25519 (key agreement)
/// key pairs, a username, and a random routing token.
class Identity {
  Identity({
    required this.username,
    required this.routingToken,
    required this.edSeed,
    required this.xSeed,
    required this.edPublic,
    required this.xPublic,
  });

  final String username;
  final String routingToken;
  final List<int> edSeed;
  final List<int> xSeed;
  final List<int> edPublic;
  final List<int> xPublic;

  static Future<Identity> generate(String username) async {
    final edSeed = AegisCrypto.randomBytes(32);
    final xSeed = AegisCrypto.randomBytes(32);
    return Identity(
      username: username,
      routingToken: AegisCrypto.hex(AegisCrypto.randomBytes(16)),
      edSeed: edSeed,
      xSeed: xSeed,
      edPublic: await AegisCrypto.edPublicFromSeed(edSeed),
      xPublic: await AegisCrypto.xPublicFromSeed(xSeed),
    );
  }

  Map<String, dynamic> toJson() => {
        'username': username,
        'routing_token': routingToken,
        'ed_seed': AegisCrypto.b64(edSeed),
        'x_seed': AegisCrypto.b64(xSeed),
        'ed_public': AegisCrypto.b64(edPublic),
        'x_public': AegisCrypto.b64(xPublic),
      };

  factory Identity.fromJson(Map<String, dynamic> j) => Identity(
        username: j['username'] as String,
        routingToken: j['routing_token'] as String,
        edSeed: AegisCrypto.b64d(j['ed_seed'] as String),
        xSeed: AegisCrypto.b64d(j['x_seed'] as String),
        edPublic: AegisCrypto.b64d(j['ed_public'] as String),
        xPublic: AegisCrypto.b64d(j['x_public'] as String),
      );

  /// Copy with a new username (identity keys stay the same).
  Identity withUsername(String newUsername) => Identity(
        username: newUsername,
        routingToken: routingToken,
        edSeed: edSeed,
        xSeed: xSeed,
        edPublic: edPublic,
        xPublic: xPublic,
      );
}

/// A peer we can talk to (learned via QR / invite code).
class Contact {
  Contact({
    required this.username,
    required this.routingToken,
    required this.identityKey,
    required this.exchangeKey,
    this.safetyNumber,
  });

  final String username;
  final String routingToken;
  final List<int> identityKey;
  final List<int> exchangeKey;
  String? safetyNumber;

  Map<String, dynamic> toJson() => {
        'username': username,
        'routing_token': routingToken,
        'identity_key': AegisCrypto.b64(identityKey),
        'exchange_key': AegisCrypto.b64(exchangeKey),
        'safety_number': safetyNumber,
      };

  factory Contact.fromJson(Map<String, dynamic> j) => Contact(
        username: j['username'] as String,
        routingToken: j['routing_token'] as String,
        identityKey: AegisCrypto.b64d(j['identity_key'] as String),
        exchangeKey: AegisCrypto.b64d(j['exchange_key'] as String),
        safetyNumber: j['safety_number'] as String?,
      );

  factory Contact.fromInvite(Map<String, dynamic> invite) => Contact(
        username: invite['username'] as String,
        routingToken: invite['routing_token'] as String,
        identityKey: AegisCrypto.b64d(invite['identity_key'] as String),
        exchangeKey: AegisCrypto.b64d(invite['exchange_key'] as String),
      );
}

/// A message stored locally for display.
class Message {
  Message({
    required this.id,
    required this.conversationId,
    required this.fromMe,
    required this.body,
    required this.timestampMs,
  });

  final String id;
  final String conversationId; // == peer routing token
  final bool fromMe;
  final String body;
  final int timestampMs;

  Map<String, dynamic> toJson() => {
        'id': id,
        'conversation_id': conversationId,
        'from_me': fromMe,
        'body': body,
        'ts': timestampMs,
      };

  factory Message.fromJson(Map<String, dynamic> j) => Message(
        id: j['id'] as String,
        conversationId: j['conversation_id'] as String,
        fromMe: j['from_me'] as bool,
        body: j['body'] as String,
        timestampMs: j['ts'] as int,
      );
}

/// Serialize/deserialize a list of messages for persistence.
String encodeMessages(List<Message> messages) =>
    jsonEncode(messages.map((m) => m.toJson()).toList());

List<Message> decodeMessages(String raw) =>
    (jsonDecode(raw) as List).map((e) => Message.fromJson(e as Map<String, dynamic>)).toList();
