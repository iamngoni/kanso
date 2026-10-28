// Kanso — shared_note_live_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'shared_note_live_bloc.dart';

@freezed
abstract class SharedNoteLiveState with _$SharedNoteLiveState {
  const factory SharedNoteLiveState({
    /// The note whose room this is.
    @Default('') String noteId,

    /// The room is open.
    @Default(false) bool live,
  }) = _SharedNoteLiveState;
}
