// Kanso — agent_task_prefs.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'agent_task_prefs.freezed.dart';

/// Settings › AI › Agent tasks.
@freezed
abstract class AgentTaskPrefs with _$AgentTaskPrefs {
  const factory AgentTaskPrefs({
    @Default(true) bool enabled,
    @Default(true) bool newNotes,
    @Default(true) bool edits,
    @Default(true) bool moves,
    @Default(true) bool merges,
  }) = _AgentTaskPrefs;
}
