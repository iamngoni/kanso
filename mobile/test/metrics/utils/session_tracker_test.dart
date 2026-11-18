// Kanso — session_tracker_test.dart
// Created 2026-11-18.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show CombinableAny, Glados, IntAnys, ListAnys, any;
import 'package:kanso/metrics/utils/session_tracker.dart';

import '../../helpers/property.dart';

/// One stretch: `front` seconds in front, then `away` seconds away.
typedef _Trip = ({int front, int away});

void main() {
  final start = DateTime.utc(2026, 11, 18, 9);
  const grace = defaultSessionGrace;

  test('a stay of exactly the grace period continues the session', () {
    var now = start;
    final tracker = SessionTracker(now: now);
    now = now.add(const Duration(seconds: 10));
    expect(tracker.leave(now), isTrue);
    now = now.add(grace);
    expect(tracker.resume(now), isNull);
    now = now.add(const Duration(seconds: 5));
    tracker.leave(now);
    expect(tracker.end(), 15);
  });

  test('ending with no trip away notes nothing', () {
    final tracker = SessionTracker(now: start);
    expect(tracker.end(), isNull);
  });

  Glados<List<_Trip>>(
    any.listWithLengthInRange(
      1,
      40,
      any.combine2(
        any.intInRange(0, 600),
        any.intInRange(0, 300),
        (int front, int away) => (front: front, away: away),
      ),
    ),
    kansoExploreConfig(),
  ).test(
    'sessions end only after a long stay away, and their lengths add up to '
    'the time in front',
    (trips) {
      var now = start;
      final tracker = SessionTracker(now: now);
      final ended = <int>[];
      var longStays = 0;
      var totalFront = 0;
      for (final trip in trips) {
        now = now.add(Duration(seconds: trip.front));
        totalFront += trip.front;
        expect(tracker.leave(now), isTrue);
        // Leaving twice changes nothing.
        expect(tracker.leave(now), isFalse);
        now = now.add(Duration(seconds: trip.away));
        final long = Duration(seconds: trip.away) > grace;
        final result = tracker.resume(now);
        expect(result != null, long);
        if (result != null) {
          ended.add(result);
          longStays += 1;
        }
        // Coming back twice changes nothing.
        expect(tracker.resume(now), isNull);
      }
      // Detached at the end closes the last session.
      now = now.add(const Duration(seconds: 1));
      tracker.leave(now);
      final last = tracker.end();
      expect(last, isNotNull);
      ended.add(last!);
      totalFront += 1;

      expect(ended, hasLength(longStays + 1));
      expect(ended.fold<int>(0, (a, b) => a + b), totalFront);
      expect(tracker.end(), isNull);
    },
  );
}
