// Kanso — fake_dictation_text_rules.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/dictation/models/data/dictated_text.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_text_rules.dart';

/// The transcript as said: no spacing, no commands.
class PassThroughRules implements DictationTextRules {
  @override
  Future<DictatedText?> dictated({
    required String before,
    required String after,
    required String transcript,
  }) async {
    final text = transcript.trim();
    return text.isEmpty
        ? null
        : DictatedText(text: text, caretUtf16: text.length);
  }
}

/// A few of the desktop's commands, enough to tell what the dictated range
/// does with them: "comma" is a comma, and "delete that" takes out the word
/// before it. Words are spaced by single spaces.
class CommandRules implements DictationTextRules {
  @override
  Future<DictatedText?> dictated({
    required String before,
    required String after,
    required String transcript,
  }) async {
    final out = <String>[];
    final words = transcript.trim().split(RegExp(r'\s+'));
    for (var i = 0; i < words.length; i++) {
      final word = words[i];
      if (word == 'comma') {
        if (out.isNotEmpty) {
          out[out.length - 1] = '${out.last},';
        }
      } else if (word == 'delete' &&
          i + 1 < words.length &&
          words[i + 1] == 'that') {
        if (out.isNotEmpty) {
          out.removeLast();
        }
        i++;
      } else if (word.isNotEmpty) {
        out.add(word);
      }
    }
    if (out.isEmpty) {
      return null;
    }
    final lead =
        before.isNotEmpty && !before.endsWith(' ') && !before.endsWith('\n');
    final text = '${lead ? ' ' : ''}${out.join(' ')}';
    return DictatedText(text: text, caretUtf16: text.length);
  }
}
