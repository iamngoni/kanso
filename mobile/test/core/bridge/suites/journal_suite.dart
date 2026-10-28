// Kanso — journal_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The Journal, through the bridge: today's note, any day's, stepping between
/// days, the month calendar and the platform calendars' first weekday. It
/// runs on this computer (`test/core/bridge`) and on a simulator or an
/// emulator (`integration_test`).
void journalSuite(OpenLibrary open) {
  group('KansoLibrary journal', () {
    // 09:30 on 2026-05-20, a Wednesday.
    const now = LocalTime(
      year: 2026,
      month: 5,
      day: 20,
      hour: 9,
      minute: 30,
      second: 0,
    );
    const today = DayDate(year: 2026, month: 5, day: 20);
    const third = DayDate(year: 2026, month: 5, day: 3);

    test('today, any day, and the days between', () async {
      final library = await open();
      final todayNote = await library.journalToday(now: now);
      expect(todayNote.titleFirst, isFalse);
      expect(KansoLibrary.dailyDateOf(noteId: todayNote.id), today);
      // Asked again, the same note: nothing is made twice.
      expect((await library.journalToday(now: now)).id, todayNote.id);
      expect((await library.note(id: todayNote.id)).title, '2026-05-20');
      expect(
        (await library.journalOpenDay(day: today, now: now)).id,
        todayNote.id,
      );

      final thirdNote = await library.journalOpenDay(day: third, now: now);
      expect(KansoLibrary.dailyDateOf(noteId: thirdNote.id), third);
      expect((await library.note(id: thirdNote.id)).title, '2026-05-03');
      // A note that names no day is no daily note.
      expect(KansoLibrary.dailyDateOf(noteId: 'note:abc'), isNull);

      // Stepping finds the nearest day with a note in that direction.
      final back = await library.journalStep(
        openNote: todayNote.id,
        now: now,
        forward: false,
      );
      expect(back?.id, thirdNote.id);
      final forward = await library.journalStep(
        openNote: thirdNote.id,
        now: now,
        forward: true,
      );
      expect(forward?.id, todayNote.id);
      // Nothing later than today, nothing before the first day.
      expect(
        await library.journalStep(
          openNote: todayNote.id,
          now: now,
          forward: true,
        ),
        isNull,
      );
      expect(
        await library.journalStep(
          openNote: thirdNote.id,
          now: now,
          forward: false,
        ),
        isNull,
      );
    });

    test('the month is whole weeks with the days that have a note', () async {
      final library = await open();
      await library.journalToday(now: now);
      await library.journalOpenDay(day: third, now: now);

      final grid = await library.journalMonth(
        year: 2026,
        month: 5,
        weekStart: 0,
        today: today,
      );
      expect(grid.year, 2026);
      expect(grid.month, 5);
      expect(grid.title, isNotEmpty);
      expect(grid.weekdayLabels, hasLength(7));
      expect(grid.weeks, isNotEmpty);
      for (final week in grid.weeks) {
        expect(week, hasLength(7));
      }
      final days = grid.weeks.expand((week) => week).toList();
      final inMonth = days.where((day) => day.inMonth).toList();
      expect(inMonth, hasLength(31));
      expect(
        inMonth.map((day) => day.date.day).toSet(),
        equals(<int>{for (var day = 1; day <= 31; day++) day}),
      );
      expect(
        days.where((day) => day.hasNote).map((day) => day.date),
        unorderedEquals(<DayDate>[third, today]),
      );
      expect(
        days.where((day) => day.isToday).map((day) => day.date),
        <DayDate>[today],
      );
    });

    test('the platforms first weekday is the grids first column', () {
      // Foundation and Java count 1 for Sunday through 7 for Saturday.
      expect(KansoLibrary.journalWeekStart(firstWeekday: 1), 6);
      expect(KansoLibrary.journalWeekStart(firstWeekday: 2), 0);
      expect(KansoLibrary.journalWeekStart(firstWeekday: 3), 1);
      expect(KansoLibrary.journalWeekStart(firstWeekday: 7), 5);
      // Anything the calendar doesn't know starts on Monday.
      expect(KansoLibrary.journalWeekStart(firstWeekday: 0), 0);
      expect(KansoLibrary.journalWeekStart(firstWeekday: 8), 0);
    });

    test('days and times that do not exist are refused', () async {
      final library = await open();
      await expectLater(
        library.journalToday(
          now: const LocalTime(
            year: 2026,
            month: 5,
            day: 20,
            hour: 24,
            minute: 0,
            second: 0,
          ),
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.journalOpenDay(
          day: const DayDate(year: 2026, month: 2, day: 30),
          now: now,
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.journalMonth(year: 2026, month: 13, weekStart: 0, today: today),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.journalMonth(year: 2026, month: 5, weekStart: 7, today: today),
        throwsA(isA<KansoError_Invalid>()),
      );
    });
  });
}
