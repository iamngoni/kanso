// Kanso — assist_editor_extras_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/assist/utils/assist_editor_extras.dart';
import 'package:kanso/core/configs/kanso_icons.dart';

import '../../helpers/fake_editor_target.dart';
import '../../helpers/fixture_ai_repository.dart';

AssistEditorExtras _extras(
  FixtureAiRepository ai, {
  bool Function()? enabled,
}) => AssistEditorExtras(
  ai: ai,
  isEnabled: enabled ?? () => true,
  selectionLabel: 'Ask AI…',
  keyLabel: 'AI assistant',
);

void main() {
  group('AssistEditorExtras', () {
    test('adds a sparkles key and, on a selection, an Ask AI item', () {
      final ai = FixtureAiRepository();
      addTearDown(ai.close);
      final extras = _extras(ai);
      final selected = FakeEditorTarget(
        'We ship on friday.',
        selection: const TextSelection(baseOffset: 0, extentOffset: 7),
      );
      final key = extras.accessoryKeys(selected).single;
      expect(key.id, 'assist');
      expect(key.icon, KansoIcons.sparkles);
      expect(key.semanticLabel, 'AI assistant');
      final item = extras.selectionActions(selected).single;
      expect(item.id, 'assist');
      expect(item.label, 'Ask AI…');
    });

    test('offers no menu item with nothing selected, but keeps the key', () {
      final ai = FixtureAiRepository();
      addTearDown(ai.close);
      final extras = _extras(ai);
      final caret = FakeEditorTarget('We ship on friday.');
      expect(extras.selectionActions(caret), isEmpty);
      expect(extras.accessoryKeys(caret), hasLength(1));
    });

    test('offers nothing while AI is off', () {
      final ai = FixtureAiRepository();
      addTearDown(ai.close);
      final extras = _extras(ai, enabled: () => false);
      final target = FakeEditorTarget(
        'Text',
        selection: const TextSelection(baseOffset: 0, extentOffset: 2),
      );
      expect(extras.accessoryKeys(target), isEmpty);
      expect(extras.selectionActions(target), isEmpty);
    });

    test('offers nothing for a note the core keeps out of AI', () async {
      final ai = FixtureAiRepository(keptOut: true);
      addTearDown(ai.close);
      final extras = _extras(ai);
      final target = FakeEditorTarget('Text');
      // Until the core has answered, the note is taken to be eligible.
      expect(extras.accessoryKeys(target), hasLength(1));
      await Future<void>.delayed(Duration.zero);
      expect(extras.accessoryKeys(target), isEmpty);
    });
  });
}
