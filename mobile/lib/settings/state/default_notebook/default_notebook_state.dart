// Kanso — default_notebook_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'default_notebook_bloc.dart';

@freezed
abstract class DefaultNotebookState with _$DefaultNotebookState {
  const factory DefaultNotebookState({
    /// The notebook chosen; null is the first notebook.
    String? notebookId,
  }) = _DefaultNotebookState;
}
