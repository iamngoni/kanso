// Kanso — voice_drops_state.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'voice_drops_bloc.dart';

@freezed
abstract class VoiceDropsState with _$VoiceDropsState {
  const factory VoiceDropsState({
    /// Whether a pass is running.
    @Default(false) bool working,

    /// How many recordings still wait (the phone can't transcribe them yet).
    @Default(0) int waiting,

    /// The notes the last pass made.
    @Default(<String>[]) List<String> saved,

    /// How many notes passes have made since the app started.
    @Default(0) int savedCount,

    /// What was refused, a sentence each.
    @Default(<String>[]) List<String> problems,

    /// The tidy choice the last pass read.
    @Default(VoiceTidyMode.ask) VoiceTidyMode tidyMode,
  }) = _VoiceDropsState;
}
