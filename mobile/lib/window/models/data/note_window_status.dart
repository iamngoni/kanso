// Kanso — note_window_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a note's window has to show of its note.
enum NoteWindowStatus {
  /// The note is being read.
  loading,

  /// The note is there.
  ready,

  /// The note was deleted for good: the window closes.
  gone,
}
