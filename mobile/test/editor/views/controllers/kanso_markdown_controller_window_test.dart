// Kanso — kanso_markdown_controller_window_test.dart
// Created 2026-10-26.
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

TextEditingValue at(String text, int caret) => TextEditingValue(
  text: text,
  selection: TextSelection.collapsed(offset: caret),
);

/// A window of 300 units with 60 of margin: the note is four windows long.
Future<EditorHarness> pumpLong(WidgetTester tester, {int? caret}) {
  return pumpEditor(
    tester,
    text: longNote,
    caret: caret,
    windowUnits: 300,
    windowMargin: 60,
  );
}

void main() {
  group('KansoMarkdownController with a window', () {
    testWidgets('a short note is one window', (tester) async {
      final editor = await pumpEditor(tester, text: 'a short note');
      expect(editor.controller.isWindowed, isFalse);
      expect(editor.controller.windowStart, 0);
      expect(editor.controller.before, isEmpty);
      expect(editor.controller.after, isEmpty);
    });

    testWidgets('a long note opens as a window around the caret', (
      tester,
    ) async {
      final editor = await pumpLong(tester, caret: 600);
      final controller = editor.controller;
      expect(controller.isWindowed, isTrue);
      expect(controller.text.length, lessThanOrEqualTo(300));
      expect(controller.windowStart, greaterThan(0));
      expect(controller.noteText, longNote);
      expect(controller.noteLength, longNote.length);
      expect(controller.noteSelection.baseOffset, 600);
      expect(controller.selection.baseOffset, 600 - controller.windowStart);
      // The window is whole lines; the text outside joins it with a break.
      expect(controller.before, endsWith('\n'));
      expect(controller.after, startsWith('\n'));
      expect(await editor.rustText(tester), longNote);
    });

    testWidgets('a note at the start has nothing before the window', (
      tester,
    ) async {
      final editor = await pumpLong(tester);
      expect(editor.controller.windowStart, 0);
      expect(editor.controller.before, isEmpty);
      expect(editor.controller.after, isNotEmpty);
    });

    testWidgets('typing is told to the core in the units of the note', (
      tester,
    ) async {
      final editor = await pumpLong(tester, caret: 600);
      final controller = editor.controller;
      final caret = controller.selection.baseOffset;
      await editor.type(
        tester,
        at(controller.text.replaceRange(caret, caret, 'XY'), caret + 2),
      );
      final expected = longNote.replaceRange(600, 600, 'XY');
      expect(controller.noteText, expected);
      expect(await editor.rustText(tester), expected);
      expect(controller.noteSelection.baseOffset, 602);
    });

    testWidgets('the window moves as the caret nears its edge', (
      tester,
    ) async {
      final editor = await pumpLong(tester, caret: 100);
      final controller = editor.controller;
      final revision = controller.windowRevision;
      // Walk the caret down the note, a line at a time.
      for (var at = 100; at < longNote.length; at += 20) {
        controller.placeCaret(at);
        await tester.pump();
        expect(controller.noteSelection.baseOffset, at);
        expect(controller.fieldOffsetOf(at), isNotNull);
        expect(controller.noteText, longNote);
        expect(controller.text.length, lessThanOrEqualTo(300));
      }
      expect(controller.windowRevision, greaterThan(revision));
      expect(controller.windowStart, greaterThan(500));
      expect(await editor.rustText(tester), longNote);
    });

    testWidgets(
      'typing on keeps the caret in the window and the core in step',
      (
        tester,
      ) async {
        final editor = await pumpLong(tester, caret: 100);
        final controller = editor.controller;
        var expected = longNote;
        var caret = 100;
        for (var i = 0; i < 400; i++) {
          final local = controller.selection.baseOffset;
          await editor.type(
            tester,
            at(controller.text.replaceRange(local, local, 'x'), local + 1),
          );
          expected = expected.replaceRange(caret, caret, 'x');
          caret++;
          expect(controller.noteSelection.baseOffset, caret);
          expect(controller.text.length, lessThanOrEqualTo(2 * 300));
        }
        expect(controller.noteText, expected);
        expect(await editor.rustText(tester), expected);
      },
    );

    testWidgets('an undo of text out of the window moves the window to it', (
      tester,
    ) async {
      final editor = await pumpLong(tester, caret: 5);
      final controller = editor.controller;
      await editor.type(tester, at(controller.text.replaceRange(5, 5, 'A'), 6));
      controller.placeCaret(longNote.length);
      await tester.pump();
      expect(controller.windowStart, greaterThan(500));
      final local = controller.selection.baseOffset;
      await editor.type(
        tester,
        at(controller.text.replaceRange(local, local, 'Z'), local + 1),
      );
      // The first undo takes the Z, near the caret.
      expect(controller.undo(), isTrue);
      expect(controller.noteText, longNote.replaceRange(5, 5, 'A'));
      // The second takes the A, at the start of the note.
      expect(controller.undo(), isTrue);
      expect(controller.noteText, longNote);
      expect(controller.windowStart, 0);
      expect(await editor.rustText(tester), longNote);
      // And redo goes back there.
      expect(controller.redo(), isTrue);
      expect(controller.noteText, longNote.replaceRange(5, 5, 'A'));
      expect(await editor.rustText(tester), longNote.replaceRange(5, 5, 'A'));
    });

    testWidgets('Select All makes the window the whole note, and a tap cuts '
        'it back', (tester) async {
      final editor = await pumpLong(tester, caret: 600);
      final controller = editor.controller;
      expect(controller.selectEverything(), isTrue);
      expect(controller.isWindowed, isFalse);
      expect(controller.text, longNote);
      expect(controller.selection.start, 0);
      expect(controller.selection.end, longNote.length);
      expect(controller.noteSelection.end, longNote.length);
      // A caret in it: the window is cut back to the cap.
      controller.selection = const TextSelection.collapsed(offset: 700);
      await tester.pump();
      expect(controller.isWindowed, isTrue);
      expect(controller.text.length, lessThanOrEqualTo(300));
      expect(controller.noteSelection.baseOffset, 700);
      expect(controller.noteText, longNote);
    });

    testWidgets('a text from outside keeps the caret, in units of the note', (
      tester,
    ) async {
      final editor = await pumpLong(tester, caret: 600);
      final controller = editor.controller;
      final outside = (await tester.runAsync(editor.editor.snapshot))!;
      final changed = 'new first line\n$longNote';
      controller.reload(
        EditorSnapshot(
          id: outside.id,
          title: outside.title,
          text: changed,
          rev: outside.rev,
          lock: outside.lock,
          properties: outside.properties,
          version: outside.version,
          history: outside.history,
          save: outside.save,
        ),
      );
      await tester.pump();
      expect(controller.noteText, changed);
      expect(controller.noteSelection.baseOffset, 600);
    });

    testWidgets('the window does not move under a composition', (tester) async {
      final editor = await pumpLong(tester, caret: 100);
      final controller = editor.controller;
      final revision = controller.windowRevision;
      final text = controller.text;
      final near = text.length - 10;
      await editor.type(
        tester,
        TextEditingValue(
          text: text,
          selection: TextSelection.collapsed(offset: near),
          composing: TextRange(start: near - 2, end: near),
        ),
      );
      expect(controller.isComposing, isTrue);
      expect(controller.windowRevision, revision);
      expect(controller.placeCaret(1000), isFalse);
      // It ends: the window moves.
      await editor.type(tester, at(text, near));
      expect(controller.windowRevision, greaterThan(revision));
      expect(controller.fieldOffsetOf(100 + near), isNotNull);
    });

    testWidgets('asks for the colours of the text outside the window', (
      tester,
    ) async {
      final editor = await pumpEditor(
        tester,
        text: List<String>.generate(
          60,
          (i) => i.isEven ? '# Heading $i' : 'plain line number $i',
        ).join('\n'),
        windowUnits: 300,
        windowMargin: 60,
      );
      final controller = editor.controller;
      var changes = 0;
      controller.outsideChanged.addListener(() => changes++);
      final side = controller.after.substring(1);
      final heading = side.indexOf('# Heading');
      expect(
        controller.outsideRunsOf(OutsideSide.after, heading, heading + 9),
        isEmpty,
      );
      await editor.settle(tester);
      await editor.settle(tester);
      expect(changes, greaterThan(0));
      final runs = controller.outsideRunsOf(
        OutsideSide.after,
        heading,
        heading + 9,
      );
      expect(runs, isNotEmpty);
      // In units of the side's text, clipped to what was asked for.
      expect(runs.first.range.start, greaterThanOrEqualTo(heading));
      expect(runs.last.range.end, lessThanOrEqualTo(heading + 9));
    });
  });
}
