// Kanso — agent_review_state.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'agent_review_bloc.dart';

@freezed
abstract class AgentReviewState with _$AgentReviewState {
  const factory AgentReviewState({
    @Default(0) int task,
    @Default(LoadStatus.loading) LoadStatus status,
    AgentReview? review,
    @Default(<int>{}) Set<int> ticked,
    @Default(<int>{}) Set<int> keep,
    @Default(false) bool applying,

    /// The sentence to show once applied ("Applied 3 changes…"); Undo is
    /// offered while it is set.
    String? applied,

    /// The sentence to show once undone.
    String? undone,
    String? trouble,
  }) = _AgentReviewState;
}
