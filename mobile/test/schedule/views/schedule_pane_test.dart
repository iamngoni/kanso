// Kanso — schedule_pane_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/schedule_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_checkbox.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_month_grid.dart';
import 'package:kanso/schedule/views/widgets/schedule_item_tile.dart';
import 'package:kanso/schedule/views/widgets/schedule_pane.dart';

import '../../helpers/fixture_schedule_repository.dart';
import '../../helpers/pump_routed.dart';
import 'schedule_harness.dart';

Future<ScheduleHarness> _show(
  WidgetTester tester, {
  FixtureScheduleRepository? schedule,
}) async {
  final harness = await ScheduleHarness.start(schedule: schedule);
  await pumpRouted(tester, harness.pane);
  await tester.pumpAndSettle();
  return harness;
}

Future<void> _pickDay(WidgetTester tester, String day) async {
  await tester.tap(find.text(day).first);
  await tester.pumpAndSettle();
}

Finder _icon(KansoIcons icon) => find.byWidgetPredicate(
  (widget) => widget is KansoIcon && widget.icon == icon,
);

/// A repeat's ↻ (the loop icon) beside how it repeats.
Finder get _repeat => find.ancestor(
  of: find.text('every week'),
  matching: find.byWidgetPredicate(
    (widget) =>
        widget is Row &&
        widget.children.whereType<KansoIcon>().any(
          (icon) => icon.icon == KansoIcons.rotateCw,
        ),
  ),
);

double _opacityOf(WidgetTester tester, Finder tile) => tester
    .widget<Opacity>(
      find.descendant(of: tile, matching: find.byType(Opacity)).first,
    )
    .opacity;

