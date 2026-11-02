// Kanso — agent_task_prefs.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'agent_task_prefs.freezed.dart';

/// The time limits Settings offers, in minutes.
const List<int> agentTimeLimits = <int>[5, 20, 60];

/// The note limits Settings offers.
const List<int> agentMaxNotesChoices = <int>[50, 200, 1000];

/// Settings › AI › Agent tasks.
@freezed
abstract class AgentTaskPrefs with _$AgentTaskPrefs {
  const factory AgentTaskPrefs({
    @Default(true) bool enabled,
    @Default(true) bool newNotes,
    @Default(true) bool edits,
    @Default(true) bool moves,
    @Default(true) bool merges,

    /// The most minutes one task may run: 5, 20 or 60.
    @Default(20) int timeLimitMinutes,

    /// The most notes one task may read: 50, 200 or 1000.
    @Default(200) int maxNotes,

    /// Ask before a task that may read every notebook starts.
    @Default(true) bool askBeforeReadingAll,
  }) = _AgentTaskPrefs;
}
