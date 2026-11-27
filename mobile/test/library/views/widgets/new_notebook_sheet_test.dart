// Kanso — new_notebook_sheet_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/library/views/widgets/new_notebook_sheet.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';

void main() {
  late FixtureLibraryRepository library;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(() => unawaited(library.close()));
    registerPageBlocs(library);
  });

  Future<GoRouter> open(WidgetTester tester, {NotebookRow? parent}) async {
    final router = await pumpRouted(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () =>
              unawaited(NewNotebookSheet.open(context, parent: parent)),
          child: const Text('open'),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    return router;
  }

  KansoButton create(WidgetTester tester) =>
      tester.widget<KansoButton>(find.widgetWithText(KansoButton, 'Create'));

  group('NewNotebookSheet', () {
    testWidgets('asks for a name, and cannot be created without one', (
      tester,
    ) async {
      await open(tester);
      expect(find.text('New notebook'), findsOneWidget);
      expect(find.text('Name'), findsWidgets);
      expect(create(tester).onPressed, isNull);
      await tester.enterText(find.byType(EditableText), '  ');
      await tester.pump();
      expect(create(tester).onPressed, isNull);
      await tester.enterText(find.byType(EditableText), 'Recipes');
      await tester.pump();
      expect(create(tester).onPressed, isNotNull);
    });

    testWidgets('Create makes the notebook, closes, and goes to it', (
      tester,
    ) async {
      final router = await open(tester);
      await tester.enterText(find.byType(EditableText), 'Recipes');
      await tester.pump();
      await tester.tap(find.widgetWithText(KansoButton, 'Create'));
      await tester.pumpAndSettle();
      expect(library.notebooksCreated, <(String, String?)>[('Recipes', null)]);
      expect(find.text('New notebook'), findsNothing);
      expect(
        Uri.decodeComponent(
          router.routerDelegate.currentConfiguration.uri.toString(),
        ),
        '/notebooks/book:made-1',
      );
    });

    testWidgets("offers the library's starters, blank first, studio last", (
      tester,
    ) async {
      await open(tester);
      final labels = <String>[
        for (final starter in mockStarters) starter.label,
      ];
      expect(labels, hasLength(8));
      var top = double.negativeInfinity;
      for (final label in labels) {
        expect(find.text(label), findsOneWidget);
        final y = tester.getTopLeft(find.text(label)).dy;
        expect(y, greaterThan(top), reason: label);
        top = y;
      }
      expect(
        find.textContaining('Idea \u2192 Scripting \u2192 Shooting'),
        findsOneWidget,
      );
      expect(find.textContaining('Video brief and Shot list'), findsOneWidget);
    });

    testWidgets('a starter shows its notebooks and its tags', (tester) async {
      await open(tester);
      expect(
        find.text(
          'Features \u00B7 Bugs \u00B7 Performance \u00B7 Experiments \u00B7 '
          'Refactoring \u00B7 Testing \u00B7 Chores \u00B7 Release notes '
          '\u00B7 Decisions',
        ),
        findsOneWidget,
      );
      for (final tag in <String>['web', 'server', 'mobile', 'docs']) {
        expect(find.text('#$tag'), findsOneWidget);
      }
      // Product development and Knowledge base both make it.
      expect(find.text('#important'), findsNWidgets(2));
      expect(find.text('#to-read'), findsOneWidget);
    });

    testWidgets('choosing Product development makes that starter', (
      tester,
    ) async {
      await open(tester);
      await tester.ensureVisible(find.text('Product development'));
      await tester.tap(find.text('Product development'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(EditableText), 'Kanso');
      await tester.pump();
      await tester.tap(find.widgetWithText(KansoButton, 'Create'));
      await tester.pumpAndSettle();
      expect(library.notebookKinds, <NewNotebookKind>[
        NewNotebookKind.product,
      ]);
    });

    group('inside a locked notebook', () {
      const lockedHint =
          "Starters can't be made inside a locked notebook. Make it at the "
          'top level, then move it in.';
      NotebookRow parent({required bool locked}) => NotebookRow(
        id: 'book:parent',
        name: 'Parent',
        count: 0,
        locked: locked,
        depth: 0,
        hasChildren: false,
      );

      testWidgets('the starters that make notes are disabled and say why', (
        tester,
      ) async {
        final locked = parent(locked: true);
        library.notebookRows = <NotebookRow>[locked];
        await open(tester, parent: locked);
        // Six starters make notes; Blank and Video studio don't.
        expect(find.text(lockedHint), findsNWidgets(6));
        await tester.ensureVisible(find.text('Product development'));
        await tester.tap(find.text('Product development'), warnIfMissed: false);
        await tester.pumpAndSettle();
        await tester.enterText(find.byType(EditableText), 'Plan');
        await tester.pump();
        await tester.tap(find.widgetWithText(KansoButton, 'Create'));
        await tester.pumpAndSettle();
        // The choice stayed on Blank, so a blank notebook was made inside.
        expect(library.notebooksCreated, <(String, String?)>[
          ('Plan', 'book:parent'),
        ]);
        expect(library.notebookKinds, <NewNotebookKind>[
          NewNotebookKind.blank,
        ]);
      });

      testWidgets('Blank and Video studio stay available', (tester) async {
        final locked = parent(locked: true);
        library.notebookRows = <NotebookRow>[locked];
        await open(tester, parent: locked);
        await tester.ensureVisible(find.text('Video studio'));
        await tester.tap(find.text('Video studio'));
        await tester.pumpAndSettle();
        await tester.enterText(find.byType(EditableText), 'Channel');
        await tester.pump();
        await tester.tap(find.widgetWithText(KansoButton, 'Create'));
        await tester.pumpAndSettle();
        expect(library.notebookKinds, <NewNotebookKind>[
          NewNotebookKind.videoStudio,
        ]);
      });

      testWidgets('an open notebook says nothing of locks', (tester) async {
        final unlocked = parent(locked: false);
        library.notebookRows = <NotebookRow>[unlocked];
        await open(tester, parent: unlocked);
        expect(find.text(lockedHint), findsNothing);
      });
    });

    testWidgets('Create makes a blank notebook by default', (tester) async {
      await open(tester);
      await tester.enterText(find.byType(EditableText), 'Recipes');
      await tester.pump();
      await tester.tap(find.widgetWithText(KansoButton, 'Create'));
      await tester.pumpAndSettle();
      expect(library.notebookKinds, <NewNotebookKind>[NewNotebookKind.blank]);
    });

    testWidgets('choosing Video studio makes a studio', (tester) async {
      await open(tester);
      await tester.ensureVisible(find.text('Video studio'));
      await tester.tap(find.text('Video studio'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(EditableText), 'Channel');
      await tester.pump();
      await tester.tap(find.widgetWithText(KansoButton, 'Create'));
      await tester.pumpAndSettle();
      expect(library.notebooksCreated, <(String, String?)>[('Channel', null)]);
      expect(library.notebookKinds, <NewNotebookKind>[
        NewNotebookKind.videoStudio,
      ]);
    });

    testWidgets('the choice can be changed back before Create', (tester) async {
      await open(tester);
      await tester.ensureVisible(find.text('Video studio'));
      await tester.tap(find.text('Video studio'));
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text('Blank notebook'));
      await tester.tap(find.text('Blank notebook'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(EditableText), 'Back');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      expect(library.notebookKinds, <NewNotebookKind>[NewNotebookKind.blank]);
    });

    testWidgets('the cards say which is chosen to a screen reader', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await open(tester);
      bool selected(String label) =>
          tester
              .getSemantics(find.bySemanticsLabel(label).first)
              .flagsCollection
              .isSelected
              .name ==
          'isTrue';
      expect(selected('Blank notebook'), isTrue);
      expect(selected('Video studio'), isFalse);
      await tester.ensureVisible(find.text('Video studio'));
      await tester.tap(find.text('Video studio'));
      await tester.pumpAndSettle();
      expect(selected('Blank notebook'), isFalse);
      expect(selected('Video studio'), isTrue);
      handle.dispose();
    });

    testWidgets('the return key makes it too', (tester) async {
      await open(tester);
      await tester.enterText(find.byType(EditableText), 'Garden');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      expect(library.notebooksCreated.single.$1, 'Garden');
    });

    testWidgets('a failure is said and the sheet stays', (tester) async {
      library.createNotebookResult = const Left<ApplicationError, MadeNotebook>(
        ApplicationError.storage(message: 'disk'),
      );
      await open(tester);
      await tester.enterText(find.byType(EditableText), 'Recipes');
      await tester.pump();
      await tester.tap(find.widgetWithText(KansoButton, 'Create'));
      await tester.pumpAndSettle();
      expect(
        find.text("The notebook couldn't be made. Try again."),
        findsOneWidget,
      );
      expect(find.text('New notebook'), findsOneWidget);
    });

    testWidgets('Cancel closes it and makes nothing', (tester) async {
      await open(tester);
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.text('New notebook'), findsNothing);
      expect(library.notebooksCreated, isEmpty);
    });
  });
}
