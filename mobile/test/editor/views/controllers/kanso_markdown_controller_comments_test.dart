// Kanso — kanso_markdown_controller_comments_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/models/data/outside_side.dart';

import '../../../helpers/editor_harness.dart';

/// Lines of 16 units and a break, so that 60 of them are 1,019 long.
final String longNote = List<String>.generate(
  60,
  (i) => 'line ${i.toString().padLeft(2, '0')} of words',
).join('\n');

const Color tint = Color(0x33AA8800);

/// The text the field draws tinted.
List<String> tinted(TextSpan span) => <String>[
  for (final child in span.children ?? <InlineSpan>[span])
    if ((child as TextSpan).style?.backgroundColor == tint) child.text!,
];

void main() {
  group('KansoMarkdownController and comments', () {
    testWidgets('selects a range in the window', (tester) async {
      final editor = await pumpEditor(tester, text: 'one two three');
      final controller = editor.controller;
      expect(controller.selectRange(4, 7), isTrue);
      expect(
        controller.selection,
        const TextSelection(baseOffset: 4, extentOffset: 7),
      );
      expect(controller.noteSelection.start, 4);
      expect(controller.noteSelection.end, 7);
    });

    testWidgets('selecting a range far off moves the window to it', (
      tester,
    ) async {
      final editor = await pumpEditor(
        tester,
        text: longNote,
        windowUnits: 300,
        windowMargin: 60,
      );
      final controller = editor.controller;
      final start = longNote.indexOf('line 55');
      expect(controller.fieldOffsetOf(start), isNull);
      expect(controller.selectRange(start, start + 7), isTrue);
      await tester.pump();
      expect(controller.noteSelection.start, start);
      expect(controller.noteSelection.end, start + 7);
      final from = controller.fieldOffsetOf(start)!;
      expect(controller.text.substring(from, from + 7), 'line 55');
    });

    testWidgets('tints the commented words, and they follow the typing '
        'before them', (tester) async {
      final editor = await pumpEditor(tester, text: 'one two three');
      final controller = editor.controller
        ..commentStyle = const TextStyle(backgroundColor: tint)
        ..commentMarks = const <TextRange>[TextRange(start: 4, end: 7)];
      TextSpan span() =>
          controller.spanFor(style: const TextStyle(), withComposing: false);
      expect(tinted(span()), <String>['two']);
      await editor.type(
        tester,
        const TextEditingValue(
          text: 'zero one two three',
          selection: TextSelection.collapsed(offset: 5),
        ),
      );
      expect(controller.commentMarks, const <TextRange>[
        TextRange(start: 9, end: 12),
      ]);
      expect(tinted(span()), <String>['two']);
    });

    testWidgets('the text outside the window is tinted where a comment is', (
      tester,
    ) async {
      final editor = await pumpEditor(
        tester,
        text: longNote,
        windowUnits: 300,
        windowMargin: 60,
      );
      final controller = editor.controller;
      final start = longNote.indexOf('line 50');
      controller.commentMarks = <TextRange>[
        TextRange(start: start, end: start + 7),
      ];
      // The side after the window starts after its first line break.
      final origin = controller.windowStart + controller.text.length + 1;
      final line = start - origin;
      expect(
        controller.outsideCommentsOf(OutsideSide.after, line, line + 16),
        const <TextRange>[TextRange(start: 0, end: 7)],
      );
      expect(
        controller.outsideCommentsOf(OutsideSide.after, 0, 16),
        isEmpty,
      );
    });

    testWidgets('others’ selections are washed in their colour, and their '
        'carets follow the typing before them', (tester) async {
      final editor = await pumpEditor(tester, text: 'one two three');
      final controller = editor.controller
        ..darkCursors = false
        ..remoteCursors = const <CursorMark>[
          CursorMark(
            name: 'ana',
            caret: 7,
            start: 4,
            end: 7,
            light: 0x2F6AAE,
            dark: 0x7FB0EC,
          ),
        ];
      TextSpan span() =>
          controller.spanFor(style: const TextStyle(), withComposing: false);
      final washed = span().children!.cast<TextSpan>().firstWhere(
        (child) => child.style?.backgroundColor != null,
      );
      expect(washed.text, 'two');
      expect(
        washed.style?.backgroundColor,
        const Color(0xFF2F6AAE).withValues(alpha: 0.22),
      );
      // On a dark page, the dark colour.
      controller.darkCursors = true;
      expect(
        controller.cursorColor(controller.remoteCursors.single),
        const Color(0xFF7FB0EC),
      );
      await editor.type(
        tester,
        const TextEditingValue(
          text: 'zero one two three',
          selection: TextSelection.collapsed(offset: 5),
        ),
      );
      final moved = controller.remoteCursors.single;
      expect((moved.start, moved.end, moved.caret), (9, 12, 12));
    });
  });
}
