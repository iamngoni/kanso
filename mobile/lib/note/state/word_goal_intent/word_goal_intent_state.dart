// Kanso — word_goal_intent_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'word_goal_intent_bloc.dart';

@freezed
abstract class WordGoalIntentState with _$WordGoalIntentState {
  const factory WordGoalIntentState({
    @Default(0) int request,
    @Default('') String noteId,
  }) = _WordGoalIntentState;

  const WordGoalIntentState._();

  /// Whether there is a picker waiting for its note's editor.
  bool get pending => request > 0 && noteId.isNotEmpty;
}
