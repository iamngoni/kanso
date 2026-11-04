// Kanso — recording_clock_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:glados/glados.dart';
import 'package:kanso/recording/utils/recording_clock.dart';

void main() {
  test('the clock reads hours, minutes and seconds', () {
    expect(recordingClock(0), '00:00:00');
    expect(recordingClock(42 * 60 + 17), '00:42:17');
    expect(recordingClock(3723), '01:02:03');
    expect(recordingClock(-5), '00:00:00');
  });

  Glados<int>(any.intInRange(0, 400000)).test(
    'the clock is hh:mm:ss and reads back as the same seconds',
    (seconds) {
      final parts = recordingClock(seconds).split(':').map(int.parse).toList();
      expect(parts, hasLength(3));
      expect(parts[1], lessThan(60));
      expect(parts[2], lessThan(60));
      expect(parts[0] * 3600 + parts[1] * 60 + parts[2], seconds);
    },
  );
}
