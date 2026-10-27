// Kanso — editor_destination.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'editor_destination.freezed.dart';

/// Where the person asked to go from the editor, once what was typed is
/// saved.
@freezed
sealed class EditorDestination with _$EditorDestination {
  /// To the note [noteId].
  const factory EditorDestination.note(String noteId) = NoteDestination;

  /// To a note titled [title] that is to be made.
  const factory EditorDestination.newNote(String title) = NewNoteDestination;
}
