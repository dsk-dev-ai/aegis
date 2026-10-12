import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../app_state.dart';
import 'chat_screen.dart';

class AddContactScreen extends StatefulWidget {
  const AddContactScreen({super.key});

  @override
  State<AddContactScreen> createState() => _AddContactScreenState();
}

class _AddContactScreenState extends State<AddContactScreen> {
  final _input = TextEditingController();
  bool _busy = false;
  String? _error;

  @override
  void dispose() {
    _input.dispose();
    super.dispose();
  }

  Future<void> _add() async {
    final code = _input.text.trim();
    if (code.isEmpty) {
      setState(() => _error = 'Paste an invite code first.');
      return;
    }
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final contact = await context.read<AppState>().addContactFromInvite(code);
      if (!mounted) return;
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => ChatScreen(contact: contact)),
      );
    } catch (e) {
      if (mounted) {
        setState(() {
          _busy = false;
          _error = 'Could not read that code: $e';
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Add contact')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Icon(Icons.group_add_outlined, size: 56, color: Color(0xFF4C7DFF)),
            const SizedBox(height: 20),
            Text(
              'Ask your friend to show their QR code, or paste the invite code '
              'they shared with you.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.white.withValues(alpha: 0.7), height: 1.4),
            ),
            const SizedBox(height: 24),
            TextField(
              controller: _input,
              minLines: 3,
              maxLines: 6,
              autofocus: true,
              decoration: InputDecoration(
                hintText: 'aegis://add?d=…',
                labelText: 'Invite code',
                errorText: _error,
              ),
            ),
            const SizedBox(height: 20),
            FilledButton.icon(
              onPressed: _busy ? null : _add,
              icon: _busy
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.add),
              label: Text(_busy ? 'Adding…' : 'Add contact'),
            ),
            const SizedBox(height: 16),
            Text(
              'After adding, open the chat and compare safety numbers to verify '
              'your connection.',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 12,
                color: Colors.white.withValues(alpha: 0.5),
                height: 1.4,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
