// Kanso — markdown_editor_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/kanso_undo_manager_client.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';

import '../../../helpers/editor_harness.dart';

/// A spell checker that marks every `teh` as a misspelling of `the`.
class TehSpellCheckService implements SpellCheckService {
  @override
  Future<List<SuggestionSpan>?> fetchSpellCheckSuggestions(
    Locale locale,
    String text,
  ) async {
    return <SuggestionSpan>[
      for (final match in RegExp('teh').allMatches(text))
        SuggestionSpan(
          TextRange(start: match.start, end: match.end),
          const <String>['the'],
        ),
    ];
  }
}

/// Every leaf span of the text the field draws, with its style merged from
/// the spans above it.
List<(String, TextStyle)> _drawn(WidgetTester tester) {
  final state = tester.state<KansoEditableTextState>(
    find.byType(KansoEditableText),
  );
  final leaves = <(String, TextStyle)>[];
  void walk(InlineSpan span, TextStyle inherited) {
    if (span is! TextSpan) {
      return;
    }
    final style = span.style == null ? inherited : inherited.merge(span.style);
    final text = span.text;
    if (text != null && text.isNotEmpty) {
      leaves.add((text, style));
    }
    for (final child in span.children ?? const <InlineSpan>[]) {
      walk(child, style);
    }
  }

  walk(state.renderEditable.text!, const TextStyle());
  return leaves;
}

TextEditingValue _at(String text, int caret) => TextEditingValue(
  text: text,
  selection: TextSelection.collapsed(offset: caret),
);

void main() {
  group('MarkdownEditor', () {
    testWidgets('draws the text in the colours of its runs', (tester) async {
      final editor = await pumpEditor(
        tester,
        text: '# Heading\n\nplain `code` text',
      );
      await editor.settle(tester);
      final drawn = _drawn(tester);
      expect(
        drawn.map((leaf) => leaf.$1).join(),
        '# Heading\n\nplain `code` text',
      );
      final heading = drawn.firstWhere((leaf) => leaf.$1.contains('Heading'));
      final plain = drawn.firstWhere((leaf) => leaf.$1.startsWith('\n\nplain'));
      expect(heading.$2.fontWeight, FontWeight.w700);
      expect(plain.$2.fontWeight, FontWeight.w400);
      final code = drawn.firstWhere((leaf) => leaf.$1 == '`code`');
      expect(code.$2.color, isNot(plain.$2.color));
    });

    testWidgets('spell check marks a word and the colours stay', (
      tester,
    ) async {
      final editor = await pumpEditor(
        tester,
        text: '# Title',
        spellCheckService: TehSpellCheckService(),
      );
      await editor.settle(tester);
      await editor.type(tester, _at('# Title\n\nteh `teh` end', 21));
      await editor.settle(tester);
      await editor.settle(tester);
      final drawn = _drawn(tester);
      expect(drawn.map((leaf) => leaf.$1).join(), '# Title\n\nteh `teh` end');
      final marked = drawn
          .where((leaf) => leaf.$2.decorationStyle == TextDecorationStyle.wavy)
          .map((leaf) => leaf.$1);
      // Both words are misspelled; only the one outside the code is marked.
      expect(marked, <String>['teh']);
      // The heading is still coloured.
      final title = drawn.firstWhere((leaf) => leaf.$1.contains('Title'));
      expect(title.$2.fontWeight, FontWeight.w700);
    });

    testWidgets('a misspelling is moved along as the text changes, and a '
        'word being typed is not marked', (tester) async {
      final editor = await pumpEditor(
        tester,
        text: 'teh',
        spellCheckService: TehSpellCheckService(),
      );
      await editor.type(tester, _at('teh ', 4));
      await editor.settle(tester);
      await editor.settle(tester);
      expect(
        _drawn(tester).where(
          (leaf) => leaf.$2.decorationStyle == TextDecorationStyle.wavy,
        ),
        hasLength(1),
      );
      // Text typed before it: the mark moves; the new word is not marked.
      await editor.type(tester, _at('x teh ', 1));
      final marked = _drawn(tester)
          .where((leaf) => leaf.$2.decorationStyle == TextDecorationStyle.wavy)
          .map((leaf) => leaf.$1);
      expect(marked, <String>['teh']);
    });

    testWidgets('takes the system undo for the Rust history', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'a');
      // The place is taken after the frame the field gets focus in.
      await tester.pump();
      expect(UndoManager.client, isA<KansoUndoManagerClient>());
      await editor.type(tester, _at('ab', 2));
      await editor.type(tester, _at('abc', 3));
      UndoManager.client!.handlePlatformUndo(UndoDirection.undo);
      await tester.pump();
      expect(editor.controller.text, 'a');
      expect(await editor.rustText(tester), 'a');
      UndoManager.client!.handlePlatformUndo(UndoDirection.redo);
      await tester.pump();
      expect(editor.controller.text, 'abc');
    });

    testWidgets(
      'the shortcuts for undo and redo reach the Rust history',
      (
        tester,
      ) async {
        final editor = await pumpEditor(tester, text: 'a');
        await editor.type(tester, _at('ab', 2));
        await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
        await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
        await tester.pump();
        expect(editor.controller.text, 'a');
        await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
        await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
        await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
        await tester.pump();
        expect(editor.controller.text, 'ab');
      },
      variant: TargetPlatformVariant.only(TargetPlatform.iOS),
    );

    testWidgets('asks the keyboard for the settings Markdown needs', (
      tester,
    ) async {
      await pumpEditor(tester, text: '');
      final field = tester.widget<EditableText>(
        find.byType(KansoEditableText),
      );
      expect(field.keyboardType, TextInputType.multiline);
      expect(field.textInputAction, TextInputAction.newline);
      expect(field.smartQuotesType, SmartQuotesType.disabled);
      expect(field.smartDashesType, SmartDashesType.disabled);
      expect(field.autocorrect, isTrue);
      expect(field.enableSuggestions, isTrue);
      expect(field.stylusHandwritingEnabled, isTrue);
      expect(field.textCapitalization, TextCapitalization.sentences);
      expect(field.rendererIgnoresPointer, isTrue);
    });

    testWidgets('a tap puts the caret in the text', (tester) async {
      final editor = await pumpEditor(tester, text: 'hello world');
      await tester.tapAt(
        tester.getTopLeft(find.byType(MarkdownEditor)) + const Offset(2, 10),
      );
      await tester.pump(const Duration(milliseconds: 400));
      expect(editor.controller.selection.isCollapsed, isTrue);
      expect(editor.controller.selection.baseOffset, 0);
      expect(editor.focusNode.hasFocus, isTrue);
    });

    testWidgets('a double tap selects a word, with handles', (tester) async {
      final editor = await pumpEditor(tester, text: 'hello world');
      final at =
          tester.getTopLeft(find.byType(MarkdownEditor)) + const Offset(20, 10);
      await tester.tapAt(at);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tapAt(at);
      await tester.pump(const Duration(milliseconds: 400));
      expect(editor.controller.selection.isCollapsed, isFalse);
      final selected = editor.controller.selection.textInside(
        editor.controller.text,
      );
      expect(selected, 'hello');
    });
  });
}
