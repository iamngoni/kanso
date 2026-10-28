// Kanso — dictation_failure.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why a session of dictation failed.
enum DictationFailure {
  /// The microphone or speech recognition is not allowed.
  permissionDenied,

  /// No recognizer can run now.
  notAvailable,

  /// Nothing was heard.
  noSpeech,

  /// The microphone could not be used, or another app took it.
  audio,

  /// A session was running already.
  busy,

  /// The system interrupted: a call, an alarm, Siri.
  interrupted,

  /// Anything else.
  unknown,
}
