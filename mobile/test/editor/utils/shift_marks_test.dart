// Kanso — shift_marks_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart' show TextRange;
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados3, IntAnys, any;
import 'package:kanso/editor/utils/shift_marks.dart';

import '../../helpers/property.dart';

/// The words of [mark] in [text].
String _words(String text, TextRange mark) =>
    text.substring(mark.start, mark.end);

void main() {
  group('shiftMarks', () {
    const mark = TextRange(start: 6, end: 10); // "beta" in "Alpha beta gamma"

    test('text typed before a mark moves it on', () {
      expect(
        shiftMarks(<TextRange>[mark], start: 0, end: 0, inserted: 3),
        <TextRange>[const TextRange(start: 9, end: 13)],
      );
    });

    test('typing at either edge stays outside it', () {
      expect(
        shiftMarks(<TextRange>[mark], start: 6, end: 6, inserted: 2),
        <TextRange>[const TextRange(start: 8, end: 12)],
      );
      expect(
        shiftMarks(<TextRange>[mark], start: 10, end: 10, inserted: 2),
        <TextRange>[mark],
      );
    });

    test('typing inside it grows it; deleting all of it drops it', () {
      expect(
        shiftMarks(<TextRange>[mark], start: 8, end: 8, inserted: 1),
        <TextRange>[const TextRange(start: 6, end: 11)],
      );
      expect(
        shiftMarks(<TextRange>[mark], start: 5, end: 11, inserted: 0),
        isEmpty,
      );
    });

    Glados3<int, int, int>(
      any.intInRange(0, 17),
      any.intInRange(0, 17),
      any.intInRange(0, 5),
      kansoExploreConfig(),
    ).test('a change that leaves a mark’s words keeps it on the same words', (
      a,
      b,
      inserted,
    ) {
      const text = 'Alpha beta gamma';
      final start = a < b ? a : b;
      final end = a < b ? b : a;
      final edited =
          text.substring(0, start) + 'x' * inserted + text.substring(end);
      final moved = shiftMarks(
        <TextRange>[mark],
        start: start,
        end: end,
        inserted: inserted,
      );
      for (final shifted in moved) {
        expect(shifted.start, lessThan(shifted.end));
        expect(shifted.end, lessThanOrEqualTo(edited.length));
      }
      final untouched = end <= mark.start || start >= mark.end;
      if (untouched) {
        expect(_words(edited, moved.single), 'beta');
      }
    });
  });
}
