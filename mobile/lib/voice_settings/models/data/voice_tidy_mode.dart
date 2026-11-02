// Kanso — voice_tidy_mode.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether a voice note is tidied by the person's own AI.
enum VoiceTidyMode {
  /// Never.
  off,

  /// A Tidy button is offered each time (the default).
  ask,

  /// Every voice capture is tidied as soon as the app can.
  always,
}
