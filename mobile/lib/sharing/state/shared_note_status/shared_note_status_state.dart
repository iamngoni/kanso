// Kanso — shared_note_status_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'shared_note_status_bloc.dart';

@freezed
abstract class SharedNoteStatusState with _$SharedNoteStatusState {
  const factory SharedNoteStatusState({
    /// The note this is the status of.
    @Default('') String noteId,

    /// Its status; null while it is read, and for a note that isn't
    /// shared.
    LiveStatus? status,

    /// Why the status could not be read, or the problems not dismissed.
    ApplicationError? error,

    /// Bumped each time the status is read again: someone's cursor may have
    /// moved, and the editor asks where the cursors are now.
    @Default(0) int beat,
  }) = _SharedNoteStatusState;

  const SharedNoteStatusState._();

  /// Whether the note's text is for reading only here: shared to view or
  /// comment on, and still shared.
  bool get readOnly {
    final status = this.status;
    return status != null &&
        !status.ended &&
        (status.role == ShareRole.viewer || status.role == ShareRole.commenter);
  }
}
