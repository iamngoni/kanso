// Kanso — flush_reason.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why the note is saved now rather than when typing pauses.
enum FlushReason {
  /// The app is going to the background.
  lifecycle,

  /// The person turned to Preview.
  preview,

  /// Another note, or a link, is about to be opened.
  navigation,

  /// The person pressed the shortcut for save.
  shortcut,

  /// The person asked to try a failed save again.
  retry,
}
