// Kanso — editor_spans_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        Generator,
        GeneratorUtils,
        Glados3,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/utils/diff_values.dart';
import 'package:kanso/editor/utils/editor_spans.dart';

import '../../helpers/property.dart';

const TextStyle _base = TextStyle(fontSize: 15);

EditorRun _run(int start, int end, {int? color, bool quiet = false}) =>
    EditorRun(
      range: Range16(start: start, end: end),
      color: color,
      quiet: quiet,
    );

/// The text of every span, in order.
List<String> _pieces(TextSpan span) => span.children == null
    ? <String>[?span.text]
    : <String>[for (final child in span.children!) (child as TextSpan).text!];

extension _Inputs on Any {
  Generator<String> get written => listWithLengthInRange<String>(
    0,
    14,
    choose(<String>['a', 'b', ' ', '\u{1F600}', '\n']),
  ).map((chars) => chars.join());

  /// Runs in order (one of five colours, quiet or not), some of which reach
  /// past the text or land inside a surrogate pair.
  Generator<List<EditorRun>> get someRuns =>
      listWithLengthInRange<int>(0, 8, intInRange(0, 6)).map((gaps) {
        final runs = <EditorRun>[];
        var at = 0;
        for (var i = 0; i + 1 < gaps.length; i += 2) {
          final start = at + gaps[i];
          final end = start + gaps[i + 1] + 1;
          runs.add(_run(start, end, color: i, quiet: i % 4 == 2));
          at = end;
        }
        return runs;
      });

  Generator<List<TextRange>> get someRanges =>
      listWithLengthInRange<int>(0, 6, intInRange(0, 22)).map((points) {
        final ranges = <TextRange>[];
        for (var i = 0; i + 1 < points.length; i += 2) {
          final a = points[i];
          final b = points[i + 1];
          ranges.add(TextRange(start: a < b ? a : b, end: a < b ? b : a));
        }
        return ranges;
      });
}

