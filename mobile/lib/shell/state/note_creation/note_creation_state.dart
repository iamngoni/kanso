// Kanso — note_creation_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_creation_bloc.dart';

@freezed
abstract class NoteCreationState with _$NoteCreationState {
  const factory NoteCreationState({
    /// Whether a note is being made.
    @Default(false) bool creating,

    /// The note just made, for the screen to go to; it says so with
    /// `openingHandled` when it has.
    NoteOpening? opening,

    /// Why the last note was not made, until it is dismissed.
    NoteCreationFailure? failure,

    /// What was left out of the note just captured, a sentence each, until
    /// it is dismissed.
    @Default(<String>[]) List<String> problems,
  }) = _NoteCreationState;
}
