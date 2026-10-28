// Kanso — note_creation_failure.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why a new note was not made.
enum NoteCreationFailure {
  /// It would have gone into a notebook that is locked and closed.
  locked,

  /// Anything else.
  failed,
}
