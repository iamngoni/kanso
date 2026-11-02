// Kanso — agent_tasks_state.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'agent_tasks_bloc.dart';

@freezed
abstract class AgentTasksState with _$AgentTasksState {
  const factory AgentTasksState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(AgentTaskPrefs()) AgentTaskPrefs prefs,
    @Default(<AgentTask>[]) List<AgentTask> tasks,

    /// A start is on its way.
    @Default(false) bool starting,

    /// Counts the tasks started from this page, so it can clear its field.
    @Default(0) int started,

    /// Why the last start didn't happen, as the core words it.
    String? trouble,
  }) = _AgentTasksState;
}
