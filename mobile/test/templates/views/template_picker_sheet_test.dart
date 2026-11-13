// Kanso — template_picker_sheet_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/templates/views/widgets/template_picker_sheet.dart';

import '../../helpers/fixture_library_repository.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_routed.dart';
import '../../helpers/register_page_blocs.dart';

void main() {
  late FixtureLibraryRepository library;
  late PageFixtures fixtures;
  String? chosen;

  setUp(() {
    chosen = null;
    library = FixtureLibraryRepository();
    addTearDown(() => unawaited(library.close()));
    fixtures = PageFixtures();
    registerPageBlocsWithFixtures(library, fixtures: fixtures);
  });

  Future<void> open(WidgetTester tester, {String? notebookId}) async {
    await pumpRouted(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () async {
            chosen = await TemplatePickerSheet.show(
              context,
              notebookId: notebookId,
            );
          },
          child: const Text('open'),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
  }

  group('TemplatePickerSheet', () {
    testWidgets('lists the templates plainly when no notebook names any', (
      tester,
    ) async {
      await open(tester, notebookId: 'book:plain');
      expect(find.text('New from template'), findsOneWidget);
      expect(find.text('Daily'), findsOneWidget);
      expect(find.text('Meeting notes'), findsOneWidget);
      expect(find.text('In this notebook'), findsNothing);
      expect(find.text('Other templates'), findsNothing);
      expect(fixtures.journal.templateNotebooks, <String?>['book:plain']);
    });

    testWidgets("puts the notebook's templates first under their own heading", (
      tester,
    ) async {
      fixtures.journal.notebookTemplates['book:studio'] = <String>[
        'note:tpl-meeting',
      ];
      await open(tester, notebookId: 'book:studio');
      expect(find.text('In this notebook'), findsOneWidget);
      expect(find.text('Other templates'), findsOneWidget);
      double top(String text) => tester.getTopLeft(find.text(text)).dy;
      expect(top('In this notebook'), lessThan(top('Meeting notes')));
      expect(top('Meeting notes'), lessThan(top('Other templates')));
      expect(top('Other templates'), lessThan(top('Daily')));
    });

    testWidgets('choosing one returns its ID', (tester) async {
      fixtures.journal.notebookTemplates['book:studio'] = <String>[
        'note:tpl-meeting',
      ];
      await open(tester, notebookId: 'book:studio');
      await tester.tap(find.text('Meeting notes'));
      await tester.pumpAndSettle();
      expect(chosen, 'note:tpl-meeting');
    });

    testWidgets('with no notebook, asks for none', (tester) async {
      await open(tester);
      expect(fixtures.journal.templateNotebooks, <String?>[null]);
      expect(find.text('In this notebook'), findsNothing);
    });

    testWidgets('a notebook naming every template has no second heading', (
      tester,
    ) async {
      fixtures.journal.notebookTemplates['book:studio'] = <String>[
        'note:tpl-daily',
        'note:tpl-meeting',
        'note:tpl-empty',
      ];
      await open(tester, notebookId: 'book:studio');
      expect(find.text('In this notebook'), findsOneWidget);
      expect(find.text('Other templates'), findsNothing);
    });
  });
}
