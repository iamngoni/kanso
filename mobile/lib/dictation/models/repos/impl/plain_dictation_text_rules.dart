// Kanso — plain_dictation_text_rules.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/dictated_text.dart';
import '../abs/dictation_text_rules.dart';

/// The rules without commands: the transcript goes in as said, with a space
/// where a word would run into the one before or after it.
///
/// A word gets a space before it when the text before it ends in something
/// other than a space, a newline, an opening bracket, or (in Chinese,
/// Japanese and Thai, which are written without spaces) a letter of the same
/// kind; and one after it when a word follows it directly. Punctuation takes
/// no space before it.
///
/// The app's rules are the Rust core's (`RustDictationTextRules`); these are
/// for tests.
class PlainDictationTextRules implements DictationTextRules {
  @override
  Future<DictatedText?> dictated({
    required String before,
    required String after,
    required String transcript,
  }) async {
    final words = transcript.trim();
    if (words.isEmpty) {
      return null;
    }
    final lead = needsSpaceBetween(
      before.isEmpty ? null : before[before.length - 1],
      words[0],
    );
    final trail = needsSpaceBetween(
      words[words.length - 1],
      after.isEmpty ? null : after[0],
    );
    final text = '${lead ? ' ' : ''}$words${trail ? ' ' : ''}';
    return DictatedText(text: text, caretUtf16: text.length);
  }

  /// Whether a space goes between the character [before] and the character
  /// [after] when they meet; either is null at the edge of the text.
  static bool needsSpaceBetween(String? before, String? after) {
    if (before == null || after == null) {
      return false;
    }
    if (whitespace.hasMatch(before) || whitespace.hasMatch(after)) {
      return false;
    }
    if (opening.contains(before) || closing.contains(after)) {
      return false;
    }
    return !unspaced(before) && !unspaced(after);
  }

  static final RegExp whitespace = RegExp(r'\s');

  /// Characters that open a bracket: nothing is spaced after them.
  static const String opening = '([{';

  /// Characters that close a phrase or a bracket: nothing is spaced before
  /// them.
  static const String closing = '.,;:!?)]}%';

  /// Whether [character] belongs to a script written without spaces between
  /// words: Han, Hiragana, Katakana and Thai, and their full-width marks.
  static bool unspaced(String character) {
    final code = character.runes.first;
    return (code >= 0x0E00 && code <= 0x0E7F) ||
        (code >= 0x3000 && code <= 0x30FF) ||
        (code >= 0x3400 && code <= 0x4DBF) ||
        (code >= 0x4E00 && code <= 0x9FFF) ||
        (code >= 0xF900 && code <= 0xFAFF) ||
        (code >= 0xFF00 && code <= 0xFFEF);
  }
}
