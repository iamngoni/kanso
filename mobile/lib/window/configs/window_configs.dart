// Kanso — window_configs.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the Dart side and the native sides agree on for note windows.
abstract final class WindowConfigs {
  /// The channel every engine's native side answers on.
  static const String channel = 'app.kanso/windows';

  /// The library the note window's entry point is in, and its name: the
  /// native sides start an engine there.
  static const String entryLibrary = 'package:kanso/note_window_main.dart';
  static const String entryPoint = 'noteWindowMain';
}
