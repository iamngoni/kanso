// Kanso — word_goal_intent_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'word_goal_intent_bloc.dart';

@freezed
sealed class WordGoalIntentEvent with _$WordGoalIntentEvent {
  /// Opens the word-goal picker for the note [noteId].
  const factory WordGoalIntentEvent.requested({required String noteId}) =
      WordGoalIntentRequestedEvent;

  /// Marks [request] as handled, so returning to the note does not run it
  /// again.
  const factory WordGoalIntentEvent.consumed({required int request}) =
      WordGoalIntentConsumedEvent;
}
