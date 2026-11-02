// Kanso — agent_review_event.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'agent_review_bloc.dart';

@freezed
sealed class AgentReviewEvent with _$AgentReviewEvent {
  /// The review of task [task] opened.
  const factory AgentReviewEvent.opened(int task) = AgentReviewOpenedEvent;

  const factory AgentReviewEvent.cardToggled(int index, {required bool on}) =
      AgentReviewCardToggledEvent;

  /// "Keep the original notes" on the merge at [index].
  const factory AgentReviewEvent.keepToggled(int index, {required bool on}) =
      AgentReviewKeepToggledEvent;

  /// Apply the ticked cards.
  const factory AgentReviewEvent.applied() = AgentReviewAppliedEvent;

  /// Take back what was applied.
  const factory AgentReviewEvent.undone() = AgentReviewUndoneEvent;
}
