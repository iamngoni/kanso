// Kanso — dictation_insertion.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../models/data/dictated_text.dart';
import '../models/data/dictation_edit.dart';
import '../models/repos/abs/dictation_text_rules.dart';

/// Where dictated words go in a note.
///
/// One dictation is one [DictationInsertion], made for it. The words said so
/// far are one run of text in the note, the dictated range: the first update
/// makes it at the selection (replacing what was selected), and each update
/// after it replaces the whole range with what the rules make of everything
/// said so far. Replacing the range, not adding to it, is what lets a rule
/// like "delete that" take back words already in the note, and only words of
/// this dictation: text outside the range is never changed.
///
/// If the person typed or moved in between, so that the range is no longer
/// where it was put, the dictation carries on at the selection with what is
/// said from then on, and leaves the words already in the note alone.
class DictationInsertion {
  DictationInsertion({required this.rules});

  final DictationTextRules rules;

  /// The dictated range in the note as the last edit left it.
  TextRange? range;
  String rangeText = '';

  /// How many of the phrases said have gone into earlier ranges.
  int settled = 0;

  /// How many phrases the last update had.
  int seen = 0;

  /// The edit that makes [value]'s dictated range say [phrases], or null when
  /// there is nothing to change. Apply it before asking for the next.
  ///
  /// [phrases] are the finished phrases, all of them since the dictation
  /// began, and [partial] the words still being spoken, if any.
  Future<DictationEdit?> next(
    TextEditingValue value,
    List<String> phrases, {
    String partial = '',
  }) async {
    final planned = await plan(value, phrases, partial);
    if (planned == null) {
      seen = phrases.length;
      return null;
    }
    final (target, from, made) = planned;
    final text = value.text;
    final inserted = made?.text ?? '';
    settled = from;
    seen = phrases.length;
    range = TextRange(start: target.start, end: target.start + inserted.length);
    rangeText = inserted;
    if (inserted == text.substring(target.start, target.end)) {
      return null;
    }
    return DictationEdit(range: target, text: inserted);
  }

  /// What the dictated range would say for [phrases] and [partial], without
  /// changing anything: the words as they would land, for the pill.
  Future<String> preview(
    TextEditingValue value,
    List<String> phrases,
    String partial,
  ) async {
    final planned = await plan(value, phrases, partial);
    return planned?.$3?.text ?? '';
  }

  /// Where the next update goes, from which phrase it starts, and what the
  /// rules make of it; null when nothing was said and nothing is in the
  /// note yet, so that the selection is not replaced by nothing.
  Future<(TextRange, int, DictatedText?)?> plan(
    TextEditingValue value,
    List<String> phrases,
    String partial,
  ) async {
    final text = value.text;
    final previous = range;
    final established =
        previous != null &&
        previous.start >= 0 &&
        previous.end <= text.length &&
        text.substring(previous.start, previous.end) == rangeText;
    final target = established ? previous : anchor(value);
    // What went into a lost range stays where it is.
    final from = established ? settled : seen;
    final said = <String>[
      ...phrases.skip(from),
      if (partial.trim().isNotEmpty) partial,
    ].join(' ');
    if (said.trim().isEmpty && !established) {
      return null;
    }
    final made = await rules.dictated(
      before: text.substring(0, target.start),
      after: nextScalar(text, target.end),
      transcript: said,
    );
    return (target, from, made);
  }

  /// Where a dictation starts in [value]: the selection, or the end of the
  /// text when the editor has none.
  TextRange anchor(TextEditingValue value) {
    final text = value.text;
    final selection = value.selection;
    if (selection.isValid &&
        selection.start >= 0 &&
        selection.end <= text.length) {
      return TextRange(start: selection.start, end: selection.end);
    }
    return TextRange.collapsed(text.length);
  }

  /// Reads one Unicode scalar at a UTF-16 offset. Never gives text rules a
  /// lone surrogate when the next character is a supplementary scalar.
  static String nextScalar(String text, int offset) {
    if (offset < 0 || offset >= text.length) {
      return '';
    }
    final first = text.codeUnitAt(offset);
    if (first >= 0xD800 && first <= 0xDBFF) {
      if (offset + 1 >= text.length) {
        return '';
      }
      final second = text.codeUnitAt(offset + 1);
      return second >= 0xDC00 && second <= 0xDFFF
          ? text.substring(offset, offset + 2)
          : '';
    }
    if (first >= 0xDC00 && first <= 0xDFFF) {
      return '';
    }
    return text.substring(offset, offset + 1);
  }

  /// [text] with [edit] made to it.
  static String apply(String text, DictationEdit edit) =>
      text.replaceRange(edit.range.start, edit.range.end, edit.text);
}
