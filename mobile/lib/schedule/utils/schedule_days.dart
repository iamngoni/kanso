// Kanso — schedule_days.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/schedule_types.dart';
import '../../core/bridge/api/types.dart';
import '../../core/configs/kanso_icons.dart';

/// [day] as a [DateTime] at midnight, on the device's clock.
DateTime dateOf(DayDate day) => DateTime(day.year, day.month, day.day);

/// [date]'s day.
DayDate dayOf(DateTime date) =>
    DayDate(year: date.year, month: date.month, day: date.day);

/// The weeks of [focus]'s month, each of seven days, from the weekday
/// [weekStart] (0 for Monday through 6 for Sunday, as the library counts);
/// the days of the months either side fill the first and last week.
List<List<DayDate>> monthWeeks(DayDate focus, int weekStart) {
  final first = DateTime(focus.year, focus.month);
  final lead = (first.weekday - 1 - weekStart + 7) % 7;
  final days = DateTime(focus.year, focus.month + 1, 0).day;
  final count = ((lead + days) / 7).ceil();
  return <List<DayDate>>[
    for (var week = 0; week < count; week++)
      <DayDate>[
        for (var day = 0; day < 7; day++)
          dayOf(DateTime(focus.year, focus.month, 1 - lead + week * 7 + day)),
      ],
  ];
}

/// The first day and the day after the last of [focus]'s month grid.
(DayDate, DayDate) monthRange(DayDate focus, int weekStart) {
  final weeks = monthWeeks(focus, weekStart);
  final last = dateOf(weeks.last.last);
  return (
    weeks.first.first,
    dayOf(DateTime(last.year, last.month, last.day + 1)),
  );
}

/// The day a step of [delta] months from [day]: the same day of it, or its
/// last.
DayDate addMonths(DayDate day, int delta) {
  final month = DateTime(day.year, day.month + delta);
  final last = DateTime(month.year, month.month + 1, 0).day;
  return DayDate(
    year: month.year,
    month: month.month,
    day: day.day > last ? last : day.day,
  );
}

/// The rows by their day, each day's in the order given.
Map<DayDate, List<ScheduleRow>> rowsByDay(List<ScheduleRow> rows) {
  final days = <DayDate, List<ScheduleRow>>{};
  for (final row in rows) {
    days.putIfAbsent(row.date, () => <ScheduleRow>[]).add(row);
  }
  return days;
}

/// The small mark before a note's title, the desktop's ▶ ⏸ ◆ drawn as
/// icons (the glyphs turn into emoji on a phone): play for what publishes,
/// pause for an embargo, a calendar for any other date property.
KansoIcons noteMark(String? property) => switch (property) {
  'publish' => KansoIcons.play,
  'embargo' => KansoIcons.pause,
  _ => KansoIcons.calendar,
};

/// Whether a row can be moved to another day: a note, or a task's own date.
/// A projected occurrence is only the repeat coming round and has no date of
/// its own to change.
bool canMove(ScheduleRow row) =>
    row.kind == ScheduleRowKind.note || !row.projected;
