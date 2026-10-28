// Kanso — voice_note_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'voice_note_text.freezed.dart';

/// What a voice note is written as: its title and its body.
@freezed
abstract class VoiceNoteText with _$VoiceNoteText {
  const factory VoiceNoteText({
    required String title,
    required String body,
  }) = _VoiceNoteText;
}