void main() {
  group('SchedulePane month view', () {
    testWidgets('shows the month, the weekdays and the chosen day', (
      tester,
    ) async {
      await _show(tester);
      expect(find.text('October 2026'), findsOneWidget);
      expect(find.text('Schedule'), findsOneWidget);
      // Today is chosen: its tasks and notes are listed under the grid.
      expect(find.text('Monday, October 26'), findsOneWidget);
      expect(find.text('Send Sam the draft'), findsOneWidget);
      expect(find.text('Studio tour'), findsOneWidget);
      expect(find.text('2 items'), findsOneWidget);
      // A task has a box; a note has the mark of its property (pause: embargo).
      expect(find.byType(KansoCheckbox), findsOneWidget);
      expect(_icon(KansoIcons.pause), findsOneWidget);
      expect(find.text('15:00'), findsOneWidget);
    });

    testWidgets(
      'puts a date-property note on its day, with its play mark and its zone',
      (
        tester,
      ) async {
        await _show(tester);
        await _pickDay(tester, '23');
        expect(find.text('Friday, October 23'), findsOneWidget);
        expect(find.text('Pixel 11 review'), findsWidgets);
        expect(_icon(KansoIcons.play), findsOneWidget);
        expect(find.text('09:00 New York'), findsOneWidget);
      },
    );

    testWidgets('marks a repeating task with ↻ and dims a projected one', (
      tester,
    ) async {
      await _show(tester);
      await _pickDay(tester, '20');
      var tile = tileOf('Film review');
      expect(_repeat, findsOneWidget);
      expect(_opacityOf(tester, tile), 1);
      await _pickDay(tester, '27');
      tile = tileOf('Film review');
      expect(_repeat, findsOneWidget);
      expect(_opacityOf(tester, tile), ScheduleItemTile.projectedOpacity);
    });

    testWidgets('strikes a done task through', (tester) async {
      await _show(tester);
      await _pickDay(tester, '22');
      final text = tester.widget<Text>(find.text('Buy oat milk'));
      expect(text.style?.decoration, TextDecoration.lineThrough);
    });

    testWidgets('says when a day has nothing on it', (tester) async {
      await _show(tester);
      await _pickDay(tester, '21');
      expect(find.text('Nothing on this day'), findsOneWidget);
    });

    testWidgets('steps to the next month and reads it', (tester) async {
      final harness = await _show(tester);
      await tester.tap(buttonLabelled('Next month'));
      await tester.pumpAndSettle();
      expect(find.text('November 2026'), findsOneWidget);
      expect(harness.schedule.requests.last.from.month, 10);
      expect(harness.bloc.state.focused.month, 11);
    });
  });

  group('SchedulePane agenda view', () {
    Future<ScheduleHarness> agenda(WidgetTester tester) async {
      final harness = await _show(tester);
      await tester.tap(find.text('Agenda'));
      await tester.pumpAndSettle();
      return harness;
    }

    testWidgets('lists the days that have something, in order', (
      tester,
    ) async {
      await agenda(tester);
      final headers = <String>[
        'Tue, Oct 20',
        'Thu, Oct 22',
        'Fri, Oct 23',
        'Mon, Oct 26',
        'Tue, Oct 27',
      ];
      var last = -1.0;
      for (final header in headers) {
        final top = tester.getTopLeft(find.text(header)).dy;
        expect(top, greaterThan(last), reason: header);
        last = top;
      }
      // Days with nothing on them are left out.
      expect(find.text('Wed, Oct 21'), findsNothing);
      // Today's header says so.
      expect(find.text('Today'), findsNWidgets(2));
    });

    testWidgets('shows tasks, notes, repeats and projected occurrences', (
      tester,
    ) async {
      await agenda(tester);
      expect(find.text('Send Sam the draft'), findsOneWidget);
      expect(find.text('Pixel 11 review'), findsWidgets);
      expect(_icon(KansoIcons.pause), findsOneWidget);
      expect(_icon(KansoIcons.play), findsOneWidget);
      expect(_repeat, findsNWidgets(2));
      final projected = find.byWidgetPredicate(
        (widget) => widget is ScheduleItemTile && widget.row.projected,
      );
      expect(projected, findsOneWidget);
      expect(
        _opacityOf(tester, projected),
        ScheduleItemTile.projectedOpacity,
      );
    });

    testWidgets('says nothing is scheduled in an empty month', (tester) async {
      final harness = await agenda(tester);
      harness.schedule.rows = <ScheduleRow>[];
      await tester.tap(buttonLabelled('Next month'));
      await tester.pumpAndSettle();
      expect(find.text('Nothing scheduled'), findsOneWidget);
    });
  });

  group('SchedulePane ticking', () {
    testWidgets('ticks a task with its box: one write, the Undo notice', (
      tester,
    ) async {
      final harness = await _show(tester);
      await tester.tapAt(tester.getCenter(find.byType(KansoCheckbox)));
      await tester.pumpAndSettle();
      expect(harness.blocs.library.tasksSet, <(String, int, String, bool)>[
        ('n-a', 4, 'Send Sam the draft @2026-10-26T15:00', true),
      ]);
      expect(
        harness.blocs.undoNotice.state.notice?.message,
        'Ticked off “Call Sam”',
      );
      expect(harness.blocs.haptics.selections, 1);
      // The days were read again.
      expect(harness.schedule.requests.length, greaterThan(1));
    });

    testWidgets('clears a done task', (tester) async {
      final harness = await _show(tester);
      await _pickDay(tester, '22');
      await tester.tapAt(tester.getCenter(find.byType(KansoCheckbox)));
      await tester.pumpAndSettle();
      expect(harness.blocs.library.tasksSet.single.$4, isFalse);
    });

    testWidgets('ticking a projected occurrence ticks the real task', (
      tester,
    ) async {
      final harness = await _show(tester);
      await _pickDay(tester, '27');
      await tester.tapAt(tester.getCenter(find.byType(KansoCheckbox)));
      await tester.pumpAndSettle();
      // The line and words are those of the written task, not a new one.
      expect(harness.blocs.library.tasksSet, <(String, int, String, bool)>[
        ('n-c', 7, 'Film review @2026-10-20T15:00 every week', true),
      ]);
    });

    testWidgets('says an item changed in its note', (tester) async {
      final harness = await _show(tester);
      harness.blocs.library.setTaskResult =
          const Right<ApplicationError, Ticked>(Ticked.missing());
      await tester.tapAt(tester.getCenter(find.byType(KansoCheckbox)));
      await tester.pumpAndSettle();
      expect(
        find.text(
          'That item changed in its note. The schedule is up to date now.',
        ),
        findsOneWidget,
      );
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.textContaining('changed in its note'), findsNothing);
    });

    testWidgets('says a locked note cannot be ticked', (tester) async {
      final harness = await _show(tester);
      harness.blocs.library.setTaskResult =
          const Left<ApplicationError, Ticked>(
            ApplicationError.locked(message: 'locked'),
          );
      await tester.tapAt(tester.getCenter(find.byType(KansoCheckbox)));
      await tester.pumpAndSettle();
      expect(
        find.text('That note is locked. Unlock it to change it.'),
        findsOneWidget,
      );
    });
  });

  group('SchedulePane Move to day…', () {
    Future<void> openMove(WidgetTester tester, String title) async {
      await tester.tap(moreOf(tileOf(title)));
      await tester.pumpAndSettle();
    }

    Future<void> moveTo(WidgetTester tester, String day) async {
      await tester.tap(find.text('Move to day…'));
      await tester.pumpAndSettle();
      expect(find.text('Move to day'), findsOneWidget);
      await tester.tap(
        find.descendant(
          of: find.byType(KansoMonthGrid),
          matching: find.text(day),
        ),
      );
      await tester.pumpAndSettle();
    }

    testWidgets('moves a task to the day picked, keeping its words', (
      tester,
    ) async {
      final harness = await _show(tester);
      await openMove(tester, 'Send Sam the draft');
      await moveTo(tester, '24');
      expect(harness.schedule.moves, <(String, int?, String, String?, DayDate)>[
        (
          'n-a',
          4,
          'Send Sam the draft @2026-10-26T15:00',
          null,
          const DayDate(year: 2026, month: 10, day: 24),
        ),
      ]);
      expect(
        harness.blocs.undoNotice.state.notice?.message,
        'Moved to Friday.',
      );
    });

    testWidgets('moves a note by its date property', (tester) async {
      final harness = await _show(tester);
      await _pickDay(tester, '23');
      await openMove(tester, 'Pixel 11 review');
      await moveTo(tester, '25');
      expect(harness.schedule.moves, <(String, int?, String, String?, DayDate)>[
        (
          'n-b',
          null,
          'Pixel 11 review',
          'publish',
          const DayDate(year: 2026, month: 10, day: 25),
        ),
      ]);
    });

    testWidgets('offers no move for a projected occurrence', (tester) async {
      final harness = await _show(tester);
      await _pickDay(tester, '27');
      await openMove(tester, 'Film review');
      expect(find.text('Move to day…'), findsNothing);
      expect(find.textContaining('A repeat coming round'), findsOneWidget);
      // It can still be ticked and opened from its actions.
      expect(find.text('Tick off'), findsOneWidget);
      expect(find.text('Open note'), findsOneWidget);
      expect(harness.schedule.moves, isEmpty);
    });

    testWidgets('says an item changed when it is gone', (tester) async {
      final harness = await _show(tester);
      harness.schedule.moveResult = const Right<ApplicationError, CardMoved>(
        CardMoved.missing(),
      );
      await openMove(tester, 'Send Sam the draft');
      await moveTo(tester, '24');
      expect(find.textContaining('changed in its note'), findsOneWidget);
    });
  });

  group('SchedulePane filters', () {
    Future<void> openFilter(WidgetTester tester) async {
      await tester.tap(buttonLabelled('Filter schedule'));
      await tester.pumpAndSettle();
    }

    testWidgets('narrows to a notebook, and back to all', (tester) async {
      final harness = await _show(tester);
      await openFilter(tester);
      expect(find.text('Notebook'), findsOneWidget);
      await tester.ensureVisible(find.text('Travel'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Travel'));
      await tester.pumpAndSettle();
      expect(harness.schedule.requests.last.books, <String>['book:travel']);
      expect(harness.bloc.state.refined, isTrue);
      await tester.ensureVisible(find.text('All notebooks'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('All notebooks'));
      await tester.pumpAndSettle();
      expect(harness.schedule.requests.last.books, isNull);
      expect(harness.schedule.keyQueries.last, isNull);
    });

    testWidgets('narrows to a date property', (tester) async {
      final harness = await _show(tester);
      await openFilter(tester);
      expect(find.text('Date property'), findsOneWidget);
      await tester.tap(find.text('embargo'));
      await tester.pumpAndSettle();
      expect(harness.schedule.requests.last.dateKeys, <String>['embargo']);
      await tester.tap(find.text('All date properties'));
      await tester.pumpAndSettle();
      expect(harness.schedule.requests.last.dateKeys, isNull);
    });

    testWidgets('says when no notebook has a date property', (tester) async {
      final schedule = FixtureScheduleRepository()..keys = <String>[];
      await _show(tester, schedule: schedule);
      await openFilter(tester);
      expect(find.text('No date properties yet'), findsOneWidget);
    });

    testWidgets('hides done tasks, and shows them again', (tester) async {
      final harness = await _show(tester);
      await _pickDay(tester, '22');
      expect(find.text('Buy oat milk'), findsOneWidget);
      await openFilter(tester);
      expect(harness.bloc.state.showDone, isTrue);
      await tester.tap(find.text('Show Completed'));
      await tester.pumpAndSettle();
      expect(harness.schedule.requests.last.includeDone, isFalse);
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(find.text('Buy oat milk'), findsNothing);
      await openFilter(tester);
      await tester.tap(find.text('Show Completed'));
      await tester.pumpAndSettle();
      expect(harness.schedule.requests.last.includeDone, isTrue);
    });
  });

  group('SchedulePane while reading', () {
    testWidgets('says why the schedule did not load, and tries again', (
      tester,
    ) async {
      final schedule = FixtureScheduleRepository()
        ..scheduleError = const ApplicationError.storage(message: 'disk');
      final harness = await _show(tester, schedule: schedule);
      expect(find.text('The schedule didn’t load'), findsOneWidget);
      schedule.scheduleError = null;
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(find.text('The schedule didn’t load'), findsNothing);
      expect(harness.bloc.state.rows, isNotEmpty);
    });

    testWidgets('opens the pane with the shape of it while it is read', (
      tester,
    ) async {
      final harness = await ScheduleHarness.start(read: false);
      await pumpRouted(tester, harness.pane);
      expect(find.byType(SchedulePane), findsOneWidget);
      expect(find.text('Send Sam the draft'), findsNothing);
    });
  });
}
