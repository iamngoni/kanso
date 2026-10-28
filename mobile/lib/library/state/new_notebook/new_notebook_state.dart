// Kanso — new_notebook_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'new_notebook_bloc.dart';

@freezed
abstract class NewNotebookState with _$NewNotebookState {
  const factory NewNotebookState({
    /// Whether the notebook is being made.
    @Default(false) bool creating,

    /// The notebook made, once it is.
    NotebookRow? created,

    /// Why none was made.
    NewNotebookFailure? failure,
  }) = _NewNotebookState;
}
