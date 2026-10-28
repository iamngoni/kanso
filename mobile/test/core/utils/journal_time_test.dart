// Kanso — journal_time_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/utils/journal_time.dart';

void main() {
  test('a moment is read off the wall clock', () {
    expect(
      localTimeOf(DateTime(2026, 10, 27, 9, 5, 7)),
      const LocalTime(
        year: 2026,
        month: 10,
        day: 27,
        hour: 9,
        minute: 5,
        second: 7,
      ),
    );
  });

  test('a day is the date in the device’s zone', () {
    expect(
      dayDateOf(DateTime(2026, 12, 31, 23, 59)),
      const DayDate(year: 2026, month: 12, day: 31),
    );
    expect(
      dateTimeOf(const DayDate(year: 2026, month: 2, day: 28)),
      DateTime(2026, 2, 28),
    );
  });

  test('weeks start on Monday for Flutter’s Monday and on Sunday for its '
      'Sunday', () {
    // Flutter counts from Sunday; the library from Monday.
    expect(weekStartOf(0), 6);
    expect(weekStartOf(1), 0);
    expect(weekStartOf(6), 5);
  });
}
