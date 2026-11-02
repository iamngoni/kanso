// Kanso — voice_drops_event.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'voice_drops_bloc.dart';

@freezed
sealed class VoiceDropsEvent with _$VoiceDropsEvent {
  /// The app started or came back to the front, or the person allowed the
  /// server recognizer: transcribe the recordings that wait.
  const factory VoiceDropsEvent.passRequested() = VoiceDropsPassRequestedEvent;
}
