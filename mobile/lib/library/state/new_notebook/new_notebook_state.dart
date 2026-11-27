// Kanso — new_notebook_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'new_notebook_bloc.dart';

@freezed
abstract class NewNotebookState with _$NewNotebookState {
  const factory NewNotebookState({
    /// What a notebook can be made from, in order; empty until the sheet
    /// has opened.
    @Default(<StarterInfo>[]) List<StarterInfo> starters,

    /// The notebook the new one is made inside, if any.
    String? parentId,

    /// Whether [parentId] is locked, or inside a locked notebook: the
    /// starters that make notes can't be made there.
    @Default(false) bool parentLocked,

    /// Whether the notebook is being made.
    @Default(false) bool creating,

    /// The notebook made, once it is.
    MadeNotebook? created,

    /// Why none was made.
    NewNotebookFailure? failure,
  }) = _NewNotebookState;
}
