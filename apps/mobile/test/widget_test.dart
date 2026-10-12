import 'package:aegis/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  testWidgets('shows onboarding when no identity exists', (tester) async {
    SharedPreferences.setMockInitialValues({});

    await tester.pumpWidget(const AegisApp());
    await tester.pumpAndSettle();

    expect(find.text('Aegis'), findsOneWidget);
    expect(find.text('Create identity'), findsOneWidget);
    expect(find.text('Choose a username'), findsOneWidget);
  });

  testWidgets('rejects an invalid username', (tester) async {
    SharedPreferences.setMockInitialValues({});

    await tester.pumpWidget(const AegisApp());
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextFormField).first, 'a');
    await tester.tap(find.text('Create identity'));
    await tester.pumpAndSettle();

    expect(find.text('At least 3 characters.'), findsOneWidget);
  });
}