void main() {
  group('editorSpans', () {
    test('is the plain text in the base style when nothing is styled', () {
      final span = editorSpans(text: 'hello', runs: const [], base: _base);
      expect(span.toPlainText(), 'hello');
      expect(span.style, _base);
    });

    test('styles each run and leaves the text between plain', () {
      final span = editorSpans(
        text: '# Title now',
        runs: <EditorRun>[_run(0, 7, color: 0xFF0000)],
        base: _base,
      );
      expect(_pieces(span), <String>['# Title', ' now']);
      final styled = span.children![0] as TextSpan;
      expect(styled.style?.color, const Color(0xFFFF0000));
      expect((span.children![1] as TextSpan).style, isNull);
    });

    test('underlines the composing range', () {
      final span = editorSpans(
        text: 'abcdef',
        runs: const [],
        base: _base,
        composing: const TextRange(start: 2, end: 4),
      );
      expect(_pieces(span), <String>['ab', 'cd', 'ef']);
      expect(
        (span.children![1] as TextSpan).style?.decoration,
        TextDecoration.underline,
      );
      expect((span.children![0] as TextSpan).style, isNull);
    });

    test('marks misspellings, but not in quiet runs', () {
      final span = editorSpans(
        text: 'teh `teh` teh',
        runs: <EditorRun>[_run(4, 9, quiet: true)],
        base: _base,
        misspellings: const <TextRange>[
          TextRange(start: 0, end: 3),
          TextRange(start: 5, end: 8),
          TextRange(start: 10, end: 13),
        ],
      );
      final marked = <String>[
        for (final child in span.children!)
          if ((child as TextSpan).style?.decorationStyle ==
              TextDecorationStyle.wavy)
            child.text!,
      ];
      expect(marked, <String>['teh', 'teh']);
    });

    test('a composing range wins over a misspelling', () {
      final span = editorSpans(
        text: 'teh',
        runs: const [],
        base: _base,
        composing: const TextRange(start: 0, end: 3),
        misspellings: const <TextRange>[TextRange(start: 0, end: 3)],
      );
      expect(
        (span.children!.single as TextSpan).style?.decorationStyle,
        isNot(TextDecorationStyle.wavy),
      );
    });

    test('tints the words comments are on, in code as well', () {
      const tint = TextStyle(backgroundColor: Color(0x33AA8800));
      final span = editorSpans(
        text: 'see `this` here',
        runs: <EditorRun>[_run(4, 10, color: 0x00FF00, quiet: true)],
        base: _base,
        comments: const <TextRange>[
          TextRange(start: 0, end: 3),
          TextRange(start: 5, end: 9),
        ],
        commentStyle: tint,
      );
      final tinted = <String>[
        for (final child in span.children!)
          if ((child as TextSpan).style?.backgroundColor ==
              tint.backgroundColor)
            child.text!,
      ];
      expect(tinted, <String>['see', 'this']);
      // The colour of the run it is in is kept.
      final code = span.children!.cast<TextSpan>().firstWhere(
        (child) => child.text == 'this',
      );
      expect(code.style?.color, const Color(0xFF00FF00));
    });

    test('washes what others selected in their colour, over a comment', () {
      const ana = Color(0x382F6AAE);
      const bo = Color(0x382D7A4D);
      const tint = Color(0x33AA8800);
      final span = editorSpans(
        text: 'one two three',
        runs: const <EditorRun>[],
        base: _base,
        comments: const <TextRange>[TextRange(start: 4, end: 13)],
        commentStyle: const TextStyle(backgroundColor: tint),
        selections: const <(TextRange, Color)>[
          (TextRange(start: 0, end: 3), ana),
          (TextRange(start: 4, end: 7), bo),
        ],
      );
      final washes = <(String, Color?)>[
        for (final child in span.children!.cast<TextSpan>())
          (child.text!, child.style?.backgroundColor),
      ];
      expect(washes, <(String, Color?)>[
        ('one', ana),
        (' ', null),
        ('two', bo),
        (' three', tint),
      ]);
    });

    Glados3<String, List<EditorRun>, List<TextRange>>(
      any.written,
      any.someRuns,
      any.someRanges,
      kansoExploreConfig(),
    ).test('a character is tinted exactly when a comment is on it', (
      text,
      runs,
      ranges,
    ) {
      const tint = Color(0x33AA8800);
      final span = editorSpans(
        text: text,
        runs: runs,
        base: _base,
        comments: ranges,
        commentStyle: const TextStyle(backgroundColor: tint),
      );
      // Where a range lands inside a surrogate pair, it starts or ends
      // before it.
      int snap(int at) {
        final clamped = at.clamp(0, text.length);
        return splitsPair(text, clamped) ? clamped - 1 : clamped;
      }

      var at = 0;
      for (final child in span.children ?? <InlineSpan>[span]) {
        final piece = (child as TextSpan).text!;
        for (var unit = at; unit < at + piece.length; unit++) {
          final commented = ranges.any(
            (range) => snap(range.start) <= unit && unit < snap(range.end),
          );
          expect(
            child.style?.backgroundColor == tint,
            commented,
            reason: 'unit $unit of "$text"',
          );
        }
        at += piece.length;
      }
    });

    Glados3<String, List<EditorRun>, List<TextRange>>(
      any.written,
      any.someRuns,
      any.someRanges,
      kansoExploreConfig(),
    ).test('the spans are the text, cut only between characters', (
      text,
      runs,
      ranges,
    ) {
      final span = editorSpans(
        text: text,
        runs: runs,
        base: _base,
        composing: ranges.isEmpty ? TextRange.empty : ranges.first,
        misspellings: ranges,
      );
      expect(span.toPlainText(), text);
      var at = 0;
      for (final piece in _pieces(span)) {
        at += piece.length;
        expect(splitsPair(text, at), isFalse);
      }
      expect(at, text.length);
    });
  });
}
