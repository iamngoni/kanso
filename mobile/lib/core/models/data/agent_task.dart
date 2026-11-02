// Kanso — agent_task.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'agent_task.freezed.dart';

/// Where an agent task is.
enum AgentTaskPhase {
  waiting,
  running,
  asking,
  ready,
  nothing,
  failed,
  applied,
  undone,
  discarded;

  /// The phase the core calls [word]; an unknown word is [failed].
  static AgentTaskPhase of(String word) {
    for (final phase in values) {
      if (phase.name == word) {
        return phase;
      }
    }
    return failed;
  }
}

/// One agent task: a goal the AI works on, then changes to review.
@freezed
abstract class AgentTask with _$AgentTask {
  const factory AgentTask({
    required int id,
    required String goal,
    required AgentTaskPhase phase,

    /// The question, the error or the number of changes, by [phase].
    @Default('') String detail,

    /// The live line: "Reading Q3 planning".
    @Default('') String line,
    @Default('') String lastMessage,
    @Default(0) int notesRead,
    @Default(0) int changes,
    @Default(0) int elapsedSeconds,
  }) = _AgentTask;

  const AgentTask._();

  /// Whether the AI is still working.
  bool get active =>
      phase == AgentTaskPhase.running || phase == AgentTaskPhase.waiting;

  /// Whether there is something to review.
  bool get reviewable =>
      phase == AgentTaskPhase.ready ||
      (phase == AgentTaskPhase.failed && changes > 0);
}
