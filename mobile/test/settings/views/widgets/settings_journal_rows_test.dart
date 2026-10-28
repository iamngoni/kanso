// Kanso — settings_journal_rows_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/reminder_permission.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_settings_row.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/page_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/settings_blocs.dart';

Future<SettingsBlocs> _pump(
  WidgetTester tester, {
  PageFixtures? fixtures,
}) async {
  final page = fixtures ?? PageFixtures();
  final blocs = await SettingsBlocs.start(fixtures: page);
  final appearance = SeededAppearanceBloc(const AppearanceState());
  closeBlocLater(appearance);
  registerPageBlocsWithFixtures(
    FixtureLibraryRepository(),
    fixtures: page,
  );
  await pumpRouted(
    tester,
    BlocProvider<AppearanceBloc>.value(
      value: appearance,
      child: blocs.provide(const SettingsPane()),
    ),
    size: const Size(393, 1600),
  );
  await tester.pumpAndSettle();
  return blocs;
}

void main() {
  group('Daily notes', () {
    testWidgets('the daily template is None until one is chosen', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Daily notes'), findsOneWidget);
      expect(find.text('Daily template'), findsOneWidget);
      expect(find.text('None'), findsNWidgets(2));
      expect(
        find.text(
          'New daily notes start from this template, with {{date}}, {{time}} '
          'and {{cursor}} filled in.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('with no templates it says how to make one', (tester) async {
      final fixtures = PageFixtures()..journal.templateRows = const [];
      await _pump(tester, fixtures: fixtures);
      expect(
        find.text(
          'New daily notes start empty. Notes in the Templates notebook can '
          'be chosen here; New template makes one.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('choosing a template sets it; None clears it', (tester) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures: fixtures);
      await tester.tap(find.text('Daily template'));
      await tester.pumpAndSettle();
      // None first, checked; then the templates.
      expect(find.text('None'), findsNWidgets(3));
      await tester.tap(find.text('Meeting notes'));
      await tester.pumpAndSettle();
      expect(fixtures.journal.daily, 'note:tpl-meeting');
      expect(find.text('Meeting notes'), findsOneWidget);

      await tester.tap(find.text('Daily template'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('None').last);
      await tester.pumpAndSettle();
      expect(fixtures.journal.daily, isNull);
      expect(find.text('Meeting notes'), findsNothing);
    });

    testWidgets('a template with no title is Untitled', (tester) async {
      final fixtures = PageFixtures()..journal.daily = 'note:tpl-empty';
      await _pump(tester, fixtures: fixtures);
      expect(find.text('Untitled'), findsOneWidget);
    });

    testWidgets('a template that could not be set is said so', (tester) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures: fixtures);
      // The picker lists the templates when it opens; the choosing then
      // fails. Failing the listing first would leave nothing to choose.
      await tester.tap(find.text('Daily template'));
      await tester.pumpAndSettle();
      fixtures.journal.failure = const ApplicationError.storage(
        message: 'Disk error.',
      );
      await tester.tap(find.text('Daily').last);
      await tester.pumpAndSettle();
      expect(find.text('Couldn’t set the daily template.'), findsOneWidget);
    });
  });

  group('Reminders', () {
    // The pane has other switches now (the discovery ones): the reminders
    // switch is the one in the "Remind me of tasks" row.
    final remindersSwitch = find.descendant(
      of: find.ancestor(
        of: find.text('Remind me of tasks'),
        matching: find.byType(KansoSettingsRow),
      ),
      matching: find.byType(KansoSwitch),
    );
    bool switchIsOn(WidgetTester tester) =>
        tester.widget<KansoSwitch>(remindersSwitch).value;

    testWidgets('are on to begin with, with the desktop’s words', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Reminders'), findsOneWidget);
      expect(find.text('Remind me of tasks'), findsOneWidget);
      expect(switchIsOn(tester), isTrue);
      expect(
        find.text(
          'A notification when a task with a time comes due, like “- [ ] '
          'Call Sam @2026-10-25T09:00”.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('the switch turns them off and on, asking once', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures: fixtures);
      await tester.tap(remindersSwitch);
      await tester.pumpAndSettle();
      expect(switchIsOn(tester), isFalse);
      expect(fixtures.notifier.requests, 0);
      expect(fixtures.notifier.cancels, 1);
      await tester.tap(find.text('Remind me of tasks'));
      await tester.pumpAndSettle();
      expect(switchIsOn(tester), isTrue);
      expect(fixtures.notifier.requests, 1);
      expect(fixtures.notifier.current, ReminderPermission.granted);
    });

    testWidgets('a refusal says how to allow them', (tester) async {
      final fixtures = PageFixtures();
      fixtures.notifier.allow = false;
      await _pump(tester, fixtures: fixtures);
      await tester.tap(remindersSwitch);
      await tester.pumpAndSettle();
      await tester.tap(remindersSwitch);
      await tester.pumpAndSettle();
      expect(
        find.text(
          'Notifications are turned off for Kanso. Allow them in your '
          'device’s settings to get reminders.',
        ),
        findsOneWidget,
      );
    });
  });
}
