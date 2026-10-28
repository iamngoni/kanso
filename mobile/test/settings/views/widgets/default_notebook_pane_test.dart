// Kanso — default_notebook_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/views/widgets/kanso_check_row.dart';
import 'package:kanso/settings/views/widgets/default_notebook_pane.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/settings_blocs.dart';

Map<String, bool> _checks(WidgetTester tester) => <String, bool>{
  for (final row in tester.widgetList<KansoCheckRow>(
    find.byType(KansoCheckRow),
  ))
    row.label: row.checked,
};

void main() {
  group('DefaultNotebookPane', () {
    testWidgets('offers the first notebook and the notebooks in order, with '
        'the first checked until one is chosen', (tester) async {
      final blocs = await SettingsBlocs.start();
      await pumpRouted(
        tester,
        blocs.provide(const DefaultNotebookPane()),
        size: const Size(393, 1000),
      );
      final checks = _checks(tester);
      expect(checks.keys.first, 'First notebook');
      expect(checks['First notebook'], isTrue);
      expect(checks.keys, containsAllInOrder(<String>['Inbox', 'Projects']));
      expect(checks.entries.where((e) => e.value), hasLength(1));
      // The Templates notebook is not one notes go in.
      expect(checks.keys, isNot(contains('Templates')));
    });

    testWidgets('choosing a notebook checks it, and saves it for new notes', (
      tester,
    ) async {
      final blocs = await SettingsBlocs.start();
      await pumpRouted(
        tester,
        blocs.provide(const DefaultNotebookPane()),
        size: const Size(393, 1000),
      );
      await tester.tap(find.text('Journal'));
      await tester.pumpAndSettle();
      expect(blocs.defaultNotebook.state.notebookId, MockIds.journal);
      expect(_checks(tester)['Journal'], isTrue);
      expect(_checks(tester)['First notebook'], isFalse);
      await tester.tap(find.text('First notebook'));
      await tester.pumpAndSettle();
      expect(blocs.defaultNotebook.state.notebookId, isNull);
      expect(_checks(tester)['First notebook'], isTrue);
    });

    testWidgets('a notebook that is locked is dimmed and cannot be chosen, '
        'and a choice that is locked now reads as the first', (tester) async {
      final library = FixtureLibraryRepository()
        ..notebookRows = <NotebookRow>[
          for (final row in mockNotebooks)
            if (row.id == MockIds.journal)
              NotebookRow(
                id: row.id,
                name: row.name,
                count: row.count,
                locked: true,
                depth: row.depth,
                hasChildren: false,
              )
            else
              row,
        ];
      final blocs = await SettingsBlocs.start(
        library: library,
        preferences: const Preferences(defaultNotebookId: MockIds.journal),
      );
      await pumpRouted(
        tester,
        blocs.provide(const DefaultNotebookPane()),
        size: const Size(393, 1000),
      );
      expect(_checks(tester)['Journal'], isFalse);
      expect(_checks(tester)['First notebook'], isTrue);
      await tester.tap(find.text('Journal'));
      await tester.pumpAndSettle();
      expect(blocs.defaultNotebook.state.notebookId, MockIds.journal);
    });
  });
}
