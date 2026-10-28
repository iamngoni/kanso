// Kanso — assist_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where a suggestion is.
enum AssistStatus {
  /// Nothing has been asked for yet.
  idle,

  /// The suggestion is being written.
  running,

  /// The suggestion is complete and can be taken.
  ready,

  /// The person stopped it. A part of a suggestion is never offered.
  stopped,

  /// The note is kept out of AI; nothing was sent.
  keptOut,

  /// The suggestion could not be made.
  failed,
}
