// Kanso — voice_note_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'voice_note_bloc.dart';

@freezed
abstract class VoiceNoteState with _$VoiceNoteState {
  const factory VoiceNoteState({
    @Default(VoiceNoteStatus.idle) VoiceNoteStatus status,

    /// The note made, while [status] is saved.
    String? noteId,

    /// Why saving failed, while [status] is failed.
    ApplicationError? error,
  }) = _VoiceNoteState;
}
