// Kanso — new_notebook_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'new_notebook_bloc.dart';

@freezed
sealed class NewNotebookEvent with _$NewNotebookEvent {
  /// The sheet opened: read the starters it offers and whether the notebook
  /// it was opened inside, [parentId], is locked.
  const factory NewNotebookEvent.opened({String? parentId}) =
      NewNotebookOpenedEvent;

  /// The person asked for a notebook named [name], blank, from a starter or
  /// as a Video studio.
  const factory NewNotebookEvent.submitted(
    String name, {
    @Default(NewNotebookKind.blank) NewNotebookKind kind,
  }) = NewNotebookSubmittedEvent;
}
