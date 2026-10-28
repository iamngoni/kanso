// Kanso — editor_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the editor of a note has to show.
enum EditorStatus {
  /// The note is being opened for editing.
  opening,

  /// It is open: its text is in the editor.
  ready,

  /// It is locked and its lock has closed (or was closed): nothing of its
  /// text is kept.
  locked,

  /// There is no such note (any more).
  notFound,

  /// Opening it failed.
  failed,
}
