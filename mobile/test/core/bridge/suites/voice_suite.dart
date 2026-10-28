// Kanso — voice_suite.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/voice.dart';

import 'open_library.dart';

/// Voice input, through the bridge: the spoken commands and their setting.
void voiceSuite(OpenLibrary open) {
  group('voice input', () {
    test('spoken punctuation and a UTF-16 caret', () async {
      final said = dictated(
        before: '',
        transcript: 'Buy milk. Comma. Eggs.',
        commands: true,
      );
      expect(said?.text, 'Buy milk, eggs.');
      expect(said?.caretUtf16, 'Buy milk, eggs.'.length);

      // An emoji is two UTF-16 units, as Dart counts them.
      final emoji = dictated(
        before: '',
        transcript: '  hello 😀  ',
        commands: false,
      );
      expect(emoji?.text, 'hello 😀');
      expect(emoji?.caretUtf16, 'hello 😀'.length);
      expect(
        dictated(before: '', transcript: '   ', commands: false),
        isNull,
      );
    });

    test('the setting is on until turned off, and is kept', () async {
      final library = await open();
      expect((await library.voiceSettings()).spokenCommands, isTrue);
      await library.setVoiceSettings(
        settings: const VoiceSettings(spokenCommands: false),
      );
      expect((await library.voiceSettings()).spokenCommands, isFalse);
    });
  });
}
