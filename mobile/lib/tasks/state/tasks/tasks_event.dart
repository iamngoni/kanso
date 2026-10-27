// Kanso — tasks_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'tasks_bloc.dart';

@freezed
sealed class TasksEvent with _$TasksEvent {
  /// The place was opened: read the tasks and keep them read.
  const factory TasksEvent.started() = TasksStartedEvent;

  /// The person picked the notebook whose tasks to show; null for all.
  const factory TasksEvent.notebookChosen(String? notebookId) =
      TasksNotebookChosenEvent;

  /// The person turned showing done tasks on or off.
  const factory TasksEvent.showDoneChanged({required bool showDone}) =
      TasksShowDoneChangedEvent;

  /// The library wrote [ids]: read the tasks again, once the writes of a
  /// burst have stopped.
  const factory TasksEvent.changesReceived(List<String> ids) =
      TasksChangesReceivedEvent;

  /// The person asked to try again after a failure.
  const factory TasksEvent.retried() = TasksRetriedEvent;

  /// The person ticked [task], or cleared it if it was ticked.
  const factory TasksEvent.ticked(TaskItem task) = TasksTickedEvent;

  /// The person has read that a task had changed.
  const factory TasksEvent.noticeDismissed() = TasksNoticeDismissedEvent;
}
