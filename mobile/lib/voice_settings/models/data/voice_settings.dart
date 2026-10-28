// Kanso — voice_settings.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'voice_settings.freezed.dart';

/// What the person chose for voice input.
@freezed
abstract class VoiceSettings with _$VoiceSettings {
  const factory VoiceSettings({
    /// Whether "comma", "new paragraph" and "delete that" are acted on
    /// rather than written. On by default, as on the desktop.
    @Default(true) bool spokenCommands,
  }) = _VoiceSettings;
}
