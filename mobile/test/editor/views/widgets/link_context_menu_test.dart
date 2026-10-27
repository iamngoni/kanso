// Kanso — link_context_menu_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/link_context_menu.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import '../../../helpers/fake_editor_document.dart';

Future<(FakeEditorDocument, List<LinkTarget>)> _pump(
  WidgetTester tester, {
  LinkTarget? link,
}) async {
  tester.view
    ..physicalSize = const Size(393, 700)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final document = FakeEditorDocument(text: 'see [[Plan]] now')..link = link;
  final controller = KansoMarkdownController(
    document: document,
    snapshot: document.current,
    themeId: 'kanso-light',
  );
  final focus = FocusNode();
  addTearDown(() {
    focus.dispose();
    controller.dispose();
  });
  final opened = <LinkTarget>[];
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: MarkdownEditor(
          controller: controller,
          focusNode: focus,
          menuBuilder: (context, state) => LinkContextMenu(
            state: state,
            document: document,
            onOpen: opened.add,
          ),
        ),
      ),
    ),
  );
  focus.requestFocus();
  // A long press selects the word and opens the menu there.
  await tester.longPressAt(
    tester.getTopLeft(find.byType(MarkdownEditor)) + const Offset(60, 10),
  );
  await tester.pump();
  await tester.pump();
  return (document, opened);
}

void main() {
  group('LinkContextMenu', () {
    testWidgets('offers Open link in a link, with the usual buttons', (
      tester,
    ) async {
      final (document, opened) = await _pump(
        tester,
        link: const LinkTarget.note(id: 'note:plan'),
      );
      expect(document.calls, contains('linkAt 6'));
      expect(find.text('Open link'), findsOneWidget);
      expect(find.text('Select All'), findsOneWidget);
      await tester.tap(find.text('Open link'));
      await tester.pump();
      expect(opened, <LinkTarget>[const LinkTarget.note(id: 'note:plan')]);
    });

    testWidgets('has no Open link where the caret is in none', (tester) async {
      await _pump(tester);
      expect(find.text('Open link'), findsNothing);
      expect(find.text('Select All'), findsOneWidget);
    });

    testWidgets('a link that leads nowhere yet is opened too', (tester) async {
      final (_, opened) = await _pump(
        tester,
        link: const LinkTarget.newNote(title: 'Plan'),
      );
      await tester.tap(find.text('Open link'));
      expect(opened.single, const LinkTarget.newNote(title: 'Plan'));
    });
  });
}
