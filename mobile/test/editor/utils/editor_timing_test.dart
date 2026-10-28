// Kanso — editor_timing_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/editor/utils/editor_timing.dart';

void main() {
  tearDown(() => EditorTiming.observer = null);

  test('measure returns what the body returns, observed or not', () {
    expect(EditorTiming.observed, isFalse);
    expect(EditorTiming.measure('kanso.test', () => 7), 7);
    final seen = <String>[];
    EditorTiming.observer = (name, micros) {
      expect(micros, greaterThanOrEqualTo(0));
      seen.add(name);
    };
    expect(EditorTiming.observed, isTrue);
    expect(EditorTiming.measure('kanso.test', () => 8), 8);
    expect(seen, <String>['kanso.test']);
  });

  test('a body that throws is still timed', () {
    final seen = <String>[];
    EditorTiming.observer = (name, _) => seen.add(name);
    expect(
      () =>
          EditorTiming.measure<void>('kanso.test', () => throw StateError('x')),
      throwsStateError,
    );
    expect(seen, <String>['kanso.test']);
  });

  test('since reports the time from a moment to now', () {
    int? micros;
    EditorTiming.observer = (_, taken) => micros = taken;
    EditorTiming.since('kanso.freshRuns', EditorTiming.nowMicros - 5000);
    expect(micros, greaterThanOrEqualTo(5000));
  });
}
