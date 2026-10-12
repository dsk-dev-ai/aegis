import 'package:flutter/material.dart';

const Color kBackground = Color(0xFF0A0E14);
const Color kSurface = Color(0xFF131A24);
const Color kSurfaceHigh = Color(0xFF1B2433);
const Color kAccent = Color(0xFF4C7DFF);
const Color kSentBubble = Color(0xFF25406B);
const Color kReceivedBubble = Color(0xFF1C2634);

ThemeData buildAegisTheme() {
  final scheme = ColorScheme.fromSeed(
    seedColor: kAccent,
    brightness: Brightness.dark,
  ).copyWith(surface: kSurface, primary: kAccent);

  return ThemeData(
    useMaterial3: true,
    colorScheme: scheme,
    scaffoldBackgroundColor: kBackground,
    fontFamily: 'Roboto',
    appBarTheme: const AppBarTheme(
      backgroundColor: kBackground,
      elevation: 0,
      centerTitle: false,
      titleTextStyle: TextStyle(
        fontSize: 20,
        fontWeight: FontWeight.w600,
        color: Colors.white,
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: kSurfaceHigh,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(14),
        borderSide: BorderSide.none,
      ),
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        padding: const EdgeInsets.symmetric(horizontal: 22, vertical: 16),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
        textStyle: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
      ),
    ),
    cardTheme: CardThemeData(
      color: kSurface,
      elevation: 0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
    ),
    snackBarTheme: const SnackBarThemeData(
      behavior: SnackBarBehavior.floating,
      backgroundColor: kSurfaceHigh,
    ),
  );
}
