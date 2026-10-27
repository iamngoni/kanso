// Kanso — shift_runs_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        CombinableAny,
        Generator,
        GeneratorUtils,
        Glados2,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/utils/shift_runs.dart';

import '../../helpers/property.dart';

EditorRun _run(int start, int end, [int color = 0]) => EditorRun(
  range: Range16(start: start, end: end),
  color: color,
  quiet: false,
);

extension _Runs on Any {
  /// Runs in order that do not overlap, in a text of up to 40 units.
  Generator<List<EditorRun>> get laidRuns =>
      listWithLengthInRange<int>(0, 7, intInRange(0, 7)).map((gaps) {
        final runs = <EditorRun>[];
        var at = 0;
        for (var i = 0; i + 1 < gaps.length; i += 2) {
          final start = at + gaps[i];
          final end = start + gaps[i + 1] + 1;
          runs.add(_run(start, end, runs.length + 1));
          at = end;
        }
        return runs;
      });

  /// A change: where it starts, how many units it takes out and how many it
  /// puts in.
  Generator<(int, int, int)> get change => combine3(
    intInRange(0, 40),
    intInRange(0, 8),
    intInRange(0, 8),
    (int start, int removed, int inserted) => (start, removed, inserted),
  );
}

void main() {
  group('shiftRuns', () {
    test('leaves runs before the change and moves those after it', () {
      final shifted = shiftRuns(
        <EditorRun>[_run(0, 2), _run(10, 14)],
        start: 5,
        end: 5,
        inserted: 3,
      );
      expect(shifted.map((r) => (r.range.start, r.range.end)), <(int, int)>[
        (0, 2),
        (13, 17),
      ]);
    });

    test('grows the run text is typed inside or at the end of', () {
      expect(
        shiftRuns(
          <EditorRun>[_run(2, 6)],
          start: 4,
          end: 4,
          inserted: 1,
        ).single.range,
        const Range16(start: 2, end: 7),
      );
      expect(
        shiftRuns(
          <EditorRun>[_run(2, 6)],
          start: 6,
          end: 6,
          inserted: 2,
        ).single.range,
        const Range16(start: 2, end: 8),
      );
      // Typed at the start of a run, the run moves on.
      expect(
        shiftRuns(
          <EditorRun>[_run(2, 6)],
          start: 2,
          end: 2,
          inserted: 2,
        ).single.range,
        const Range16(start: 4, end: 8),
      );
    });

    test('cuts and drops runs a deletion covers', () {
      final shifted = shiftRuns(
        <EditorRun>[
          _run(0, 4, 1),
          _run(4, 6, 2),
          _run(6, 10, 3),
          _run(10, 12, 4),
        ],
        start: 3,
        end: 8,
        inserted: 0,
      );
      expect(shifted.map((r) => (r.range.start, r.range.end)), <(int, int)>[
        (0, 3),
        (3, 5),
        (5, 7),
      ]);
      expect(shifted.map((r) => r.color), <int?>[1, 3, 4]);
    });

    test('gives the new text to one run when a change covers two in part', () {
      final shifted = shiftRuns(
        <EditorRun>[_run(0, 4, 1), _run(4, 8, 2)],
        start: 2,
        end: 6,
        inserted: 2,
      );
      expect(shifted.map((r) => (r.range.start, r.range.end)), <(int, int)>[
        (0, 4),
        (4, 6),
      ]);
    });

    test('drops a run the change covers entirely, whatever it puts in', () {
      final shifted = shiftRuns(
        <EditorRun>[_run(3, 5, 1)],
        start: 2,
        end: 6,
        inserted: 3,
      );
      expect(shifted, isEmpty);
    });

    test('keeps what a run says besides where it is', () {
      const run = EditorRun(
        range: Range16(start: 1, end: 3),
        color: 7,
        italic: true,
        weight: 700,
        quiet: true,
      );
      final shifted = shiftRuns(
        <EditorRun>[run],
        start: 0,
        end: 0,
        inserted: 1,
      );
      expect(shifted.single.color, 7);
      expect(shifted.single.italic, isTrue);
      expect(shifted.single.weight, 700);
      expect(shifted.single.quiet, isTrue);
    });

    Glados2<List<EditorRun>, (int, int, int)>(
      any.laidRuns,
      any.change,
      kansoExploreConfig(),
    ).test('the runs stay in order and apart', (runs, change) {
      final (start, removed, inserted) = change;
      final shifted = shiftRuns(
        runs,
        start: start,
        end: start + removed,
        inserted: inserted,
      );
      var at = 0;
      for (final run in shifted) {
        expect(run.range.start, greaterThanOrEqualTo(at));
        expect(run.range.end, greaterThan(run.range.start));
        at = run.range.end;
      }
    });

    Glados2<List<EditorRun>, (int, int, int)>(
      any.laidRuns,
      any.change,
      kansoExploreConfig(),
    ).test('text outside the change keeps its colour', (runs, change) {
      final (start, removed, inserted) = change;
      final end = start + removed;
      final shifted = shiftRuns(
        runs,
        start: start,
        end: end,
        inserted: inserted,
      );
      int? colourAt(List<EditorRun> in_, int at) {
        for (final run in in_) {
          if (run.range.start <= at && at < run.range.end) {
            return run.color;
          }
        }
        return null;
      }

      final delta = inserted - removed;
      for (var at = 0; at < 60; at++) {
        if (at < start) {
          expect(colourAt(shifted, at), colourAt(runs, at), reason: 'at $at');
        } else if (at >= start + inserted && at + 0 >= start + inserted) {
          final before = at - delta;
          if (before >= end) {
            expect(
              colourAt(shifted, at),
              colourAt(runs, before),
              reason: 'at $at',
            );
          }
        }
      }
    });
  });
}
