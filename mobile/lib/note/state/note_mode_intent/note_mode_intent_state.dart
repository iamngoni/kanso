// Kanso — note_mode_intent_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_mode_intent_bloc.dart';

@freezed
abstract class NoteModeIntentState with _$NoteModeIntentState {
  const factory NoteModeIntentState({
    @Default('') String noteId,
    NoteMode? mode,
  }) = _NoteModeIntentState;
}
