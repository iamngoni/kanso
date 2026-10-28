// Kanso — default_notebook_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'default_notebook_bloc.dart';

@freezed
sealed class DefaultNotebookEvent with _$DefaultNotebookEvent {
  /// The app started: read the saved choice.
  const factory DefaultNotebookEvent.started() = DefaultNotebookStartedEvent;

  /// The person chose the notebook [notebookId], or, with null, the first
  /// notebook.
  const factory DefaultNotebookEvent.chosen(String? notebookId) =
      DefaultNotebookChosenEvent;
}
