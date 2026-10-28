// Kanso — editor_ime_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';

import '../test/helpers/editor_harness.dart';

/// A spell checker that marks every `teh` as a misspelling of `the`.
class TehSpellCheckService implements SpellCheckService {
  @override
  Future<List<SuggestionSpan>?> fetchSpellCheckSuggestions(
    Locale locale,
    String text,
  ) async => <SuggestionSpan>[
    for (final match in RegExp('teh').allMatches(text))
      SuggestionSpan(
        TextRange(start: match.start, end: match.end),
        const <String>['the'],
      ),
  ];
}

TextEditingValue at(String text, int caret, {TextRange? composing}) =>
    TextEditingValue(
      text: text,
      selection: TextSelection.collapsed(offset: caret),
      composing: composing ?? TextRange.empty,
    );

/// Every leaf span of the text the field draws, with its style merged from
/// the spans above it, and every placeholder span.
({List<(String, TextStyle)> leaves, int placeholders}) drawn(
  WidgetTester tester,
) {
  final state = tester.state<KansoEditableTextState>(
    find.byType(KansoEditableText),
  );
  final leaves = <(String, TextStyle)>[];
  var placeholders = 0;
  void walk(InlineSpan span, TextStyle inherited) {
    if (span is PlaceholderSpan) {
      placeholders++;
      return;
    }
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

  walk(state.buildTextSpan(), const TextStyle());
  return (leaves: leaves, placeholders: placeholders);
}

/// What is underlined as a composition: underlined, but not wavy.
List<String> composed(WidgetTester tester) => <String>[
  for (final (text, style) in drawn(tester).leaves)
    if (style.decoration == TextDecoration.underline &&
        style.decorationStyle != TextDecorationStyle.wavy)
      text,
];

List<String> misspelled(WidgetTester tester) => <String>[
  for (final (text, style) in drawn(tester).leaves)
    if (style.decorationStyle == TextDecorationStyle.wavy) text,
];

/// The input methods of phones and the core, together: what the keyboard
/// does to the field, and that the Rust core's text is the field's after
/// each step.
///
/// Run it with `flutter test integration_test/editor_ime_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  /// Types each step, and checks the field's text, the core's and the
  /// composition's underline after it.
  Future<void> typeSteps(
    WidgetTester tester,
    EditorHarness editor,
    List<TextEditingValue> steps,
  ) async {
    for (final step in steps) {
      await editor.type(tester, step);
      expect(editor.controller.text, step.text);
      expect(editor.controller.isComposing, !step.composing.isCollapsed);
      expect(await editor.rustText(tester), step.text);
      final composing = step.composing;
      expect(
        composed(tester),
        composing.isCollapsed
            ? isEmpty
            : <String>[step.text.substring(composing.start, composing.end)],
      );
    }
  }

  group('input methods', () {
    testWidgets('Japanese: romaji, kana and kanji are never replaced', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'abc');
      await typeSteps(tester, editor, <TextEditingValue>[
        at('abck', 4, composing: const TextRange(start: 3, end: 4)),
        at('abcか', 4, composing: const TextRange(start: 3, end: 4)),
        at('abcかん', 5, composing: const TextRange(start: 3, end: 5)),
        at('abc漢', 4, composing: const TextRange(start: 3, end: 4)),
        at('abc漢字', 5, composing: const TextRange(start: 3, end: 5)),
        at('abc漢字', 5),
      ]);
      // Text from outside waited for the composition; none was open at the
      // end.
      expect(editor.controller.text, 'abc漢字');
    });

    testWidgets('Chinese after an emoji: units are UTF-16', (tester) async {
      final editor = await pumpEditor(tester, text: '🙂 ');
      await typeSteps(tester, editor, <TextEditingValue>[
        at('🙂 n', 4, composing: const TextRange(start: 3, end: 4)),
        at('🙂 ni', 5, composing: const TextRange(start: 3, end: 5)),
        at('🙂 你', 4, composing: const TextRange(start: 3, end: 4)),
        at('🙂 你好', 5, composing: const TextRange(start: 3, end: 5)),
        at('🙂 你好', 5),
      ]);
    });

    testWidgets('Latin words composed as Gboard does', (tester) async {
      final editor = await pumpEditor(tester, text: '');
      await typeSteps(tester, editor, <TextEditingValue>[
        at('hel', 3, composing: const TextRange(start: 0, end: 3)),
        at('hello', 5, composing: const TextRange(start: 0, end: 5)),
        at('hello ', 6),
        at('hello wor', 9, composing: const TextRange(start: 6, end: 9)),
        at('hello world', 11, composing: const TextRange(start: 6, end: 11)),
        at('hello world ', 12),
      ]);
    });

    testWidgets('a composition does not hold back the save', (tester) async {
      final editor = await pumpEditor(tester, text: 'abc');
      await editor.type(
        tester,
        at('abc composing', 13, composing: const TextRange(start: 4, end: 13)),
      );
      expect(editor.controller.isComposing, isTrue);
      // Well past the autosave's delay, still composing.
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 1500)),
      );
      final stored = (await tester.runAsync(
        () => editor.library.note(id: editor.noteId),
      ))!;
      expect(stored.body, 'abc composing');
    });

    testWidgets('Return commits the composition before it continues a list', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: '- one');
      await editor.type(
        tester,
        at('- onex', 6, composing: const TextRange(start: 5, end: 6)),
      );
      expect(editor.controller.isComposing, isTrue);
      // The keyboard commits the word and types the newline in one update.
      await editor.type(tester, at('- onex\n', 7));
      expect(editor.controller.text, '- onex\n- ');
      expect(await editor.rustText(tester), '- onex\n- ');
    });
  });

  group('autocorrect and dictation', () {
    testWidgets('an autocorrected word is one undo step', (tester) async {
      final editor = await pumpEditor(tester, text: '');
      await editor.type(tester, at('teh', 3));
      await editor.type(tester, at('teh ', 4));
      // The keyboard replaces the word it just left.
      await editor.type(tester, at('the ', 4));
      expect(editor.controller.text, 'the ');
      expect(await editor.rustText(tester), 'the ');
      expect(editor.controller.undo(), isTrue);
      expect(editor.controller.text, 'teh ');
      expect(await editor.rustText(tester), 'teh ');
      expect(editor.controller.redo(), isTrue);
      expect(editor.controller.text, 'the ');
    });

    testWidgets('dictation: a placeholder, then a phrase of words', (
      tester,
    ) async {
      final editor = await pumpEditor(tester, text: 'Notes: ');
      await editor.type(tester, at('Notes: ', 7));
      final state = tester.state<KansoEditableTextState>(
        find.byType(KansoEditableText),
      );
      expect(drawn(tester).placeholders, 0);
      state.insertTextPlaceholder(const Size(40, 20));
      await tester.pump();
      expect(drawn(tester).placeholders, greaterThan(0));
      await editor.type(
        tester,
        at('Notes: buy milk and call the plumber', 36),
      );
      state.removeTextPlaceholder();
      await tester.pump();
      expect(drawn(tester).placeholders, 0);
      expect(editor.controller.text, 'Notes: buy milk and call the plumber');
      expect(
        await editor.rustText(tester),
        'Notes: buy milk and call the plumber',
      );
      expect(tester.takeException(), isNull);
    });

    testWidgets('handwriting placeholders show no errors', (tester) async {
      final editor = await pumpEditor(
        tester,
        text: 'teh ',
        spellCheckService: TehSpellCheckService(),
      );
      await editor.settle(tester);
      tester.state<KansoEditableTextState>(find.byType(KansoEditableText))
        ..insertTextPlaceholder(const Size(60, 24))
        ..removeTextPlaceholder();
      await tester.pump();
      await editor.type(tester, at('teh ink', 7));
      await editor.settle(tester);
      expect(tester.takeException(), isNull);
      expect(editor.controller.text, 'teh ink');
      expect(await editor.rustText(tester), 'teh ink');
    });
  });

  group('spell check', () {
    testWidgets('underlines misspellings, keeps the colours, and leaves code '
        'and addresses alone', (tester) async {
      final editor = await pumpEditor(
        tester,
        text: '# Teh heading',
        spellCheckService: TehSpellCheckService(),
      );
      await editor.settle(tester);
      const typed = '# Heading\n\nteh `teh` <https://teh.example> teh';
      await editor.type(tester, at(typed, typed.length));
      await editor.settle(tester);
      await editor.settle(tester);
      expect(misspelled(tester), <String>['teh', 'teh']);
      final heading = drawn(tester).leaves.firstWhere(
        (leaf) => leaf.$1.contains('Heading'),
      );
      expect(heading.$2.fontWeight, FontWeight.w700);
      expect(editor.controller.runs, isNotEmpty);
    });
  });
}
