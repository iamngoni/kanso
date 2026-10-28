// Kanso — journal_screens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/journal/views/widgets/calendar_sheet.dart';

import '../../helpers/library_fixtures.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_browse_app.dart';

const Size _phone = Size(393, 852);

final String _dailyAt =
    '/notebooks/${Uri.encodeComponent(MockIds.journal)}/note/'
    '${Uri.encodeComponent(mockDailyNote.id)}';

Future<BrowseApp> _pump(
  WidgetTester tester,
  PageFixtures fixtures, {
  required String at,
}) => withClock(
  Clock.fixed(mockNow),
  () => pumpBrowseApp(tester, _phone, at: at, fixtures: fixtures),
);

void main() {
  group('the day controls of a daily note', () {
    testWidgets('step to the next day and the day before', (tester) async {
      final fixtures = PageFixtures();
      final app = await _pump(tester, fixtures, at: _dailyAt);
      expect(find.bySemanticsLabel('Previous day'), findsOneWidget);
      expect(find.bySemanticsLabel('Next day'), findsOneWidget);
      expect(find.bySemanticsLabel('Calendar'), findsOneWidget);

      await tester.tap(find.bySemanticsLabel('Next day'));
      await tester.pumpAndSettle();
      expect(fixtures.journal.calls, <String>['step:forward']);
      expect(app.where, contains('note%3Adaily-2026-10-27'));

      await tester.tap(find.bySemanticsLabel('Previous day').first);
      await tester.pumpAndSettle();
      expect(fixtures.journal.calls.last, 'step:back');
      expect(app.where, contains('note%3Adaily-2026-10-26'));
    });

    testWidgets('are not on a note outside the Journal notebook', (
      tester,
    ) async {
      await _pump(
        tester,
        PageFixtures(),
        at: '/notes/note/${Uri.encodeComponent(MockIds.syncDesign)}',
      );
      expect(find.bySemanticsLabel('Previous day'), findsNothing);
      expect(find.bySemanticsLabel('Calendar'), findsNothing);
    });

    testWidgets('say why a day could not be opened, and go on', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      final app = await _pump(tester, fixtures, at: _dailyAt);
      fixtures.journal.failure = const ApplicationError.storage(
        message: 'Disk error.',
      );
      await tester.tap(find.bySemanticsLabel('Next day'));
      await tester.pumpAndSettle();
      expect(find.text('Couldn’t make that note. Try again.'), findsOneWidget);
      expect(app.where, contains('note%3Adaily-2026-10-26'));
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.text('Couldn’t make that note. Try again.'), findsNothing);
    });
  });

  group('the Journal notebook', () {
    testWidgets('has today, the days around it and the calendar', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      final app = await _pump(
        tester,
        fixtures,
        at: '/notebooks/${Uri.encodeComponent(MockIds.journal)}',
      );
      await tester.tap(find.text('Today’s note'));
      await tester.pumpAndSettle();
      expect(fixtures.journal.calls, <String>['today']);
      expect(app.where, contains('note%3Adaily-2026-10-26'));
    });

    testWidgets('another notebook does not', (tester) async {
      await _pump(
        tester,
        PageFixtures(),
        at: '/notebooks/${Uri.encodeComponent(MockIds.work)}',
      );
      expect(find.text('Today’s note'), findsNothing);
    });
  });

  group('the calendar', () {
    Future<void> open(WidgetTester tester) => withClock(
      Clock.fixed(mockNow),
      () async {
        await tester.tap(find.bySemanticsLabel('Calendar'));
        await tester.pumpAndSettle();
      },
    );

    testWidgets('shows the month of the open note, with its hint', (
      tester,
    ) async {
      await _pump(tester, PageFixtures(), at: _dailyAt);
      await open(tester);
      expect(find.byType(CalendarSheet), findsOneWidget);
      expect(find.text('October 2026'), findsOneWidget);
      expect(find.text('Today'), findsOneWidget);
      expect(find.textContaining('A dot marks a day with a note'), findsOne);
      // Days with a note, and today, say so.
      expect(
        find.bySemanticsLabel('Monday, October 26, 2026, today, has a note'),
        findsOneWidget,
      );
      expect(
        find.bySemanticsLabel('Sunday, October 25, 2026, has a note'),
        findsOneWidget,
      );
      expect(
        find.bySemanticsLabel('Tuesday, October 27, 2026'),
        findsOneWidget,
      );
    });

    testWidgets('steps between months and comes back to today', (
      tester,
    ) async {
      await _pump(tester, PageFixtures(), at: _dailyAt);
      await open(tester);
      await tester.tap(find.bySemanticsLabel('Previous month'));
      await tester.pumpAndSettle();
      expect(find.text('September 2026'), findsOneWidget);
      await tester.tap(find.bySemanticsLabel('Next month'));
      await tester.tap(find.bySemanticsLabel('Next month'));
      await tester.pumpAndSettle();
      expect(find.text('November 2026'), findsOneWidget);
      await tester.tap(find.text('Today'));
      await tester.pumpAndSettle();
      expect(find.text('October 2026'), findsOneWidget);
    });

    testWidgets('a day opens that day’s note and closes the sheet', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      final app = await _pump(tester, fixtures, at: _dailyAt);
      await open(tester);
      await tester.tap(
        find.bySemanticsLabel('Monday, October 12, 2026, has a note'),
      );
      await tester.pumpAndSettle();
      expect(find.byType(CalendarSheet), findsNothing);
      expect(fixtures.journal.calls, <String>['day:2026-10-12']);
      expect(app.where, contains('note%3Adaily-2026-10-12'));
    });

    testWidgets('a month that cannot be read can be tried again', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures, at: _dailyAt);
      fixtures.journal.failure = const ApplicationError.storage(
        message: 'Disk error.',
      );
      await open(tester);
      expect(find.text('The calendar couldn’t be read.'), findsOneWidget);
      fixtures.journal.failure = null;
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(find.text('October 2026'), findsOneWidget);
    });
  });

  group('the new menu', () {
    Future<void> openMenu(WidgetTester tester) async {
      await tester.tap(find.bySemanticsLabel('New'));
      await tester.pumpAndSettle();
    }

    testWidgets('is not in the trash', (tester) async {
      await _pump(tester, PageFixtures(), at: '/trash');
      expect(find.bySemanticsLabel('New'), findsNothing);
    });

    testWidgets('makes a new template and opens it', (tester) async {
      final fixtures = PageFixtures();
      final app = await _pump(tester, fixtures, at: '/notes');
      await openMenu(tester);
      await tester.tap(find.text('New template'));
      await tester.pumpAndSettle();
      expect(fixtures.journal.calls, <String>['newTemplate']);
      expect(app.where, contains('note%3Anew-template'));
    });

    testWidgets('makes a note from a template in the notebook shown', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      final app = await _pump(
        tester,
        fixtures,
        at: '/notebooks/${Uri.encodeComponent(MockIds.work)}',
      );
      await openMenu(tester);
      await tester.tap(find.text('New from template…'));
      await tester.pumpAndSettle();
      expect(find.text('Daily'), findsOneWidget);
      expect(find.text('Meeting notes'), findsOneWidget);
      // A template with no title is Untitled.
      expect(find.text('Untitled'), findsOneWidget);
      await tester.tap(find.text('Meeting notes'));
      await tester.pumpAndSettle();
      expect(fixtures.journal.calls, <String>[
        'from:note:tpl-meeting@${MockIds.work}',
      ]);
      expect(app.where, contains('note%3Afrom-note%3Atpl-meeting'));
    });

    testWidgets('lists no templates as an empty state', (tester) async {
      final fixtures = PageFixtures();
      fixtures.journal.templateRows = const [];
      await _pump(tester, fixtures, at: '/notes');
      await openMenu(tester);
      await tester.tap(find.text('New from template…'));
      await tester.pumpAndSettle();
      expect(find.text('No templates yet'), findsOneWidget);
    });

    testWidgets('has today’s note and the calendar', (tester) async {
      final fixtures = PageFixtures();
      final app = await _pump(tester, fixtures, at: '/notes');
      await openMenu(tester);
      await tester.tap(find.text('Today’s note'));
      await tester.pumpAndSettle();
      expect(app.where, contains('note%3Adaily-2026-10-26'));
      // The menu belongs to the list, not the note page: go back to it.
      app.router.go('/notes');
      await tester.pumpAndSettle();
      await openMenu(tester);
      await tester.tap(find.text('Calendar'));
      await tester.pumpAndSettle();
      expect(find.byType(CalendarSheet), findsOneWidget);
    });
  });
}
