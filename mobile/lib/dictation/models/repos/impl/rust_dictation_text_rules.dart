// Kanso — rust_dictation_text_rules.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/voice.dart' as voice;
import '../../../../voice_settings/models/repos/abs/voice_settings_repository.dart';
import '../../data/dictated_text.dart';
import '../abs/dictation_text_rules.dart';

/// The Rust core's rules for dictated words (`kanso_engine::voice_commands`):
/// the spacing and capitals that follow the marks, and, when the person has
/// them on, spoken punctuation and commands. The same rules as the desktop's.
@LazySingleton(as: DictationTextRules)
class RustDictationTextRules implements DictationTextRules {
  RustDictationTextRules(this.settings);

  final VoiceSettingsRepository settings;

  @override
  Future<DictatedText?> dictated({
    required String before,
    required String after,
    required String transcript,
  }) async {
    // A failed read of the choice leaves the commands on, their default.
    final commands = (await settings.settings()).fold(
      (error) => true,
      (kept) => kept.spokenCommands,
    );
    final made = voice.dictated(
      before: before,
      after: after.isEmpty ? null : after,
      transcript: transcript,
      commands: commands,
    );
    return made == null
        ? null
        : DictatedText(text: made.text, caretUtf16: made.caretUtf16);
  }
}
