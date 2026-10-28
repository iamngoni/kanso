// Kanso — note_window_request.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'note_window_request.freezed.dart';

/// What a note's window asks of the main window, because only the main
/// window can do it (the desktop's `forward`). The main window is brought
/// forward and does it.
@freezed
sealed class NoteWindowRequest with _$NoteWindowRequest {
  /// A new note, in All Notes.
  const factory NoteWindowRequest.newNote() = NewNoteRequest;

  /// A new note titled [title], for a link in the text to a note that is
  /// not there yet.
  const factory NoteWindowRequest.newNoteTitled(String title) =
      NewNoteTitledRequest;

  /// Search all notes.
  const factory NoteWindowRequest.search() = SearchRequest;

  /// The command palette. It is handed over, and acts once the palette is
  /// in the main window.
  const factory NoteWindowRequest.palette() = PaletteRequest;

  /// The note [noteId] in the main window, for a link in the text.
  const factory NoteWindowRequest.openNote(String noteId) = OpenNoteRequest;
}
