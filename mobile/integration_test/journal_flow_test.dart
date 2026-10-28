// Kanso — journal_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/repos/abs/journal_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/journal/views/widgets/calendar_sheet.dart';
import 'package:kanso/note/views/pages/note_page.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// The journal as a person uses it: today's note, the day before and after
/// it, and a day picked from the calendar.
///
/// Run it with `flutter test integration_test/journal_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  String openNote(WidgetTester tester) =>
      tester.widget<NotePage>(find.byType(NotePage)).noteId;

  Future<DayDate?> openDay(WidgetTester tester) =>
      getIt<JournalRepository>().dailyDate(openNote(tester));

  testWidgets('open today, step between days and pick a day', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    await pumpUntil(tester, inList('Weekly review'));

    // Today's note, from the new menu.
    await tester.tap(find.bySemanticsLabel('New').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Today’s note'));
    await pumpUntil(tester, find.byType(NotePage));
    await pumpUntil(tester, find.bySemanticsLabel('Next day'));
    final today = (await openDay(tester))!;
    final now = DateTime.now();
    expect(
      (today.year, today.month, today.day),
      (now.year, now.month, now.day),
    );

    // The day before, and back.
    final todaysNote = openNote(tester);
    await tester.tap(find.bySemanticsLabel('Previous day'));
    await pumpUntilTrue(
      tester,
      () =>
          find.byType(NotePage).evaluate().isNotEmpty &&
          openNote(tester) != todaysNote,
      reason: 'the note of the day before',
    );
    final before = (await openDay(tester))!;
    expect(
      DateTime(before.year, before.month, before.day),
      DateTime(now.year, now.month, now.day - 1),
    );
    await pumpUntil(tester, find.bySemanticsLabel('Next day'));
    await tester.tap(find.bySemanticsLabel('Next day'));
    await pumpUntilTrue(
      tester,
      () =>
          find.byType(NotePage).evaluate().isNotEmpty &&
          openNote(tester) == todaysNote,
      reason: 'today again',
    );

    // A day of the calendar: the 15th of this month (never in the head or
    // tail of its neighbours).
    await pumpUntil(tester, find.bySemanticsLabel('Calendar'));
    await tester.tap(find.bySemanticsLabel('Calendar'));
    await pumpUntil(tester, find.byType(CalendarSheet));
    await pumpUntil(tester, find.text('15'));
    await tester.tap(
      find.descendant(
        of: find.byType(CalendarSheet),
        matching: find.text('15'),
      ),
    );
    await pumpUntilTrue(
      tester,
      () =>
          find.byType(CalendarSheet).evaluate().isEmpty &&
          find.byType(NotePage).evaluate().isNotEmpty &&
          openNote(tester) != todaysNote,
      reason: 'the note of the 15th',
    );
    final picked = (await openDay(tester))!;
    expect((picked.year, picked.month, picked.day), (now.year, now.month, 15));

    semantics.dispose();
  });
}
