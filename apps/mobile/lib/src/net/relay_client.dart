import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

/// Thin WebSocket client for the Aegis relay.
///
/// Protocol: connects, sends `{"hello": "<routing_token>"}`, then exchanges
/// opaque `RelayEnvelope` JSON frames.
class RelayClient {
  RelayClient({
    required this.url,
    required this.onEnvelope,
    required this.onStatus,
  });

  final String url;
  final void Function(Map<String, dynamic> envelope) onEnvelope;
  final void Function(bool connected) onStatus;

  WebSocketChannel? _channel;
  bool _connected = false;

  bool get connected => _connected;

  void connect(String routingToken) {
    try {
      final channel = WebSocketChannel.connect(Uri.parse(url));
      _channel = channel;
      channel.sink.add(jsonEncode({'hello': routingToken}));
      _setConnected(true);
      channel.stream.listen(
        (data) {
          try {
            onEnvelope(jsonDecode(data as String) as Map<String, dynamic>);
          } catch (_) {
            // ignore malformed frame
          }
        },
        onDone: () => _setConnected(false),
        onError: (_) => _setConnected(false),
        cancelOnError: true,
      );
    } catch (_) {
      _setConnected(false);
    }
  }

  void send(Map<String, dynamic> envelope) {
    _channel?.sink.add(jsonEncode(envelope));
  }

  void close() {
    try {
      _channel?.sink.close();
    } catch (_) {}
    _channel = null;
    _setConnected(false);
  }

  void _setConnected(bool value) {
    if (_connected != value) {
      _connected = value;
      onStatus(value);
    }
  }
}
