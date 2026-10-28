// Kanso — dictation_unavailable_reason.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why speech cannot be recognized on this phone, in this language.
enum DictationUnavailableReason {
  /// The phone has no speech recognizer.
  noRecognizer,

  /// The language has no recognizer that works on the phone itself. Kanso
  /// never sends a voice to a server on iOS.
  notOnDevice,

  /// The recognizer does not know the language.
  unsupportedLanguage,
}
