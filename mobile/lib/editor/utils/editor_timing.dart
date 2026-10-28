// Kanso — editor_timing.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:developer';

/// Where the editor's own work is timed.
///
/// Every span is a `Timeline` span (`kanso.apply`, `kanso.buildTextSpan`),
/// which the profiler shows. A performance test that cannot reach the
/// profiler (the bench app, on a phone) sets `observer` to be told the
/// duration of each span, and of the time colours took to arrive
/// (`kanso.freshRuns`). With no observer the cost is the timeline's.
abstract final class EditorTiming {
  /// Told the name and duration in microseconds of each timed span; null
  /// outside the performance tests.
  static void Function(String name, int micros)? observer;

  /// Whether anything is listening.
  static bool get observed => observer != null;

  /// Runs [body] as the span [name].
  static T measure<T>(String name, T Function() body) {
    final seen = observer;
    if (seen == null) {
      return Timeline.timeSync(name, body);
    }
    final watch = Stopwatch()..start();
    try {
      return Timeline.timeSync(name, body);
    } finally {
      seen(name, watch.elapsedMicroseconds);
    }
  }

  /// The moment now, in microseconds, in the clock of `Timeline` and of
  /// frame timestamps.
  static int get nowMicros => Timeline.now;

  /// Tells the observer that [name] took from [startedUs] to now.
  static void since(String name, int startedUs) {
    observer?.call(name, Timeline.now - startedUs);
  }
}
