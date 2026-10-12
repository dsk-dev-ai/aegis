import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../app_state.dart';
import '../crypto/aegis_crypto.dart';
import '../theme.dart';

class MyQrScreen extends StatelessWidget {
  const MyQrScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final identity = context.watch<AppState>().identity;
    if (identity == null) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }

    final code = AegisCrypto.encodeInvite(
      username: identity.username,
      identityKey: identity.edPublic,
      exchangeKey: identity.xPublic,
      routingToken: identity.routingToken,
    );

    return Scaffold(
      appBar: AppBar(title: const Text('My QR code')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            Text(
              'Let a friend scan this to add you.',
              style: TextStyle(color: Colors.white.withValues(alpha: 0.7)),
            ),
            const SizedBox(height: 24),
            Container(
              padding: const EdgeInsets.all(20),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(20),
              ),
              child: QrImageView(
                data: code,
                version: QrVersions.auto,
                size: 240,
                backgroundColor: Colors.white,
                eyeStyle: const QrEyeStyle(
                  eyeShape: QrEyeShape.square,
                  color: Color(0xFF0A0E14),
                ),
                dataModuleStyle: const QrDataModuleStyle(
                  dataModuleShape: QrDataModuleShape.square,
                  color: Color(0xFF0A0E14),
                ),
              ),
            ),
            const SizedBox(height: 20),
            Text(
              identity.username,
              style: const TextStyle(fontSize: 20, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 20),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.link, size: 18, color: kAccent),
                        const SizedBox(width: 8),
                        const Text('Invite code',
                            style: TextStyle(fontWeight: FontWeight.w600)),
                        const Spacer(),
                        IconButton(
                          tooltip: 'Copy',
                          icon: const Icon(Icons.copy, size: 18),
                          onPressed: () async {
                            await Clipboard.setData(ClipboardData(text: code));
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                const SnackBar(content: Text('Invite code copied')),
                              );
                            }
                          },
                        ),
                      ],
                    ),
                    SelectableText(
                      code,
                      style: const TextStyle(fontFamily: 'monospace', fontSize: 12, height: 1.4),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Text(
              'This code contains your public keys only. Your private keys never leave this device.',
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
