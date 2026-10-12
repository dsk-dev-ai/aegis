import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

import 'crypto/aegis_crypto.dart';
import 'models/models.dart';
import 'net/relay_client.dart';

/// Application state: identity, contacts, messages, and relay connection.
/// Persisted to [SharedPreferences].
class AppState extends ChangeNotifier {
  /// Android emulators reach the host machine at 10.0.2.2 instead of 127.0.0.1.
  static String get defaultRelay {
    if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
      return 'ws://10.0.2.2:8080/v1/ws';
    }
    return 'ws://127.0.0.1:8080/v1/ws';
  }

  static const String _kIdentity = 'aegis.identity';
  static const String _kContacts = 'aegis.contacts';
  static const String _kMessages = 'aegis.messages';
  static const String _kRelay = 'aegis.relay_url';

  final _uuid = const Uuid();

  Identity? identity;
  final List<Contact> contacts = [];
  final Map<String, List<Message>> messages = {};

  String relayUrl = defaultRelay;
  bool relayConnected = false;
  bool loading = true;

  RelayClient? _relay;
  SharedPreferences? _prefs;

  bool get hasIdentity => identity != null;

  Future<void> init() async {
    _prefs = await SharedPreferences.getInstance();
    relayUrl = _prefs!.getString(_kRelay) ?? defaultRelay;

    final rawIdentity = _prefs!.getString(_kIdentity);
    if (rawIdentity != null) {
      identity = Identity.fromJson(jsonDecode(rawIdentity) as Map<String, dynamic>);

      final rawContacts = _prefs!.getString(_kContacts);
      if (rawContacts != null) {
        for (final e in jsonDecode(rawContacts) as List) {
          contacts.add(Contact.fromJson(e as Map<String, dynamic>));
        }
      }

      final rawMessages = _prefs!.getString(_kMessages);
      if (rawMessages != null) {
        (jsonDecode(rawMessages) as Map<String, dynamic>).forEach((k, v) {
          messages[k] = (v as List)
              .map((e) => Message.fromJson(e as Map<String, dynamic>))
              .toList();
        });
      }

      _connectRelay();
    }

    loading = false;
    notifyListeners();
  }

  Future<void> createIdentity(String username) async {
    identity = await Identity.generate(username);
    await _prefs!.setString(_kIdentity, jsonEncode(identity!.toJson()));
    _connectRelay();
    notifyListeners();
  }

  Future<void> setRelayUrl(String url) async {
    relayUrl = url.trim();
    await _prefs!.setString(_kRelay, relayUrl);
    _connectRelay();
    notifyListeners();
  }

  void reconnect() => _connectRelay();

  void _connectRelay() {
    final id = identity;
    if (id == null) return;
    _relay?.close();
    _relay = RelayClient(
      url: relayUrl,
      onEnvelope: _onEnvelope,
      onStatus: (connected) {
        relayConnected = connected;
        notifyListeners();
      },
    );
    _relay!.connect(id.routingToken);
  }

  Contact? contactByToken(String token) {
    for (final c in contacts) {
      if (c.routingToken == token) return c;
    }
    return null;
  }

  /// Add (or refresh) a contact from a scanned/pasted `aegis://add?...` code.
  Future<Contact> addContactFromInvite(String code) async {
    final id = identity!;
    final invite = AegisCrypto.decodeInvite(code);
    final contact = Contact.fromInvite(invite);

    if (contact.routingToken == id.routingToken) {
      throw const FormatException('That is your own invite code.');
    }

    contact.safetyNumber =
        await AegisCrypto.safetyNumber(id.edPublic, contact.identityKey);

    final index = contacts.indexWhere((c) => c.routingToken == contact.routingToken);
    if (index >= 0) {
      contacts[index] = contact;
    } else {
      contacts.add(contact);
    }
    await _saveContacts();
    notifyListeners();
    return contact;
  }

  Future<void> renameContact(Contact contact, String username) async {
    final index = contacts.indexWhere((c) => c.routingToken == contact.routingToken);
    if (index < 0) return;
    contacts[index] = Contact(
      username: username,
      routingToken: contact.routingToken,
      identityKey: contact.identityKey,
      exchangeKey: contact.exchangeKey,
      safetyNumber: contact.safetyNumber,
    );
    await _saveContacts();
    notifyListeners();
  }

  List<Message> messagesFor(String token) => List.unmodifiable(messages[token] ?? const []);

  Message? lastMessage(String token) {
    final list = messages[token];
    return (list == null || list.isEmpty) ? null : list.last;
  }

  Future<void> sendMessage(Contact contact, String text) async {
    final id = identity!;
    final trimmed = text.trim();
    if (trimmed.isEmpty) return;

    final key = await AegisCrypto.deriveConversationKey(
      myDhSeed: id.xSeed,
      peerDhPublic: contact.exchangeKey,
      myIdentityPublic: id.edPublic,
      peerIdentityPublic: contact.identityKey,
    );
    final aad = AegisCrypto.conversationAad(id.edPublic, contact.identityKey);

    final messageId = _uuid.v4();
    final ts = DateTime.now().millisecondsSinceEpoch;
    final payload = {
      'type': 'text',
      'id': messageId,
      'body': trimmed,
      'ts': ts,
      'sender': id.username,
    };

    final ciphertext = await AegisCrypto.encryptJson(key, aad, payload);
    _relay?.send({
      'to': contact.routingToken,
      'from': id.routingToken,
      'payload': ciphertext,
      'ttl_secs': 1209600,
    });

    _append(
      contact.routingToken,
      Message(
        id: messageId,
        conversationId: contact.routingToken,
        fromMe: true,
        body: trimmed,
        timestampMs: ts,
      ),
    );
  }

  Future<void> _onEnvelope(Map<String, dynamic> envelope) async {
    final id = identity;
    if (id == null) return;

    final to = envelope['to'] as String?;
    final from = envelope['from'] as String?;
    if (to != id.routingToken || from == null) return;

    final contact = contactByToken(from);
    if (contact == null) return; // unknown sender; ignore

    final key = await AegisCrypto.deriveConversationKey(
      myDhSeed: id.xSeed,
      peerDhPublic: contact.exchangeKey,
      myIdentityPublic: id.edPublic,
      peerIdentityPublic: contact.identityKey,
    );
    final aad = AegisCrypto.conversationAad(id.edPublic, contact.identityKey);

    try {
      final payload = await AegisCrypto.decryptJson(
        key,
        aad,
        envelope['payload'] as String,
      );
      if (payload['type'] == 'text') {
        _append(
          from,
          Message(
            id: payload['id'] as String,
            conversationId: from,
            fromMe: false,
            body: payload['body'] as String,
            timestampMs: payload['ts'] as int,
          ),
        );
      }
    } catch (_) {
      // Undecryptable frame (wrong keys / tampering) — drop silently.
    }
  }

  void _append(String token, Message message) {
    messages.putIfAbsent(token, () => []).add(message);
    _saveMessages();
    notifyListeners();
  }

  Future<void> _saveContacts() =>
      _prefs!.setString(_kContacts, jsonEncode(contacts.map((c) => c.toJson()).toList()));

  Future<void> _saveMessages() => _prefs!.setString(
        _kMessages,
        jsonEncode(messages.map((k, v) => MapEntry(k, v.map((m) => m.toJson()).toList()))),
      );
}
