// Kanso — dictation_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where dictation is.
enum DictationStatus {
  /// Ready, or finished: not listening.
  idle,

  /// Asking for permission, or starting the microphone.
  requesting,

  /// The microphone is open.
  listening,

  /// Speech cannot be recognized on this phone; see the reason.
  unavailable,

  /// The microphone or speech recognition is not allowed.
  denied,

  /// The session failed; see the failure.
  error,
}
