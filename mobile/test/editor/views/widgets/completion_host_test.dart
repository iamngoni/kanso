// Kanso — completion_host_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/completion_host.dart';
import 'package:kanso/editor/views/widgets/completion_popup.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';

import '../../../helpers/fake_editor_document.dart';

Completion _completion(int rows) => Completion(
  kind: CompletionKind.wiki,
  replace: const Range16(start: 0, end: 3),
  items: <CompletionItem>[
    for (var i = 0; i < rows; i++)
      CompletionItem(
        label: 'Note $i',
        highlight: const Range16(start: 0, end: 2),
        insert: '[[Note $i]]',
        newNote: false,
      ),
  ],
);

class _Rig {
  _Rig(this.document, this.controller, this.focus);

  final FakeEditorDocument document;
  final KansoMarkdownController controller;
  final FocusNode focus;
}

Future<_Rig> _pump(
  WidgetTester tester, {
  required String text,
  Size size = const Size(393, 700),
  double above = 0,
  double bottomInset = 0,
}) async {
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1
    ..viewInsets = FakeViewPadding(bottom: bottomInset);
  addTearDown(() {
    tester.view.reset();
    tester.view.resetViewInsets();
  });
  final document = FakeEditorDocument(text: text);
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
  final key = GlobalKey<EditableTextState>();
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        resizeToAvoidBottomInset: false,
        body: SingleChildScrollView(
          child: Column(
            children: <Widget>[
              SizedBox(height: above),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: CompletionHost(
                  controller: controller,
                  editableKey: key,
                  reservedBottom: 45,
                  child: MarkdownEditor(
                    controller: controller,
                    focusNode: focus,
                    editableKey: key,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    ),
  );
  focus.requestFocus();
  await tester.pump();
  return _Rig(document, controller, focus);
}

Rect _popup(WidgetTester tester) =>
    tester.getRect(find.byType(CompletionPopup));

Future<void> _offer(WidgetTester tester, _Rig rig, Completion? offered) async {
  rig.controller.offer(offered);
  await tester.pump();
  await tester.pump();
}

void main() {
  group('CompletionHost', () {
    testWidgets('shows nothing until there is something to complete', (
      tester,
    ) async {
      await _pump(tester, text: '[[No');
      expect(find.byType(CompletionPopup), findsNothing);
    });

    testWidgets('shows the completions below the caret, and takes them back', (
      tester,
    ) async {
      final rig = await _pump(tester, text: 'Some text [[No');
      rig.controller.selection = const TextSelection.collapsed(offset: 14);
      await tester.pump();
      await _offer(tester, rig, _completion(3));
      expect(find.byType(CompletionPopup), findsOneWidget);
      final caret = tester
          .state<EditableTextState>(find.byType(KansoEditableText))
          .renderEditable
          .getLocalRectForCaret(const TextPosition(offset: 14));
      final editor = tester.getTopLeft(find.byType(MarkdownEditor));
      final popup = _popup(tester);
      expect(popup.top, closeTo(editor.dy + caret.bottom + 4, 1));
      expect(popup.height, CompletionPopup.heightFor(3));
      await _offer(tester, rig, null);
      expect(find.byType(CompletionPopup), findsNothing);
    });

    testWidgets('is as wide as the editor on a phone', (tester) async {
      final rig = await _pump(tester, text: '[[No');
      rig.controller.selection = const TextSelection.collapsed(offset: 4);
      await tester.pump();
      await _offer(tester, rig, _completion(2));
      expect(_popup(tester).width, 393 - 32);
      expect(_popup(tester).left, 16);
    });

    testWidgets('is no wider than 400 points on a tablet, at the caret', (
      tester,
    ) async {
      final rig = await _pump(
        tester,
        text: 'a longer line before [[No',
        size: const Size(1000, 700),
      );
      rig.controller.selection = const TextSelection.collapsed(offset: 25);
      await tester.pump();
      await _offer(tester, rig, _completion(2));
      expect(_popup(tester).width, CompletionHost.maxWidth);
      expect(_popup(tester).left, greaterThan(16));
    });

    testWidgets('goes above the caret where there is no room below', (
      tester,
    ) async {
      // The caret is near the foot of a screen with a keyboard up.
      final rig = await _pump(
        tester,
        text: '[[No',
        size: const Size(393, 640),
        above: 330,
        bottomInset: 260,
      );
      rig.controller.selection = const TextSelection.collapsed(offset: 4);
      await tester.pump();
      await _offer(tester, rig, _completion(4));
      final caret = tester
          .state<EditableTextState>(find.byType(KansoEditableText))
          .renderEditable;
      final caretTop = caret
          .localToGlobal(
            caret.getLocalRectForCaret(const TextPosition(offset: 4)).topLeft,
          )
          .dy;
      final popup = _popup(tester);
      expect(popup.bottom, lessThanOrEqualTo(caretTop));
      expect(popup.bottom, closeTo(caretTop - 4, 1));
    });

    testWidgets('does not take the focus, and a tap on a row chooses it', (
      tester,
    ) async {
      final rig = await _pump(tester, text: '[[No');
      rig.controller.selection = const TextSelection.collapsed(offset: 4);
      await tester.pump();
      await _offer(tester, rig, _completion(3));
      expect(rig.focus.hasFocus, isTrue);
      rig.document.nextOutcome = const EditOutcome(
        version: 3,
        replace: Replacement(start: 0, end: 3, text: '[[Note 1]]'),
        selection: Range16(start: 10, end: 10),
        history: HistoryState(canUndo: true, canRedo: false),
      );
      await tester.tap(
        find.byWidgetPredicate(
          (widget) => widget is HighlightedText && widget.text == 'Note 1',
        ),
      );
      await tester.pump();
      expect(rig.focus.hasFocus, isTrue);
      expect(
        rig.document.commands.single,
        const EditCommand.insert(
          range: Range16(start: 0, end: 3),
          text: '[[Note 1]]',
        ),
      );
      await tester.pump();
      expect(find.byType(CompletionPopup), findsNothing);
    });

    testWidgets('follows the active row as the keys move it', (tester) async {
      final rig = await _pump(tester, text: '[[No');
      rig.controller.selection = const TextSelection.collapsed(offset: 4);
      await tester.pump();
      await _offer(tester, rig, _completion(3));
      rig.controller.stepCompletion(1);
      await tester.pump();
      expect(
        tester
            .widget<CompletionPopup>(find.byType(CompletionPopup))
            .activeIndex,
        1,
      );
    });
  });
}
