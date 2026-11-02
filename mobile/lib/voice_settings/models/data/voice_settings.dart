// Kanso — voice_settings.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).
import 'package:freezed_annotation/freezed_annotation.dart';

import 'voice_tidy_mode.dart';

part 'voice_settings.freezed.dart';

/// What the person chose for voice input and voice capture.
@freezed
abstract class VoiceSettings with _$VoiceSettings {
  const factory VoiceSettings({
    /// Whether "comma", "new paragraph" and "delete that" are acted on
    /// rather than written. On by default, as on the desktop.
    @Default(true) bool spokenCommands,

    /// Tidy voice notes: off, ask each time, or always.
    @Default(VoiceTidyMode.ask) VoiceTidyMode tidy,

    /// Whether a recording longer than three minutes is a meeting, which
    /// tidy gives a Summary, Decisions, Action items and Notes.
    @Default(true) bool meetings,

    /// Whether the recording's audio is kept with the note. A meeting's is
    /// kept either way.
    @Default(false) bool keepAudio,

    /// Whether the system's server-backed recognizer may transcribe a
    /// recording the phone can't do itself. Off: it waits.
    @Default(false) bool allowServerRecognition,
  }) = _VoiceSettings;
}
