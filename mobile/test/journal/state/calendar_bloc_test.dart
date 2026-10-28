// Kanso — calendar_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/journal/state/calendar/calendar_bloc.dart';

import '../../helpers/fixture_journal_repository.dart';

const _today = DayDate(year: 2026, month: 10, day: 26);

CalendarEvent _started({String? openNote, int weekStart = 0}) =>
    CalendarEvent.started(
      openNote: openNote,
      today: _today,
      weekStart: weekStart,
    );

/// The bloc's stream is broadcast: a state emitted before the listener
/// attached would be missed, so the current state is read first.
Future<void> _ready(CalendarBloc bloc) {
  bool ready(CalendarState state) =>
      state.status == LoadStatus.ready && state.grid != null;
  return ready(bloc.state)
      ? Future<void>.value()
      : bloc.stream.firstWhere(ready);
}

void main() {
  late FixtureJournalRepository fixture;

  setUp(() => fixture = FixtureJournalRepository());

  group('CalendarBloc', () {
    test('opens on the month of today, with today and notes marked', () async {
      final bloc = CalendarBloc(fixture)..add(_started());
      addTearDown(bloc.close);
      await _ready(bloc);
      final grid = bloc.state.grid!;
      expect((grid.year, grid.month), (2026, 10));
      expect(grid.title, 'October 2026');
      final days = grid.weeks.expand((week) => week).toList();
      expect(days.where((day) => day.isToday).single.date, _today);
      expect(
        days.where((day) => day.hasNote && day.inMonth).length,
        5,
      );
    });

    test('opens on the month of the daily note that is open', () async {
      final bloc = CalendarBloc(fixture)
        ..add(_started(openNote: 'note:daily-2026-03-09'));
      addTearDown(bloc.close);
      await _ready(bloc);
      expect(bloc.state.grid!.title, 'March 2026');
    });

    test('a note that is not a daily note opens on today', () async {
      final bloc = CalendarBloc(fixture)
        ..add(_started(openNote: 'note:weekly'));
      addTearDown(bloc.close);
      await _ready(bloc);
      expect(bloc.state.grid!.title, 'October 2026');
    });

    test('weeks start on the day asked', () async {
      final monday = CalendarBloc(fixture)..add(_started());
      final sunday = CalendarBloc(fixture)..add(_started(weekStart: 6));
      addTearDown(monday.close);
      addTearDown(sunday.close);
      await _ready(monday);
      await _ready(sunday);
      expect(monday.state.grid!.weekdayLabels.first, 'Mon');
      expect(sunday.state.grid!.weekdayLabels.first, 'Sun');
    });

    test('steps over the end of a year in both directions', () async {
      final bloc = CalendarBloc(fixture)..add(_started());
      addTearDown(bloc.close);
      await _ready(bloc);
      bloc.add(const CalendarEvent.monthStepped(3));
      await bloc.stream.firstWhere((s) => s.grid?.title == 'January 2027');
      bloc.add(const CalendarEvent.monthStepped(-1));
      await bloc.stream.firstWhere((s) => s.grid?.title == 'December 2026');
      bloc.add(const CalendarEvent.monthStepped(-12));
      await bloc.stream.firstWhere((s) => s.grid?.title == 'December 2025');
    });

    test('today goes back to the month of today', () async {
      final bloc = CalendarBloc(fixture)..add(_started());
      addTearDown(bloc.close);
      await _ready(bloc);
      bloc.add(const CalendarEvent.monthStepped(-5));
      await bloc.stream.firstWhere((s) => s.grid?.title == 'May 2026');
      bloc.add(const CalendarEvent.todayShown());
      await bloc.stream.firstWhere((s) => s.grid?.title == 'October 2026');
    });

    blocTest<CalendarBloc, CalendarState>(
      'a month that cannot be read is a failure; trying again reads it',
      build: () {
        fixture.failure = const ApplicationError.storage(message: 'Disk.');
        return CalendarBloc(fixture);
      },
      act: (bloc) async {
        bloc.add(_started());
        await bloc.stream.firstWhere((s) => s.status == LoadStatus.failed);
        fixture.failure = null;
        bloc.add(const CalendarEvent.retried());
        await _ready(bloc);
      },
      verify: (bloc) {
        expect(bloc.state.error, isNull);
        expect(bloc.state.grid, isNotNull);
      },
    );
  });
}
