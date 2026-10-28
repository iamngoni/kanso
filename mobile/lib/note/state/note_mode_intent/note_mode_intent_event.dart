// Kanso — note_mode_intent_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_mode_intent_bloc.dart';

@freezed
sealed class NoteModeIntentEvent with _$NoteModeIntentEvent {
  const factory NoteModeIntentEvent.requested({
    required String noteId,
    required NoteMode mode,
  }) = NoteModeIntentRequestedEvent;
}
