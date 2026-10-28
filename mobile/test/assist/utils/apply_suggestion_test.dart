// Kanso — apply_suggestion_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/assist/models/data/assist_placement.dart';
import 'package:kanso/assist/models/data/assist_snapshot.dart';
import 'package:kanso/assist/utils/apply_suggestion.dart';

import '../../helpers/fake_editor_target.dart';

AssistSnapshot _snapshot(FakeEditorTarget target) => AssistSnapshot(
  noteId: target.noteId,
  revision: target.revision,
  windowOrigin: target.windowOrigin,
  text: target.value.text,
  selection: TextRange(
    start: target.value.selection.start,
    end: target.value.selection.end,
  ),
);

void main() {
  group('applySuggestion', () {
    test('replaces the selection in one edit', () {
      final target = FakeEditorTarget(
        'Hello. we ship on friday. Bye.',
        selection: const TextSelection(baseOffset: 7, extentOffset: 24),
      );
      final snapshot = _snapshot(target);
      expect(
        applySuggestion(
          target,
          snapshot,
          AssistPlacement.replace,
          'We ship on Friday.',
        ),
        isTrue,
      );
      expect(target.value.text, 'Hello. We ship on Friday.. Bye.');
      expect(target.edits, 1);
    });

    test('inserts below the selection, as a new paragraph', () {
      final target = FakeEditorTarget(
        'One.\nTwo.',
        selection: const TextSelection(baseOffset: 0, extentOffset: 4),
      );
      applySuggestion(
        target,
        _snapshot(target),
        AssistPlacement.insertBelow,
        'Summary.',
      );
      expect(target.value.text, 'One.\n\nSummary.\nTwo.');
    });

    test('inserts at the caret after a blank line without adding more', () {
      final target = FakeEditorTarget('One.\n\n');
      applySuggestion(
        target,
        _snapshot(target),
        AssistPlacement.insertBelow,
        'Next.',
      );
      expect(target.value.text, 'One.\n\nNext.');
    });

    test('changes nothing if the selected text changed meanwhile', () {
      final target = FakeEditorTarget(
        'We ship on friday.',
        selection: const TextSelection(baseOffset: 0, extentOffset: 18),
      );
      final snapshot = _snapshot(target);
      target.type('We ship on Monday.');
      expect(
        applySuggestion(target, snapshot, AssistPlacement.replace, 'x'),
        isFalse,
      );
      expect(target.value.text, 'We ship on Monday.');
      expect(target.edits, 0);
    });

    test('changes nothing if the text before a caret changed', () {
      final target = FakeEditorTarget('Earlier words.');
      final snapshot = _snapshot(target);
      target.type('Other words.');
      expect(
        applySuggestion(target, snapshot, AssistPlacement.insertBelow, 'x'),
        isFalse,
      );
    });

    test('changes nothing if the text got shorter than the selection', () {
      final target = FakeEditorTarget(
        'Long enough text.',
        selection: const TextSelection(baseOffset: 5, extentOffset: 17),
      );
      final snapshot = _snapshot(target);
      target.type('Long');
      expect(
        applySuggestion(target, snapshot, AssistPlacement.replace, 'x'),
        isFalse,
      );
    });

    test('changes nothing after any edit even when selected text survives', () {
      final target = FakeEditorTarget(
        'Selected text. Later words.',
        selection: const TextSelection(baseOffset: 0, extentOffset: 14),
      );
      final snapshot = _snapshot(target);
      target.type('Selected text. Changed later words.');
      expect(
        applySuggestion(target, snapshot, AssistPlacement.replace, 'x'),
        isFalse,
      );
      expect(target.edits, 0);
    });

    test('changes nothing after target release', () {
      final target = FakeEditorTarget('Selected text.');
      final snapshot = _snapshot(target);
      target.active = false;
      expect(
        applySuggestion(target, snapshot, AssistPlacement.replace, 'x'),
        isFalse,
      );
      expect(target.edits, 0);
    });
  });
}
