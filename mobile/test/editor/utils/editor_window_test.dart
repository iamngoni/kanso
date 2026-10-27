// Kanso — editor_window_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        CombinableAny,
        Generator,
        GeneratorUtils,
        Glados2,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/editor/models/data/editor_window.dart';
import 'package:kanso/editor/utils/diff_values.dart';
import 'package:kanso/editor/utils/editor_window.dart';

import '../../helpers/property.dart';

const int _cap = 40;
const int _margin = 6;

extension _Notes on Any {
  /// Notes of lines of letters, some empty, with a pair of surrogates and a
  /// CJK character among them, up to a few hundred units.
  Generator<String> get note => listWithLengthInRange<String>(
    0,
    300,
    choose(<String>['a', 'b', 'c', ' ', '\n', '\u{1F600}', '日']),
  ).map((chars) => chars.join());

  /// A selection of a note as two fractions of its length, in hundredths.
  Generator<(int, int)> get fractions => combine2(
    intInRange(0, 101),
    intInRange(0, 101),
    (int a, int b) => a <= b ? (a, b) : (b, a),
  );
}

/// The selection [fractions] make in [note], not inside a surrogate pair.
(int, int) _select(String note, (int, int) fractions) {
  int unit(int hundredths) {
    var at = note.length * hundredths ~/ 100;
    if (splitsPair(note, at)) {
      at--;
    }
    return at;
  }

  return (unit(fractions.$1), unit(fractions.$2));
}

bool _wholeLines(String note, EditorWindow window) =>
    (window.start == 0 || note.codeUnitAt(window.start - 1) == 10) &&
    (window.end == note.length || note.codeUnitAt(window.end) == 10);

void main() {
  group('lineStartAt and lineEndAt', () {
    test('find the line an offset is on', () {
      const note = 'ab\ncd\n\nef';
      expect(lineStartAt(note, 0), 0);
      expect(lineStartAt(note, 2), 0);
      expect(lineStartAt(note, 3), 3);
      expect(lineStartAt(note, 6), 6);
      expect(lineStartAt(note, 9), 7);
      expect(lineEndAt(note, 0), 2);
      expect(lineEndAt(note, 2), 2);
      expect(lineEndAt(note, 3), 5);
      expect(lineEndAt(note, 6), 6);
      expect(lineEndAt(note, 9), 9);
    });
  });

  group('windowAround', () {
    test('is the whole note when the note is small', () {
      const note = 'one\ntwo\nthree';
      expect(
        windowAround(note, 4, 4, cap: 100, margin: 5),
        const EditorWindow(start: 0, end: 13),
      );
    });

    test('holds whole lines around the caret within the cap', () {
      final note = List<String>.generate(100, (i) => 'line $i').join('\n');
      final window = windowAround(note, 300, 300, cap: 100, margin: 10);
      expect(window.contains(300), isTrue);
      expect(window.length, lessThanOrEqualTo(100));
      expect(_wholeLines(note, window), isTrue);
      // Centred, as far as lines allow.
      expect(300 - window.start, closeTo(window.end - 300, 20));
    });

    test('holds a selection of more lines than the cap, with a margin', () {
      final note = List<String>.generate(100, (i) => 'line $i').join('\n');
      final window = windowAround(note, 100, 500, cap: 100, margin: 10);
      expect(window.containsRange(100, 500), isTrue);
      expect(window.length, greaterThan(400));
      expect(_wholeLines(note, window), isTrue);
    });

    test('holds a line longer than the cap whole', () {
      final note = 'a' * 500;
      expect(
        windowAround(note, 250, 250, cap: 100, margin: 10),
        const EditorWindow(start: 0, end: 500),
      );
    });

    Glados2<String, (int, int)>(
      any.note,
      any.fractions,
      kansoExploreConfig(),
    ).test('holds the selection, in whole lines, within the cap', (
      note,
      fractions,
    ) {
      final (start, end) = _select(note, fractions);
      final window = windowAround(
        note,
        start,
        end,
        cap: _cap,
        margin: _margin,
      );
      expect(window.containsRange(start, end), isTrue);
      expect(_wholeLines(note, window), isTrue);
      expect(splitsPair(note, window.start), isFalse);
      expect(splitsPair(note, window.end), isFalse);
      // Within the cap, unless the lines of the selection or a margin
      // around them are more than that.
      final first = lineStartAt(note, start);
      final last = lineEndAt(note, end);
      if (last - first + 2 * _margin < _cap) {
        expect(window.length, lessThanOrEqualTo(_cap));
      }
    });
  });

  group('nextWindow', () {
    test('keeps a window that has room around the selection', () {
      final note = List<String>.generate(100, (i) => 'line $i').join('\n');
      final window = windowAround(note, 300, 300, cap: 100, margin: 10);
      expect(nextWindow(note, window, 300, 300, cap: 100, margin: 10), isNull);
    });

    test('moves a window when the caret nears an edge with more beyond', () {
      final note = List<String>.generate(100, (i) => 'line $i').join('\n');
      final window = windowAround(note, 300, 300, cap: 100, margin: 10);
      final near = window.end - 5;
      final moved = nextWindow(note, window, near, near, cap: 100, margin: 10);
      expect(moved, isNotNull);
      expect(moved!.start, greaterThan(window.start));
      expect(moved.contains(near), isTrue);
    });

    test('does not move at the edges of the note', () {
      const note = 'one\ntwo\nthree';
      const window = EditorWindow(start: 0, end: 13);
      expect(nextWindow(note, window, 13, 13, cap: 100, margin: 5), isNull);
      expect(nextWindow(note, window, 0, 0, cap: 100, margin: 5), isNull);
    });

    test('a window grown over the cap is cut back for a small selection', () {
      final note = List<String>.generate(100, (i) => 'line $i').join('\n');
      final whole = wholeNote(note.length);
      final cut = nextWindow(note, whole, 300, 300, cap: 100, margin: 10);
      expect(cut, isNotNull);
      expect(cut!.length, lessThanOrEqualTo(100));
      // With everything selected it stays.
      expect(
        nextWindow(note, whole, 0, note.length, cap: 100, margin: 10),
        isNull,
      );
    });

    Glados2<String, (int, int)>(
      any.note,
      any.fractions,
      kansoExploreConfig(),
    ).test('a window it chooses is kept, and holds the selection', (
      note,
      fractions,
    ) {
      final (start, end) = _select(note, fractions);
      final first = nextWindow(
        note,
        wholeNote(note.length),
        start,
        end,
        cap: _cap,
        margin: _margin,
      );
      final window = first ?? wholeNote(note.length);
      expect(window.containsRange(start, end), isTrue);
      expect(_wholeLines(note, window), isTrue);
      // Asked again with the same selection, it stays.
      expect(
        nextWindow(note, window, start, end, cap: _cap, margin: _margin),
        isNull,
      );
    });

    Glados2<String, (int, int)>(
      any.note,
      any.fractions,
      kansoExploreConfig(),
    ).test('walking the caret along the note always has it in the window', (
      note,
      fractions,
    ) {
      var window = wholeNote(note.length);
      final (target, _) = _select(note, fractions);
      for (var caret = 0; caret <= target; caret += 7) {
        if (splitsPair(note, caret)) {
          continue;
        }
        final next = nextWindow(
          note,
          window,
          caret,
          caret,
          cap: _cap,
          margin: _margin,
        );
        window = next ?? window;
        expect(window.contains(caret), isTrue);
        expect(_wholeLines(note, window), isTrue);
      }
    });
  });
}
