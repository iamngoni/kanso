// Kanso — note_history_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_history_bloc.dart';

@freezed
abstract class NoteHistoryState with _$NoteHistoryState {
  const factory NoteHistoryState({
    /// The note whose history this is.
    @Default('') String noteId,
    @Default(HistoryStatus.loading) HistoryStatus status,

    /// The versions, newest first, beside the note as it is now.
    NoteHistory? history,

    /// The note's revision when the history was read: what a restore is
    /// checked against.
    String? currentRev,

    /// The version being looked at; the newest at first.
    String? selectedRev,
    @Default(HistoryPane.changes) HistoryPane pane,

    /// What changed from the selected version to the note now.
    @Default(<DiffHunk>[]) List<DiffHunk> hunks,
    @Default(LoadStatus.loading) LoadStatus diffStatus,

    /// The section copied last, for its button to say so.
    int? copiedSection,

    /// The selected version is being restored.
    @Default(false) bool restoring,

    /// The version was restored: the screen is done.
    @Default(false) bool restored,

    /// Why the last restore, or the last look at a version, did not happen.
    HistoryProblem? problem,
  }) = _NoteHistoryState;
}
