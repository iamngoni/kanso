// Kanso — tasks_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'tasks_bloc.dart';

@freezed
abstract class TasksState with _$TasksState {
  const factory TasksState({
    @Default(TasksStatus.loading) TasksStatus status,

    /// The notebook whose tasks show; null for every notebook.
    String? notebookId,

    /// Whether done tasks show too, in Completed.
    @Default(false) bool showDone,

    /// The keys of the tasks ticked since the place opened: they stay where
    /// they were, struck through, until it is opened again.
    @Default(<String>[]) List<String> kept,

    /// The tasks, in groups; only the groups that have any.
    @Default(TaskBoard(groups: <TaskSection>[])) TaskBoard board,

    /// Why reading failed, while [status] is failed.
    ApplicationError? error,

    /// What went wrong with the last tick, until it is dismissed.
    TasksNotice? notice,
  }) = _TasksState;

  const TasksState._();

  /// The query that reads [board].
  TaskListQuery get query =>
      TaskListQuery(notebookId: notebookId, showDone: showDone, kept: kept);

  /// How many tasks are listed.
  int get count =>
      board.groups.fold(0, (sum, group) => sum + group.items.length);

  /// The tasks listed, in order.
  List<TaskItem> get items => <TaskItem>[
    for (final group in board.groups) ...group.items,
  ];
}
