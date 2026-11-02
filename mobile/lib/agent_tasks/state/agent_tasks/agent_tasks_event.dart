// Kanso — agent_tasks_event.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'agent_tasks_bloc.dart';

@freezed
sealed class AgentTasksEvent with _$AgentTasksEvent {
  /// The page opened.
  const factory AgentTasksEvent.started() = AgentTasksStartedEvent;

  /// The timer asks how the tasks are doing.
  const factory AgentTasksEvent.polled() = AgentTasksPolledEvent;

  /// The person sent [goal]; with no [noteIds] the AI may read every
  /// unlocked note.
  const factory AgentTasksEvent.goalSent(
    String goal, {
    @Default(<String>[]) List<String> noteIds,
  }) = AgentTasksGoalSentEvent;

  const factory AgentTasksEvent.stopped(int id) = AgentTasksStoppedEvent;

  const factory AgentTasksEvent.replied(int id, String text) =
      AgentTasksRepliedEvent;

  const factory AgentTasksEvent.dismissed(int id) = AgentTasksDismissedEvent;

  const factory AgentTasksEvent.troubleCleared() =
      AgentTasksTroubleClearedEvent;
}
