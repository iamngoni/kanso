// Kanso — journal_time.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../bridge/api/types.dart';

/// The wall-clock reading of [time] in the device's zone, as the library
/// takes a moment.
LocalTime localTimeOf(DateTime time) {
  final local = time.toLocal();
  return LocalTime(
    year: local.year,
    month: local.month,
    day: local.day,
    hour: local.hour,
    minute: local.minute,
    second: local.second,
  );
}

/// The day [time] falls on in the device's zone.
DayDate dayDateOf(DateTime time) {
  final local = time.toLocal();
  return DayDate(year: local.year, month: local.month, day: local.day);
}

/// Midnight of [day] on the device's clock.
DateTime dateTimeOf(DayDate day) => DateTime(day.year, day.month, day.day);

/// The first column of a calendar whose weeks start on the weekday
/// [firstDayOfWeekIndex] counts from Sunday (Flutter's
/// `MaterialLocalizations.firstDayOfWeekIndex`): 0 for Monday through 6 for
/// Sunday, as the library counts.
int weekStartOf(int firstDayOfWeekIndex) => (firstDayOfWeekIndex + 6) % 7;
