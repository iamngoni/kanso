// Kanso — note_assist_intent_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_assist_intent_bloc.dart';

@freezed
abstract class NoteAssistIntentState with _$NoteAssistIntentState {
  const factory NoteAssistIntentState({
    @Default(0) int request,
    @Default('') String noteId,
    AssistAction? action,
    @Default(false) bool summarizeNote,
  }) = _NoteAssistIntentState;

  const NoteAssistIntentState._();

  /// Whether there is an action waiting for its note's editor.
  bool get pending =>
      request > 0 && noteId.isNotEmpty && (summarizeNote || action != null);
}
