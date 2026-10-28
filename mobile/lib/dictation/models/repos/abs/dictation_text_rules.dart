// Kanso — dictation_text_rules.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/dictated_text.dart';

/// How spoken words become text in the note: spacing at the edges, spoken
/// punctuation and commands ("comma", "new paragraph", "delete that").
///
/// Every phrase on its way to the note, the words still being spoken and the
/// final ones alike, goes through the same rules, so that what the pill
/// shows is what lands.
// One method, and an interface the Rust-backed rules and test fakes implement.
// ignore: one_member_abstracts
abstract interface class DictationTextRules {
  /// The text that goes in for [transcript], everything said this time,
  /// between [before], the note's text up to where it goes, and [after], the
  /// character that follows it (empty at the end of the note). Null when
  /// nothing is left to put in.
  ///
  /// The rules see only what was said this time: "delete that" takes out the
  /// last phrase said, never the note's own text.
  Future<DictatedText?> dictated({
    required String before,
    required String after,
    required String transcript,
  });
}
