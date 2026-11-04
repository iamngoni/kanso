// Kanso — player_clock_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:glados/glados.dart';
import 'package:kanso/recording/utils/player_clock.dart';

void main() {
  test('time reads minutes and seconds, then hours', () {
    expect(playerClock(0), '00:00');
    expect(playerClock(12000), '00:12');
    expect(playerClock(754000), '12:34');
    expect(playerClock(3723000), '1:02:03');
    expect(playerClock(-5), '00:00');
    expect(playerClock(999), '00:00');
  });

  test('speeds read as on the button', () {
    expect(playerSpeedLabel(1), '1×');
    expect(playerSpeedLabel(0.75), '0.75×');
    expect(playerSpeedLabel(1.25), '1.25×');
    expect(playerSpeedLabel(1.5), '1.5×');
    expect(playerSpeedLabel(2), '2×');
  });

  Glados<int>(any.intInRange(0, 400000)).test(
    'the clock reads back as the same seconds',
    (seconds) {
      final parts = playerClock(
        seconds * 1000,
      ).split(':').map(int.parse).toList();
      expect(parts.length, anyOf(2, 3));
      final total = parts.length == 3
          ? parts[0] * 3600 + parts[1] * 60 + parts[2]
          : parts[0] * 60 + parts[1];
      // Past an hour the minutes are two digits of 60; under it they may
      // run on to 59 only.
      expect(total, seconds);
    },
  );

  Glados2<double, int>(
    any.doubleInRange(-2, 3),
    any.intInRange(0, 1 << 30),
  ).test('a seek lands inside the recording and reads back as the fraction', (
    fraction,
    length,
  ) {
    final at = playerMomentAt(fraction, length);
    expect(at, inInclusiveRange(0, length));
    if (length > 1000 && fraction >= 0 && fraction <= 1) {
      expect(playerProgress(at, length), closeTo(fraction, 0.001));
    }
    expect(playerProgress(at, length), inInclusiveRange(0.0, 1.0));
  });

  Glados2<int, int>(
    any.intInRange(-10, 1 << 30),
    any.intInRange(-10, 1 << 30),
  ).test('progress is always between nothing and all', (position, length) {
    expect(playerProgress(position, length), inInclusiveRange(0.0, 1.0));
  });
}
