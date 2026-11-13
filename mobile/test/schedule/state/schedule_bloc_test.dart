// Kanso — schedule_bloc_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/schedule_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/schedule/models/data/schedule_status.dart';
import 'package:kanso/schedule/state/schedule/schedule_bloc.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/close_bloc_later.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/fixture_schedule_repository.dart';
import '../views/schedule_harness.dart';

void main() {
  late FixtureScheduleRepository schedule;
  late FixtureLibraryRepository library;
  late RecordingHaptics haptics;
  late UndoNoticeBloc undo;

  setUp(() {
    schedule = FixtureScheduleRepository();
    library = FixtureLibraryRepository();
    haptics = RecordingHaptics();
    undo = UndoNoticeBloc(library, haptics);
    addTearDown(library.close);
    closeBlocLater(undo);
  });

  Future<ScheduleBloc> started() async {
    final bloc = ScheduleBloc(schedule, library, haptics, undo)
      ..add(const ScheduleEvent.started(today: scheduleToday, weekStart: 0));
    closeBlocLater(bloc);
    await bloc.stream.firstWhere((s) => s.status == ScheduleStatus.ready);
    return bloc;
  }

  group('ScheduleBloc', () {
    test('reads the month grid of today, from the first weekday', () async {
      final bloc = await started();
      // October 2026 on a Monday-first grid: 28 Sep to 1 Nov.
      final request = schedule.requests.single;
      expect(request.from, const DayDate(year: 2026, month: 9, day: 28));
      expect(request.until, const DayDate(year: 2026, month: 11, day: 2));
      expect(request.books, isNull);
      expect(request.dateKeys, isNull);
      expect(request.includeDone, isTrue);
      expect(bloc.state.selected, scheduleToday);
      expect(bloc.state.dateKeys, <String>['embargo', 'publish']);
      expect(bloc.state.rows, hasLength(6));
    });

    test('a Sunday-first grid starts a day earlier', () async {
      final bloc = ScheduleBloc(schedule, library, haptics, undo)
        ..add(const ScheduleEvent.started(today: scheduleToday, weekStart: 6));
      closeBlocLater(bloc);
      await bloc.stream.firstWhere((s) => s.status == ScheduleStatus.ready);
      expect(
        schedule.requests.single.from,
        const DayDate(year: 2026, month: 9, day: 27),
      );
    });

    test('stepping a month reads it, and Today comes back', () async {
      final bloc = await started();
      bloc.add(const ScheduleEvent.monthStepped(1));
      await bloc.stream.firstWhere((s) => s.focused.month == 11);
      expect(bloc.state.selected.month, 11);
      expect(schedule.requests.last.from.month, 10);
      bloc.add(const ScheduleEvent.todayShown(scheduleToday));
      await bloc.stream.firstWhere((s) => s.focused.month == 10);
      expect(bloc.state.selected, scheduleToday);
    });

    test('choosing a day of the month does not read again', () async {
      final bloc = await started();
      bloc.add(
        const ScheduleEvent.daySelected(DayDate(year: 2026, month: 10, day: 9)),
      );
      await bloc.stream.firstWhere((s) => s.selected.day == 9);
      expect(schedule.requests, hasLength(1));
    });

    test('choosing a day of a neighbour month moves to that month', () async {
      final bloc = await started();
      bloc.add(
        const ScheduleEvent.daySelected(DayDate(year: 2026, month: 9, day: 29)),
      );
      await bloc.stream.firstWhere((s) => s.focused.month == 9);
      expect(schedule.requests, hasLength(2));
    });

    test('the filters narrow the request and clear again', () async {
      final bloc = await started();
      bloc.add(const ScheduleEvent.notebookToggled('book:travel'));
      await bloc.stream.firstWhere((s) => s.books.isNotEmpty);
      await pumpEventQueue();
      expect(schedule.requests.last.books, <String>['book:travel']);
      expect(schedule.keyQueries.last, <String>['book:travel']);
      bloc.add(const ScheduleEvent.keyToggled('publish'));
      await bloc.stream.firstWhere((s) => s.keys.isNotEmpty);
      await pumpEventQueue();
      expect(schedule.requests.last.dateKeys, <String>['publish']);
      bloc.add(const ScheduleEvent.showDoneChanged(showDone: false));
      await bloc.stream.firstWhere((s) => !s.showDone);
      await pumpEventQueue();
      expect(schedule.requests.last.includeDone, isFalse);
      expect(bloc.state.refined, isTrue);
      bloc
        ..add(const ScheduleEvent.notebooksCleared())
        ..add(const ScheduleEvent.keysCleared())
        ..add(const ScheduleEvent.showDoneChanged(showDone: true));
      await pumpEventQueue();
      expect(bloc.state.refined, isFalse);
      expect(schedule.requests.last.books, isNull);
    });

    test(
      'a date property the notebooks no longer have leaves the filter',
      () async {
        final bloc = await started();
        bloc.add(const ScheduleEvent.keyToggled('publish'));
        await bloc.stream.firstWhere((s) => s.keys.isNotEmpty);
        schedule.keys = <String>['embargo'];
        bloc.add(const ScheduleEvent.notebookToggled('book:travel'));
        await pumpEventQueue();
        expect(bloc.state.keys, isEmpty);
        expect(bloc.state.dateKeys, <String>['embargo']);
      },
    );

    test('a failed reading says so and Retry reads again', () async {
      schedule.scheduleError = const ApplicationError.storage(message: 'x');
      final bloc = ScheduleBloc(schedule, library, haptics, undo)
        ..add(const ScheduleEvent.started(today: scheduleToday, weekStart: 0));
      closeBlocLater(bloc);
      await bloc.stream.firstWhere((s) => s.status == ScheduleStatus.failed);
      schedule.scheduleError = null;
      bloc.add(const ScheduleEvent.retried());
      await bloc.stream.firstWhere((s) => s.status == ScheduleStatus.ready);
      expect(bloc.state.error, isNull);
    });

    test('a library change reads the days again', () async {
      await started();
      final before = schedule.requests.length;
      library.changeStream.add(<String>['n-a']);
      await Future<void>.delayed(
        scheduleDebounce + const Duration(milliseconds: 50),
      );
      expect(schedule.requests.length, greaterThan(before));
    });

    test('ticking a task writes its box and offers Undo', () async {
      final bloc = await started();
      bloc.add(ScheduleEvent.ticked(bloc.state.rows.first));
      await pumpEventQueue();
      expect(library.tasksSet.single, (
        'n-a',
        4,
        'Send Sam the draft @2026-10-26T15:00',
        true,
      ));
      expect(undo.state.notice?.ticket, 7);
      expect(haptics.selections, 1);
    });

    test('ticking a note does nothing', () async {
      final bloc = await started();
      final note = bloc.state.rows.firstWhere(
        (row) => row.kind == ScheduleRowKind.note,
      );
      bloc.add(ScheduleEvent.ticked(note));
      await pumpEventQueue();
      expect(library.tasksSet, isEmpty);
    });

    test('a locked note says so', () async {
      final bloc = await started();
      library.setTaskResult = const Left<ApplicationError, Ticked>(
        ApplicationError.locked(message: 'locked'),
      );
      bloc.add(ScheduleEvent.ticked(bloc.state.rows.first));
      await bloc.stream.firstWhere((s) => s.notice != null);
      expect(bloc.state.notice, ScheduleNotice.locked);
      bloc.add(const ScheduleEvent.noticeDismissed());
      await bloc.stream.firstWhere((s) => s.notice == null);
    });

    test('moving a task or a note asks the core with what it needs', () async {
      final bloc = await started();
      const to = DayDate(year: 2026, month: 10, day: 24);
      final task = bloc.state.rows.first;
      final note = bloc.state.rows.firstWhere(
        (row) => row.kind == ScheduleRowKind.note,
      );
      bloc
        ..add(ScheduleEvent.moved(task, to))
        ..add(ScheduleEvent.moved(note, to));
      await pumpEventQueue();
      expect(schedule.moves, <(String, int?, String, String?, DayDate)>[
        ('n-a', 4, task.text, null, to),
        (note.noteId, null, note.text, note.property, to),
      ]);
      expect(undo.state.notice?.ticket, 9);
    });

    test('a projected occurrence is not moved', () async {
      final bloc = await started();
      final projected = bloc.state.rows.firstWhere((row) => row.projected);
      bloc.add(
        ScheduleEvent.moved(
          projected,
          const DayDate(year: 2026, month: 10, day: 24),
        ),
      );
      await pumpEventQueue();
      expect(schedule.moves, isEmpty);
    });

    test('a move that finds the item gone says so', () async {
      final bloc = await started();
      schedule.moveResult = const Right<ApplicationError, CardMoved>(
        CardMoved.missing(),
      );
      bloc.add(
        ScheduleEvent.moved(
          bloc.state.rows.first,
          const DayDate(year: 2026, month: 10, day: 24),
        ),
      );
      await bloc.stream.firstWhere((s) => s.notice != null);
      expect(bloc.state.notice, ScheduleNotice.moved);
    });
  });
}
