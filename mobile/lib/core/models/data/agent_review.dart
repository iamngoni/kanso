// Kanso — agent_review.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'agent_card.dart';

part 'agent_review.freezed.dart';

/// The changes an agent proposed, to approve.
@freezed
abstract class AgentReview with _$AgentReview {
  const factory AgentReview({
    required String header,
    @Default('') String fromAgent,
    @Default(<String>[]) List<String> warnings,
    @Default(<AgentCard>[]) List<AgentCard> cards,
  }) = _AgentReview;
}
