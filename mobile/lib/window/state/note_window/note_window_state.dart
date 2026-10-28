// Kanso — note_window_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_window_bloc.dart';

@freezed
abstract class NoteWindowState with _$NoteWindowState {
  const factory NoteWindowState({
    @Default('') String noteId,
    @Default(NoteWindowStatus.loading) NoteWindowStatus status,

    /// The note's title. Titles are not sealed, but a locked note's is kept
    /// out of the window's name (see `noteWindowTitle`).
    @Default('') String title,

    /// Whether the note is locked, open or closed.
    @Default(false) bool locked,

    /// Whether the note is in the trash.
    @Default(false) bool trashed,
  }) = _NoteWindowState;
}
