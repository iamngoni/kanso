// Kanso — kanso_markdown_controller_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import '../../../helpers/editor_harness.dart';

TextEditingValue _at(String text, int caret, {TextRange? composing}) {
  return TextEditingValue(
    text: text,
    selection: TextSelection.collapsed(offset: caret),
    composing: composing ?? TextRange.empty,
  );
}

void main() {
  group('KansoMarkdownController', () {
    testWidgets('keeps what is typed in Rust, character by character', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'hello');
      await editor.type(tester, _at('hello!', 6));
      await editor.type(tester, _at('hello! 🙂', 9));
      await editor.type(tester, _at('hello 🙂', 7));
      expect(editor.controller.text, 'hello 🙂');
      expect(await editor.rustText(tester), 'hello 🙂');
      // A paste over a selection is one change too.
      await editor.type(tester, _at('he', 2));
      expect(await editor.rustText(tester), 'he');
    });

    testWidgets('Return continues a list, and Shift-Return does not', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: '- one');
      editor.controller.selection = const TextSelection.collapsed(offset: 5);
      await tester.pump();
      await editor.type(tester, _at('- one\n', 6));
      expect(editor.controller.text, '- one\n- ');
      expect(editor.controller.selection.baseOffset, 8);
      expect(await editor.rustText(tester), '- one\n- ');

      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await editor.type(tester, _at('- one\n- \n', 9));
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      expect(editor.controller.text, '- one\n- \n');
      expect(await editor.rustText(tester), '- one\n- \n');
    });

    testWidgets('Return right after a composed word still continues the list', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: '- one');
      await editor.type(
        tester,
        _at('- onex', 6, composing: const TextRange(start: 5, end: 6)),
      );
      expect(editor.controller.isComposing, isTrue);
      // The keyboard commits the word and types the newline in one update.
      await editor.type(tester, _at('- onex\n', 7));
      expect(editor.controller.text, '- onex\n- ');
      expect(await editor.rustText(tester), '- onex\n- ');
    });

    testWidgets('Return on an empty list item ends the list', (tester) async {
      final editor = await pumpEditor(tester, text: '- one\n- ');
      editor.controller.selection = const TextSelection.collapsed(offset: 8);
      await tester.pump();
      await editor.type(tester, _at('- one\n- \n', 9));
      expect(editor.controller.text, isNot(contains('- \n')));
      expect(await editor.rustText(tester), editor.controller.text);
    });

    testWidgets('Tab moves to the next cell of a table', (tester) async {
      const table = '| a | b |\n| - | - |\n| c | d |';
      final editor = await pumpEditor(tester, text: table);
      editor.controller.selection = const TextSelection.collapsed(offset: 3);
      await tester.pump();
      expect(editor.controller.run(const EditCommand.tab(out: false)), isTrue);
      expect(
        editor.controller.selection.baseOffset,
        greaterThan(3),
        reason: 'the caret moved on',
      );
      expect(await editor.rustText(tester), editor.controller.text);
    });

    testWidgets('Tab indents with two spaces outside lists and tables', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'plain');
      editor.controller.selection = const TextSelection.collapsed(offset: 0);
      await tester.pump();
      expect(editor.controller.run(const EditCommand.tab(out: false)), isTrue);
      expect(editor.controller.text, '  plain');
    });

    testWidgets('undo and redo walk the steps back and forward', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'a');
      await editor.type(tester, _at('ab', 2));
      await editor.type(tester, _at('abc', 3));
      expect(editor.controller.history.canUndo, isTrue);
      // Typed on in a run, it is one step.
      expect(editor.controller.undo(), isTrue);
      expect(editor.controller.text, 'a');
      expect(editor.controller.history.canRedo, isTrue);
      expect(await editor.rustText(tester), 'a');
      expect(editor.controller.redo(), isTrue);
      expect(editor.controller.text, 'abc');
      expect(editor.controller.undo(), isTrue);
      expect(editor.controller.undo(), isFalse);
    });

    testWidgets('a format changes the text and the caret in one turn', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'make bold');
      editor.controller.selection = const TextSelection(
        baseOffset: 5,
        extentOffset: 9,
      );
      await tester.pump();
      expect(
        editor.controller.run(
          const EditCommand.format(format: FormatKind.bold),
        ),
        isTrue,
      );
      expect(editor.controller.text, 'make **bold**');
      expect(await editor.rustText(tester), 'make **bold**');
    });

    testWidgets('never replaces the text under a composition', (tester) async {
      final editor = await pumpEditor(tester, text: 'abc');
      await editor.type(
        tester,
        _at('abcni', 5, composing: const TextRange(start: 3, end: 5)),
      );
      expect(editor.controller.isComposing, isTrue);
      final outside = (await tester.runAsync(editor.editor.snapshot))!;
      editor.controller.reload(
        EditorSnapshot(
          id: outside.id,
          title: outside.title,
          text: 'from elsewhere',
          rev: outside.rev,
          lock: outside.lock,
          properties: outside.properties,
          version: outside.version,
          selection: outside.selection,
          history: outside.history,
          save: outside.save,
          wordGoal: outside.wordGoal,
        ),
      );
      await tester.pump();
      expect(editor.controller.text, 'abcni');
      // The composition ends: now it is shown.
      await editor.type(tester, _at('abc你', 4));
      expect(editor.controller.isComposing, isFalse);
      expect(editor.controller.text, 'from elsewhere');
    });

    testWidgets('colours the text with the runs the core sends', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: '# Heading\n\nplain text');
      await editor.settle(tester);
      expect(editor.controller.runs, isNotEmpty);
      final span = editor.controller.spanFor(
        style: const TextStyle(),
        withComposing: false,
      );
      final first = span.children!.first as TextSpan;
      expect(first.text, startsWith('#'));
      expect(first.style?.color, isNotNull);
      expect(span.toPlainText(), '# Heading\n\nplain text');
    });

    testWidgets(
      'moves the colours along with a keystroke until fresh ones come',
      (
        tester,
      ) async {
        final editor = await pumpEditor(tester, text: '# Head\n\nplain');
        await editor.settle(tester);
        final before = editor.controller.runs.length;
        await editor.type(tester, _at('# Heads\n\nplain', 7));
        // Not yet asked for: the old runs, moved.
        final moved = editor.controller.runs;
        expect(moved, isNotEmpty);
        expect(moved.length, before);
        expect(moved.map((run) => run.range.end), contains(7));
        await editor.settle(tester);
        expect(editor.controller.runs, isNotEmpty);
      },
    );

    testWidgets('a lone surrogate half is never handed to the core', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: '😀');
      await editor.type(tester, _at('😁', 2));
      expect(editor.controller.text, '😁');
      expect(await editor.rustText(tester), '😁');
    });
  });
}
