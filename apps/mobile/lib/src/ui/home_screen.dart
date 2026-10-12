import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../app_state.dart';
import '../models/models.dart';
import '../theme.dart';
import 'add_contact_screen.dart';
import 'chat_screen.dart';
import 'format.dart';
import 'my_qr_screen.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watch<AppState>();
    final contacts = [...state.contacts]
      ..sort((a, b) {
        final at = state.lastMessage(a.routingToken)?.timestampMs ?? 0;
        final bt = state.lastMessage(b.routingToken)?.timestampMs ?? 0;
        if (at == bt) return a.username.toLowerCase().compareTo(b.username.toLowerCase());
        return bt.compareTo(at);
      });

    return Scaffold(
      appBar: AppBar(
        title: const Text('Aegis'),
        actions: [
          _ConnectionDot(connected: state.relayConnected, onTap: () => _showConnection(context)),
          IconButton(
            tooltip: 'My QR code',
            icon: const Icon(Icons.qr_code_2),
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => const MyQrScreen()),
            ),
          ),
          const SizedBox(width: 4),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => Navigator.of(context).push(
          MaterialPageRoute(builder: (_) => const AddContactScreen()),
        ),
        icon: const Icon(Icons.person_add_alt_1),
        label: const Text('Add contact'),
      ),
      body: !state.relayConnected
          ? Column(
              children: [
                _OfflineBanner(onRetry: state.reconnect),
                Expanded(child: _body(context, state, contacts)),
              ],
            )
          : _body(context, state, contacts),
    );
  }

  Widget _body(BuildContext context, AppState state, List<Contact> contacts) {
    if (contacts.isEmpty) {
      return const _EmptyState();
    }
    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: contacts.length,
      separatorBuilder: (_, __) => const Divider(height: 1, indent: 76),
      itemBuilder: (context, i) {
        final c = contacts[i];
        final last = state.lastMessage(c.routingToken);
        return ListTile(
          leading: CircleAvatar(
            radius: 26,
            backgroundColor: kAccent.withValues(alpha: 0.2),
            child: Text(
              initials(c.username),
              style: const TextStyle(color: kAccent, fontWeight: FontWeight.w600),
            ),
          ),
          title: Text(c.username, style: const TextStyle(fontWeight: FontWeight.w600)),
          subtitle: Text(
            last == null ? 'Tap to start chatting' : (last.fromMe ? 'You: ${last.body}' : last.body),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(color: Colors.white.withValues(alpha: 0.6)),
          ),
          trailing: last == null
              ? const Icon(Icons.lock_outline, size: 16, color: Colors.white38)
              : Text(
                  shortTime(last.timestampMs),
                  style: const TextStyle(fontSize: 12, color: Colors.white38),
                ),
          onTap: () => Navigator.of(context).push(
            MaterialPageRoute(builder: (_) => ChatScreen(contact: c)),
          ),
        );
      },
    );
  }

  void _showConnection(BuildContext context) {
    final state = context.read<AppState>();
    showModalBottomSheet(
      context: context,
      showDragHandle: true,
      builder: (_) => Padding(
        padding: const EdgeInsets.fromLTRB(20, 0, 20, 32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Connection', style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
            const SizedBox(height: 12),
            _kv('Status', state.relayConnected ? 'Connected' : 'Disconnected'),
            _kv('Relay', state.relayUrl),
            _kv('Username', state.identity?.username ?? '—'),
            _kv('Routing token', state.identity?.routingToken ?? '—'),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: FilledButton.tonal(
                onPressed: () {
                  state.reconnect();
                  Navigator.of(context).pop();
                },
                child: const Text('Reconnect'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _kv(String key, String value) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(width: 110, child: Text(key, style: const TextStyle(color: Colors.white54))),
            Expanded(child: SelectableText(value)),
          ],
        ),
      );
}

class _ConnectionDot extends StatelessWidget {
  const _ConnectionDot({required this.connected, required this.onTap});
  final bool connected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      customBorder: const CircleBorder(),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Container(
          width: 12,
          height: 12,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: connected ? const Color(0xFF31C48D) : const Color(0xFFF05252),
            boxShadow: [
              BoxShadow(
                color: (connected ? const Color(0xFF31C48D) : const Color(0xFFF05252))
                    .withValues(alpha: 0.5),
                blurRadius: 8,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _OfflineBanner extends StatelessWidget {
  const _OfflineBanner({required this.onRetry});
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: const Color(0xFF3A2A12),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        child: Row(
          children: [
            const Icon(Icons.cloud_off, size: 18, color: Color(0xFFFFB020)),
            const SizedBox(width: 10),
            const Expanded(child: Text('Relay offline — messages will queue.')),
            TextButton(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  const _EmptyState();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.forum_outlined, size: 72, color: Colors.white.withValues(alpha: 0.2)),
            const SizedBox(height: 20),
            const Text('No conversations yet',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            Text(
              'Share your QR code or paste a friend’s invite to start an\nencrypted conversation.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.white.withValues(alpha: 0.6), height: 1.4),
            ),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const MyQrScreen()),
              ),
              icon: const Icon(Icons.qr_code_2),
              label: const Text('Show my QR code'),
            ),
          ],
        ),
      ),
    );
  }
}
