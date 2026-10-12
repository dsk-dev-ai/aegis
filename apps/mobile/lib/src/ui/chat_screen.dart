import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../app_state.dart';
import '../models/models.dart';
import '../theme.dart';
import 'format.dart';

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key, required this.contact});

  final Contact contact;

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  final _input = TextEditingController();
  final _scroll = ScrollController();

  @override
  void dispose() {
    _input.dispose();
    _scroll.dispose();
    super.dispose();
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scroll.hasClients) {
        _scroll.animateTo(
          _scroll.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _send() async {
    final text = _input.text;
    if (text.trim().isEmpty) return;
    _input.clear();
    await context.read<AppState>().sendMessage(widget.contact, text);
    _scrollToBottom();
  }

  void _showSafetyNumber() {
    final number = widget.contact.safetyNumber ?? 'Not available';
    showDialog<void>(
      context: context,
      builder: (_) => AlertDialog(
        title: const Text('Verify safety number'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Compare this with ${widget.contact.username}. If it matches, '
              'your conversation is secure.',
              style: TextStyle(color: Colors.white.withValues(alpha: 0.7), height: 1.4),
            ),
            const SizedBox(height: 16),
            SelectableText(
              number,
              style: const TextStyle(
                fontFamily: 'monospace',
                fontSize: 16,
                height: 1.6,
                letterSpacing: 1.2,
              ),
            ),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Close')),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = context.watch<AppState>();
    final messages = state.messagesFor(widget.contact.routingToken);
    _scrollToBottom();

    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: InkWell(
          onTap: _showSafetyNumber,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(widget.contact.username),
              Row(
                children: [
                  const Icon(Icons.lock, size: 12, color: Color(0xFF31C48D)),
                  const SizedBox(width: 4),
                  Text(
                    'End-to-end encrypted',
                    style: TextStyle(
                      fontSize: 12,
                      color: Colors.white.withValues(alpha: 0.6),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
      body: Column(
        children: [
          Expanded(
            child: messages.isEmpty
                ? const _EmptyChat()
                : ListView.builder(
                    controller: _scroll,
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                    itemCount: messages.length,
                    itemBuilder: (context, i) => _Bubble(message: messages[i]),
                  ),
          ),
          _Composer(controller: _input, onSend: _send),
        ],
      ),
    );
  }
}

class _Bubble extends StatelessWidget {
  const _Bubble({required this.message});
  final Message message;

  @override
  Widget build(BuildContext context) {
    final me = message.fromMe;
    return Align(
      alignment: me ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 3),
        padding: const EdgeInsets.fromLTRB(14, 10, 14, 8),
        constraints: BoxConstraints(
          maxWidth: MediaQuery.of(context).size.width * 0.75,
        ),
        decoration: BoxDecoration(
          color: me ? kSentBubble : kReceivedBubble,
          borderRadius: BorderRadius.only(
            topLeft: const Radius.circular(16),
            topRight: const Radius.circular(16),
            bottomLeft: Radius.circular(me ? 16 : 4),
            bottomRight: Radius.circular(me ? 4 : 16),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Text(message.body, style: const TextStyle(fontSize: 15.5, height: 1.3)),
            const SizedBox(height: 4),
            Text(
              shortTime(message.timestampMs),
              style: TextStyle(fontSize: 11, color: Colors.white.withValues(alpha: 0.5)),
            ),
          ],
        ),
      ),
    );
  }
}

class _Composer extends StatelessWidget {
  const _Composer({required this.controller, required this.onSend});
  final TextEditingController controller;
  final Future<void> Function() onSend;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 8, 12, 12),
        child: Row(
          children: [
            Expanded(
              child: TextField(
                controller: controller,
                textInputAction: TextInputAction.send,
                minLines: 1,
                maxLines: 5,
                onSubmitted: (_) => onSend(),
                decoration: const InputDecoration(
                  hintText: 'Message',
                  contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                ),
              ),
            ),
            const SizedBox(width: 8),
            Material(
              color: kAccent,
              shape: const CircleBorder(),
              child: InkWell(
                customBorder: const CircleBorder(),
                onTap: onSend,
                child: const Padding(
                  padding: EdgeInsets.all(14),
                  child: Icon(Icons.send_rounded, size: 22, color: Colors.white),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _EmptyChat extends StatelessWidget {
  const _EmptyChat();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.lock_outline, size: 56, color: Colors.white.withValues(alpha: 0.2)),
            const SizedBox(height: 16),
            Text(
              'Messages you send are end-to-end encrypted.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.white.withValues(alpha: 0.6), height: 1.4),
            ),
          ],
        ),
      ),
    );
  }
}
