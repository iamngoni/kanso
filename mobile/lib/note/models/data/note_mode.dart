// Kanso — note_mode.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How an open note is shown.
enum NoteMode {
  /// The note to read. Until the rendered preview comes, its highlighted
  /// source.
  preview,

  /// The note's Markdown to edit.
  edit,
}
