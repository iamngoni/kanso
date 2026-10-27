// Kanso — edit_log_test.dart
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
import 'package:kanso/editor/utils/edit_log.dart';
import 'package:kanso/editor/utils/shift_runs.dart';

import '../../helpers/property.dart';

EditorRun _run(int start, int end, int color) => EditorRun(
  range: Range16(start: start, end: end),
  color: color,
  quiet: false,
);

extension _Edits on Any {
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

  /// A run of changes: where, how much it takes out and how much it puts in.
  Generator<List<(int, int, int)>> get changes => listWithLengthInRange(
    0,
    8,
    combine3(
      intInRange(0, 40),
      intInRange(0, 6),
      intInRange(0, 6),
      (int start, int removed, int inserted) => (start, removed, inserted),
    ),
  );
}

List<(int, int)> _spans(List<EditorRun> runs) =>
    runs.map((run) => (run.range.start, run.range.end)).toList();

void main() {
  group('EditLog', () {
    test('colours of the text as it is need no moving', () {
      final log = EditLog();
      final runs = <EditorRun>[_run(0, 3, 1)];
      expect(log.carry(runs, from: 4, now: 4), same(runs));
    });

    test('colours of an older text are moved through the changes since', () {
      final log = EditLog()
        ..record(from: 4, to: 5, start: 0, end: 0, inserted: 2)
        ..record(from: 5, to: 6, start: 10, end: 11, inserted: 0);
      final carried = log.carry(
        <EditorRun>[_run(3, 6, 1), _run(12, 14, 2)],
        from: 4,
        now: 6,
      );
      expect(_spans(carried!), <(int, int)>[(5, 8), (13, 15)]);
    });

    test('a gap in the changes makes the colours unusable', () {
      final log = EditLog()
        ..record(from: 4, to: 5, start: 0, end: 0, inserted: 1)
        ..record(from: 6, to: 7, start: 0, end: 0, inserted: 1);
      expect(log.carry(<EditorRun>[], from: 4, now: 7), isNull);
      expect(log.carry(<EditorRun>[], from: 3, now: 5), isNull);
    });

    test('a reset ends the chain', () {
      final log = EditLog()
        ..record(from: 4, to: 5, start: 0, end: 0, inserted: 1)
        ..reset();
      expect(log.length, 0);
      expect(log.carry(<EditorRun>[], from: 4, now: 5), isNull);
    });

    test('forgets the changes colours have been shown for', () {
      final log = EditLog()
        ..record(from: 1, to: 2, start: 0, end: 0, inserted: 1)
        ..record(from: 2, to: 3, start: 0, end: 0, inserted: 1)
        ..forget(2);
      expect(log.length, 1);
      expect(log.carry(<EditorRun>[], from: 2, now: 3), isNotNull);
    });

    test('keeps only the latest changes', () {
      final log = EditLog(limit: 3);
      for (var version = 0; version < 10; version++) {
        log.record(
          from: version,
          to: version + 1,
          start: 0,
          end: 0,
          inserted: 1,
        );
      }
      expect(log.length, 3);
      expect(log.carry(<EditorRun>[], from: 6, now: 10), isNull);
      expect(log.carry(<EditorRun>[], from: 7, now: 10), isNotNull);
    });

    Glados2<List<EditorRun>, List<(int, int, int)>>(
      any.laidRuns,
      any.changes,
      kansoExploreConfig(),
    ).test('carrying is shifting through each change in turn', (
      runs,
      changes,
    ) {
      final log = EditLog();
      var expected = runs;
      var version = 0;
      for (final (start, removed, inserted) in changes) {
        log.record(
          from: version,
          to: version + 1,
          start: start,
          end: start + removed,
          inserted: inserted,
        );
        expected = shiftRuns(
          expected,
          start: start,
          end: start + removed,
          inserted: inserted,
        );
        version++;
      }
      final carried = log.carry(runs, from: 0, now: version);
      expect(carried, isNotNull);
      expect(_spans(carried!), _spans(expected));
      // Whatever the changes, the runs stay in order and apart.
      var at = 0;
      for (final run in carried) {
        expect(run.range.start, greaterThanOrEqualTo(at));
        at = run.range.end;
      }
    });
  });
}
