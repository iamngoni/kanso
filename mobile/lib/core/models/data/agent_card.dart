// Kanso — agent_card.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'agent_card.freezed.dart';

/// What a change does.
enum AgentCardKind {
  create,
  edit,
  move,
  merge;

  static AgentCardKind of(String word) => values.firstWhere(
    (kind) => kind.name == word,
    orElse: () => edit,
  );
}

/// One change to approve.
@freezed
abstract class AgentCard with _$AgentCard {
  const factory AgentCard({
    required int index,
    required AgentCardKind kind,
    required String title,
    @Default('') String book,
    @Default(false) bool ticked,

    /// The note changed since the agent read it.
    @Default(false) bool changedSince,

    /// Not one of the notes the person chose.
    @Default(false) bool outside,
    @Default(false) bool published,

    /// `+ added`, `- removed` or plain lines.
    @Default(<String>[]) List<String> lines,
  }) = _AgentCard;
}
