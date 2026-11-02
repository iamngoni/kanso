// Kanso — triage_suite.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/triage_types.dart';

import 'open_library.dart';

/// Inbox triage through the bridge: every call of `KansoTriage` and
/// `KansoLibrary.triage`.
void triageSuite(OpenLibrary open) {
  group('KansoTriage', () {
    test('the settings start as specified and are kept', () async {
      final library = await open();
      final triage = library.triage();
      final first = await triage.settings();
      expect(first.showBar, isTrue);
      expect(first.suggestTitles, isTrue);
      expect(first.suggestLinks, isTrue);
      expect(first.maxTags, 3);
      expect(first.useAi, isFalse);

      final saved = await triage.setSettings(
        input: const TriageSettings(
          showBar: false,
          suggestTitles: false,
          suggestLinks: false,
          maxTags: 2,
          useAi: false,
        ),
      );
      expect(saved.showBar, isFalse);
      expect(saved.maxTags, 2);
      expect((await triage.settings()).maxTags, 2);
    });

    test('a typed tag becomes a new chip, an empty one is refused', () async {
      final library = await open();
      final triage = library.triage();
      final chip = await triage.resolve(
        field: TriageField.tag,
        text: 'groceries',
      );
      expect(chip.field, TriageField.tag);
      expect(chip.isNew, isTrue);
      expect(chip.ai, isFalse);
      await expectLater(
        triage.resolve(field: TriageField.tag, text: '  '),
        throwsA(anything),
      );
    });

    test('cards, plans, filing and Undo', () async {
      final library = await open();
      final triage = library.triage();
      final id = await makeNote(library, 'Untitled', 'Buy milk and eggs.');

      final status = await triage.status();
      expect(status.aiOn, isFalse);
      expect(status.waiting, greaterThanOrEqualTo(0));

      final cards = await triage.cards(limit: 50);
      expect(cards.length, lessThanOrEqualTo(50));
      final card = await triage.card(noteId: id);
      expect(card == null || card.noteId == id, isTrue);

      // A rejection is remembered, and lifted.
      await triage.reject(noteId: id, field: TriageField.tag, value: 'milk');
      await triage.unreject(noteId: id, field: TriageField.tag, value: 'milk');

      final chip = await triage.resolve(
        field: TriageField.tag,
        text: 'groceries',
      );
      final view = await library.note(id: id);
      final filings = <TriageFiling>[
        TriageFiling(noteId: id, rev: view.rev, accepted: <TriageChip>[chip]),
      ];
      final plan = await triage.plan(filings: filings);
      expect(plan.undo, 0);
      expect((await library.note(id: id)).tags, isEmpty);

      final filed = await triage.file(filings: filings);
      expect(filed.filed, 1);
      expect(filed.tags, 1);
      expect(filed.undo, greaterThan(0));
      expect((await library.note(id: id)).tags, isNotEmpty);

      await library.undo(ticket: filed.undo);
      expect((await library.note(id: id)).tags, isEmpty);
    });

    test('asking the model without one set up says so', () async {
      final library = await open();
      final triage = library.triage();
      final id = await makeNote(library, 'Untitled', 'Buy milk.');
      await expectLater(triage.refine(noteId: id), throwsA(anything));
    });
  });
}
