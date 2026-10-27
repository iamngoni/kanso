// Kanso — note_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_bloc.dart';

@freezed
abstract class NoteState with _$NoteState {
  const factory NoteState({
    /// The note being shown.
    @Default('') String noteId,

    /// The built-in theme its text is highlighted in.
    @Default('') String themeId,
    @Default(NoteScreenStatus.loading) NoteScreenStatus status,

    /// Whether the note is read or edited.
    @Default(NoteMode.preview) NoteMode mode,

    /// Whether the note was just made, so that it opened ready to write in:
    /// with the title to type in when it has none.
    @Default(false) bool openedNew,

    /// Where the caret starts when the editor opens: the offset asked for,
    /// or where it was left; null for the start of the text. In units of the
    /// text the editor shows, which has no front matter, unless
    /// [caretInBody].
    int? caret,

    /// Whether [caret] is an offset in the whole body, as a tap on the
    /// preview names it, rather than in the editor's text.
    @Default(false) bool caretInBody,

    /// How far the editor's page starts scrolled.
    @Default(0) double scroll,

    /// The note as read. While it is locked and closed it has its title and
    /// its lock and no text, tags or status; null until it is read, and
    /// when there is none.
    NoteView? note,

    /// The text of the note, cut into lines with their styled runs; empty
    /// unless [status] is ready.
    @Default(<SourceLine>[]) List<SourceLine> lines,

    /// Why reading failed, while [status] is failed.
    ApplicationError? error,
  }) = _NoteState;
}
