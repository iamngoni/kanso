// Kanso — editor_shortcuts_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/editor_shortcuts.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import '../../../helpers/fake_editor_document.dart';

const _completion = Completion(
  kind: CompletionKind.wiki,
  replace: Range16(start: 0, end: 2),
  items: <CompletionItem>[
    CompletionItem(label: 'Plan', insert: '[[Plan]]', newNote: false),
    CompletionItem(label: 'Plot', insert: '[[Plot]]', newNote: false),
    CompletionItem(label: 'Plum', insert: '[[Plum]]', newNote: false),
  ],
);

class _Rig {
  _Rig(this.document, this.controller);

  final FakeEditorDocument document;
  final KansoMarkdownController controller;
  int saves = 0;
  int dismissed = 0;
  int escapes = 0;
}

Future<_Rig> _pump(
  WidgetTester tester, {
  String text = 'text',
  TargetPlatform platform = TargetPlatform.iOS,
}) async {
  tester.view
    ..physicalSize = const Size(393, 600)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
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
  final rig = _Rig(document, controller);
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light.copyWith(platform: platform),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: EditorShortcuts(
          controller: controller,
          platform: platform,
          completionOpen: () => controller.completion != null,
          onSave: () => rig.saves++,
          onDismissCompletion: () => rig.dismissed++,
          onEscape: () => rig.escapes++,
          child: MarkdownEditor(controller: controller, focusNode: focus),
        ),
      ),
    ),
  );
  focus.requestFocus();
  await tester.pump();
  return rig;
}

Future<void> _chord(
  WidgetTester tester,
  LogicalKeyboardKey key, {
  bool command = false,
  bool shift = false,
}) async {
  if (command) {
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
  }
  if (shift) {
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
  }
  await tester.sendKeyEvent(key);
  if (shift) {
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  }
  if (command) {
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
  }
  await tester.pump();
}

void main() {
  group('EditorShortcuts', () {
    testWidgets('Command-B and Command-I ask for bold and italic', (
      tester,
    ) async {
      final rig = await _pump(tester);
      await _chord(tester, LogicalKeyboardKey.keyB, command: true);
      await _chord(tester, LogicalKeyboardKey.keyI, command: true);
      expect(rig.document.commands, <EditCommand>[
        const EditCommand.format(format: FormatKind.bold),
        const EditCommand.format(format: FormatKind.italic),
      ]);
    });

    testWidgets('Control is the key on Android', (tester) async {
      final rig = await _pump(tester, platform: TargetPlatform.android);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyB);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pump();
      expect(rig.document.commands, <EditCommand>[
        const EditCommand.format(format: FormatKind.bold),
      ]);
    });

    testWidgets('Command-S saves now', (tester) async {
      final rig = await _pump(tester);
      await _chord(tester, LogicalKeyboardKey.keyS, command: true);
      expect(rig.saves, 1);
    });

    testWidgets('Tab indents and Shift-Tab outdents', (tester) async {
      final rig = await _pump(tester);
      await _chord(tester, LogicalKeyboardKey.tab);
      await _chord(tester, LogicalKeyboardKey.tab, shift: true);
      expect(rig.document.commands, <EditCommand>[
        const EditCommand.tab(out: false),
        const EditCommand.tab(out: true),
      ]);
    });

    testWidgets('Escape goes to Preview when nothing is offered', (
      tester,
    ) async {
      final rig = await _pump(tester);
      await _chord(tester, LogicalKeyboardKey.escape);
      expect(rig.escapes, 1);
      expect(rig.dismissed, 0);
    });

    testWidgets('Escape closes the completions first', (tester) async {
      final rig = await _pump(tester);
      rig.controller.completion = _completion;
      await _chord(tester, LogicalKeyboardKey.escape);
      expect(rig.dismissed, 1);
      expect(rig.escapes, 0);
    });

    testWidgets('Down and Up move through the completions, within them', (
      tester,
    ) async {
      final rig = await _pump(tester);
      rig.controller.completion = _completion;
      await _chord(tester, LogicalKeyboardKey.arrowDown);
      expect(rig.controller.completionIndex, 1);
      await _chord(tester, LogicalKeyboardKey.arrowDown);
      await _chord(tester, LogicalKeyboardKey.arrowDown);
      expect(rig.controller.completionIndex, 2);
      await _chord(tester, LogicalKeyboardKey.arrowUp);
      await _chord(tester, LogicalKeyboardKey.arrowUp);
      await _chord(tester, LogicalKeyboardKey.arrowUp);
      expect(rig.controller.completionIndex, 0);
    });

    testWidgets('Return chooses the one that is active', (tester) async {
      final rig = await _pump(tester, text: '[[P');
      rig.controller
        ..completion = _completion
        ..completionIndex = 2;
      rig.document.nextOutcome = const EditOutcome(
        version: 4,
        replace: Replacement(start: 0, end: 2, text: '[[Plum]]'),
        selection: Range16(start: 8, end: 8),
        history: HistoryState(canUndo: true, canRedo: false),
      );
      await _chord(tester, LogicalKeyboardKey.enter);
      expect(
        rig.document.commands.single,
        const EditCommand.insert(
          range: Range16(start: 0, end: 2),
          text: '[[Plum]]',
        ),
      );
    });

    testWidgets('with nothing offered, the arrow keys are the field\u2019s', (
      tester,
    ) async {
      final rig = await _pump(tester, text: 'one\ntwo');
      rig.controller.selection = const TextSelection.collapsed(offset: 1);
      await tester.pump();
      await _chord(tester, LogicalKeyboardKey.arrowDown);
      // The caret moved down a line: the field took the key.
      expect(rig.controller.selection.baseOffset, greaterThan(3));
      expect(rig.document.commands, isEmpty);
      expect(rig.controller.completionIndex, 0);
    });
  });
}
