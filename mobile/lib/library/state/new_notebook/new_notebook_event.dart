// Kanso — new_notebook_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'new_notebook_bloc.dart';

@freezed
sealed class NewNotebookEvent with _$NewNotebookEvent {
  /// The person asked for a notebook named [name], blank or as a Video
  /// studio.
  const factory NewNotebookEvent.submitted(
    String name, {
    @Default(NewNotebookKind.blank) NewNotebookKind kind,
  }) = NewNotebookSubmittedEvent;
}
