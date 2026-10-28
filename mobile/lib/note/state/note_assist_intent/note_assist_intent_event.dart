// Kanso — note_assist_intent_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_assist_intent_bloc.dart';

@freezed
sealed class NoteAssistIntentEvent with _$NoteAssistIntentEvent {
  /// Runs [action] on the open note, or opens its whole-note summary when
  /// [summarizeNote] is true.
  const factory NoteAssistIntentEvent.requested({
    required String noteId,
    AssistAction? action,
    @Default(false) bool summarizeNote,
  }) = NoteAssistIntentRequestedEvent;

  /// Marks [request] as handled, so returning to the note does not run it
  /// again.
  const factory NoteAssistIntentEvent.consumed({required int request}) =
      NoteAssistIntentConsumedEvent;
}
