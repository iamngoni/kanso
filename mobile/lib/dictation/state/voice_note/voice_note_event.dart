// Kanso — voice_note_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'voice_note_bloc.dart';

@freezed
sealed class VoiceNoteEvent with _$VoiceNoteEvent {
  /// The microphone came on: the clock for the note's length starts.
  const factory VoiceNoteEvent.listeningStarted() =
      VoiceNoteListeningStartedEvent;

  /// The person is done: save [transcript] as a new note.
  const factory VoiceNoteEvent.saveRequested(String transcript) =
      VoiceNoteSaveRequestedEvent;
}
