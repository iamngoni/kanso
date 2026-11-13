// Kanso — schedule_days_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados3, IntAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/schedule/utils/schedule_days.dart';

import '../../helpers/property.dart';

void main() {
  group('schedule days', () {
    test('October 2026 is five Monday-first weeks', () {
      final weeks = monthWeeks(
        const DayDate(year: 2026, month: 10, day: 26),
        0,
      );
      expect(weeks, hasLength(5));
      expect(weeks.first.first, const DayDate(year: 2026, month: 9, day: 28));
      expect(weeks.last.last, const DayDate(year: 2026, month: 11, day: 1));
    });

    test('adding months keeps the day or the last of the month', () {
      const jan31 = DayDate(year: 2027, month: 1, day: 31);
      expect(
        addMonths(jan31, 1),
        const DayDate(year: 2027, month: 2, day: 28),
      );
      expect(
        addMonths(jan31, -2),
        const DayDate(year: 2026, month: 11, day: 30),
      );
      expect(
        addMonths(jan31, 12),
        const DayDate(year: 2028, month: 1, day: 31),
      );
    });

    test('notes are marked by their property', () {
      expect(noteMark('publish'), KansoIcons.play);
      expect(noteMark('embargo'), KansoIcons.pause);
      expect(noteMark('shoot'), KansoIcons.calendar);
      expect(noteMark(null), KansoIcons.calendar);
    });

    Glados3<int, int, int>(
      any.intInRange(2020, 2036),
      any.intInRange(1, 13),
      any.intInRange(0, 7),
      kansoExploreConfig(),
    ).test('a month grid is whole weeks of every day, from the first weekday', (
      year,
      month,
      weekStart,
    ) {
      final focus = DayDate(year: year, month: month, day: 1);
      final weeks = monthWeeks(focus, weekStart);
      final days = <DayDate>[for (final week in weeks) ...week];
      expect(weeks.length, inInclusiveRange(4, 6));
      expect(days.length, weeks.length * 7);
      // Every week starts on the weekday asked for, and the days run on
      // without a gap.
      for (final week in weeks) {
        expect((dateOf(week.first).weekday - 1) % 7, weekStart);
      }
      for (var at = 1; at < days.length; at++) {
        expect(
          dateOf(days[at]).difference(dateOf(days[at - 1])).inHours,
          inInclusiveRange(23, 25),
        );
      }
      // The month's own days are all there, and the first and last weeks
      // each hold a day of the month.
      final inMonth = days.where((day) => day.month == month).length;
      expect(inMonth, DateTime(year, month + 1, 0).day);
      expect(weeks.first.any((day) => day.month == month), isTrue);
      expect(weeks.last.any((day) => day.month == month), isTrue);
      // The range the library is asked for is exactly the grid.
      final (from, until) = monthRange(focus, weekStart);
      expect(from, days.first);
      expect(
        dateOf(until).difference(dateOf(days.last)).inHours,
        inInclusiveRange(23, 25),
      );
    });

    Glados3<int, int, int>(
      any.intInRange(2020, 2036),
      any.intInRange(1, 13),
      any.intInRange(-24, 25),
      kansoExploreConfig(),
    ).test('stepping months lands in the month asked for', (
      year,
      month,
      delta,
    ) {
      final day = DayDate(year: year, month: month, day: 28);
      final next = addMonths(day, delta);
      final expected = DateTime(year, month + delta);
      expect(next.year, expected.year);
      expect(next.month, expected.month);
      expect(next.day, 28);
    });
  });
}
