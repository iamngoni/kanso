// Kanso — cards_suite.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/cards_types.dart';
import 'package:kanso/core/bridge/api/draft_types.dart';

import 'open_library.dart';

/// Flashcards and drafting through the bridge: every call of `KansoCards`
/// and `KansoDrafting`. AI is off in a test library, so each AI stream ends
/// in one failure that says why.
void cardsSuite(OpenLibrary open) {
  const all = CardScopeArg(kind: CardScopeKind.all, id: '');

  group('KansoCards', () {
    test('a card line is found, answered, suspended and counted', () async {
      final library = await open();
      final cards = library.cards();
      final id = await makeNote(
        library,
        'Capitals',
        'France :: Paris\n\nJapan :: Tokyo\n',
      );

      final lines = await cards.noteCards(noteId: id);
      expect(lines.map((c) => c.question), <String>['France', 'Japan']);

      final due = await cards.due(scope: all, limit: 10, utcOffsetMinutes: 0);
      expect(due.length, 2);
      expect(due.first.intervals.length, 4);

      final counts = await cards.counts(scope: all, utcOffsetMinutes: 0);
      expect(counts.total, 2);

      final answered = await cards.answer(
        noteId: id,
        cardKey: due.first.cardKey,
        grade: 3,
        utcOffsetMinutes: 0,
      );
      expect(answered.dueMs, greaterThan(0));

      await cards.suspend(
        noteId: id,
        cardKey: due.last.cardKey,
        suspended: true,
      );
      final after = await cards.counts(scope: all, utcOffsetMinutes: 0);
      expect(after.suspended, 1);

      final stats = await cards.stats(scope: all, utcOffsetMinutes: 0);
      expect(stats.counts.total, 2);

      expect(
        await cards.reminderText(utcOffsetMinutes: 0),
        anyOf(isNull, isA<String>()),
      );
    });

    test('cards are added under Flashcards, settings are kept', () async {
      final library = await open();
      final cards = library.cards();
      final id = await makeNote(library, 'Notes', 'Some text.');
      final added = await cards.addCards(
        noteId: id,
        cards: const <CardPair>[CardPair(question: 'Why?', answer: 'Because')],
      );
      expect(added, 1);
      expect((await cards.noteCards(noteId: id)).single.question, 'Why?');

      final first = await cards.settings();
      final saved = await cards.setSettings(
        input: CardSettingsView(
          newPerDay: 5,
          reviewsPerDay: first.reviewsPerDay,
          retention: 0.9,
          showIcons: false,
        ),
      );
      expect(saved.newPerDay, 5);
      expect((await cards.settings()).showIcons, isFalse);

      expect(await cards.resetHistory(), greaterThanOrEqualTo(0));
    });

    test('AI streams end in one failure while AI is off', () async {
      final library = await open();
      final cards = library.cards();
      final id = await makeNote(library, 'Plain', 'A :: B');
      expect(
        await cards.makeBlocked(noteId: id),
        anyOf(isNull, isA<String>()),
      );
      final run = library.ai().newRun();

      final suggested = await cards.suggest(noteId: id, run: run).toList();
      expect(suggested.last, isA<SuggestEvent_Failed>());
      final quiz = await cards.quiz(noteIds: <String>[id], run: run).toList();
      expect(quiz.last, isA<QuizEvent_Failed>());
      final graded = await cards
          .grade(question: 'A', noteId: id, answer: 'B', run: run)
          .toList();
      expect(graded.last, isA<GradeEvent_Failed>());
    });
  });

  group('KansoDrafting', () {
    test('sources, previews and saving a draft', () async {
      final library = await open();
      final drafting = library.drafting();
      final id = await makeNote(library, 'Source', 'Some facts.');

      final sources = await drafting.sources(noteIds: <String>[id]);
      expect(sources.notes.length, 1);

      final preview = await drafting.preview(
        text: 'A claim [1].',
        sources: <DraftSourceRef>[
          DraftSourceRef(number: 1, id: id, title: 'Source'),
        ],
      );
      expect(preview.markdown, contains('Sources'));

      final saved = await drafting.save(
        title: 'Draft',
        markdown: preview.markdown,
        bookId: preview.defaultBookId,
        sourceCount: 1,
      );
      expect(saved, startsWith('note:'));

      expect(
        await drafting.notesOfNotebook(bookId: preview.defaultBookId),
        isNotNull,
      );
      expect(await drafting.notesOfTag(tagId: 'tag:none'), isEmpty);
    });

    test('a draft that cannot start is one failure', () async {
      final library = await open();
      final id = await makeNote(library, 'Source', 'Some facts.');
      final events = await library
          .drafting()
          .write(
            noteIds: <String>[id],
            shape: DraftShape.report,
            request: '',
            length: DraftLength.short,
            run: library.ai().newRun(),
          )
          .toList();
      expect(events.last, isA<DraftEvent_Failed>());
    });
  });
}
