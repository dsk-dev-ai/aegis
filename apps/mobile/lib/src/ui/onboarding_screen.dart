import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../app_state.dart';
import '../theme.dart';

class OnboardingScreen extends StatefulWidget {
  const OnboardingScreen({super.key});

  @override
  State<OnboardingScreen> createState() => _OnboardingScreenState();
}

class _OnboardingScreenState extends State<OnboardingScreen> {
  final _username = TextEditingController();
  final _relay = TextEditingController();
  final _formKey = GlobalKey<FormState>();
  bool _busy = false;
  bool _advanced = false;

  @override
  void initState() {
    super.initState();
    _relay.text = context.read<AppState>().relayUrl;
  }

  @override
  void dispose() {
    _username.dispose();
    _relay.dispose();
    super.dispose();
  }

  String? _validateUsername(String? value) {
    final v = (value ?? '').trim();
    if (v.length < 3) return 'At least 3 characters.';
    if (v.length > 32) return 'At most 32 characters.';
    if (!RegExp(r'^[a-zA-Z0-9_.-]+$').hasMatch(v)) {
      return 'Letters, digits, and _ . - only.';
    }
    return null;
  }

  Future<void> _create() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _busy = true);
    final state = context.read<AppState>();
    final relay = _relay.text.trim();
    if (relay.isNotEmpty && relay != state.relayUrl) {
      await state.setRelayUrl(relay);
    }
    await state.createIdentity(_username.text.trim());
    if (mounted) setState(() => _busy = false);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(28),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 440),
              child: Form(
                key: _formKey,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Icon(Icons.shield_moon, size: 72, color: kAccent),
                    const SizedBox(height: 20),
                    const Text(
                      'Aegis',
                      textAlign: TextAlign.center,
                      style: TextStyle(fontSize: 36, fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Private, end-to-end encrypted messaging.\nNo phone number, no email.',
                      textAlign: TextAlign.center,
                      style: TextStyle(color: Colors.white.withValues(alpha: 0.7), height: 1.4),
                    ),
                    const SizedBox(height: 36),
                    TextFormField(
                      controller: _username,
                      autofocus: true,
                      textInputAction: TextInputAction.done,
                      decoration: const InputDecoration(
                        labelText: 'Choose a username',
                        prefixIcon: Icon(Icons.person_outline),
                      ),
                      validator: _validateUsername,
                      onFieldSubmitted: (_) => _create(),
                    ),
                    const SizedBox(height: 16),
                    FilledButton.icon(
                      onPressed: _busy ? null : _create,
                      icon: _busy
                          ? const SizedBox(
                              width: 18,
                              height: 18,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.lock_outline),
                      label: Text(_busy ? 'Generating keys…' : 'Create identity'),
                    ),
                    const SizedBox(height: 20),
                    TextButton(
                      onPressed: () => setState(() => _advanced = !_advanced),
                      child: Text(_advanced ? 'Hide advanced' : 'Advanced settings'),
                    ),
                    if (_advanced)
                      TextFormField(
                        controller: _relay,
                        decoration: const InputDecoration(
                          labelText: 'Relay URL',
                          prefixIcon: Icon(Icons.dns_outlined),
                          helperText: 'WebSocket endpoint of the Aegis relay',
                        ),
                      ),
                    const SizedBox(height: 24),
                    Text(
                      'Your keys are generated and stored only on this device.',
                      textAlign: TextAlign.center,
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.white.withValues(alpha: 0.5),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
