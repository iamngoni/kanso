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
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';

void main() {
  late FixtureLibraryRepository library;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(() => unawaited(library.close()));
    registerPageBlocs(library);
  });

  Future<GoRouter> open(WidgetTester tester) async {
    final router = await pumpRouted(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () => unawaited(NewNotebookSheet.open(context)),
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

    testWidgets('offers a blank notebook and a Video studio, blank first', (
      tester,
    ) async {
      await open(tester);
      expect(find.text('Blank notebook'), findsOneWidget);
      expect(find.text('Video studio'), findsOneWidget);
      expect(
        find.textContaining('Idea \u2192 Scripting \u2192 Shooting'),
        findsOneWidget,
      );
      expect(find.textContaining('Video brief and Shot list'), findsOneWidget);
      expect(
        tester.getTopLeft(find.text('Blank notebook')).dy,
        lessThan(tester.getTopLeft(find.text('Video studio')).dy),
      );
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
      await tester.tap(find.text('Video studio'));
      await tester.pumpAndSettle();
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
      library.createNotebookResult = const Left<ApplicationError, NotebookRow>(
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
