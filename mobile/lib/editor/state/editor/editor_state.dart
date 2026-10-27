// Kanso — editor_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'editor_bloc.dart';

@freezed
abstract class EditorState with _$EditorState {
  const factory EditorState({
    /// The note being edited.
    @Default('') String noteId,
    @Default(EditorStatus.opening) EditorStatus status,

    /// The open note, while [status] is ready.
    EditorDocument? document,

    /// The note as the editor last took it from the core (opened, reloaded
    /// or taken from the other version): the text for the field to show.
    /// Null unless [status] is ready.
    EditorSnapshot? snapshot,

    /// Whether the note is saved, and the question of a change made
    /// elsewhere.
    SaveState? save,

    /// The note's properties.
    @Default(<Property>[]) List<Property> properties,
    @Default(false) bool propertiesOpen,

    /// Why the last change to the properties did not work.
    ApplicationError? propertyError,

    /// What the caret is in, and so what the keyboard bar may do.
    EditorContext? context,

    /// What is offered where the caret is, if anything.
    Completion? completion,

    /// Where the person asked to go (a link followed), for the screen to go
    /// to; it says so with `destinationHandled` when it has.
    EditorDestination? destination,

    /// Why a link could not be opened.
    ApplicationError? linkError,

    /// How many times the field must take the text of [snapshot] again.
    @Default(0) int reloadSeq,

    /// Why opening failed, while [status] is failed.
    ApplicationError? error,
  }) = _EditorState;
}
