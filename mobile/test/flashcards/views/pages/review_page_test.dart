// Kanso — review_page_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/cards.dart';
import 'package:kanso/core/bridge/api/cards_types.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/flashcards/utils/flashcards_text.dart';
import 'package:kanso/flashcards/views/pages/review_page.dart';
import 'package:kanso/injection.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/kanso_test_app.dart';

class _MockLibrary extends Mock implements KansoLibrary {}

class _MockCards extends Mock implements KansoCards {}

ReviewCard _card(String question, {bool isNew = true}) => ReviewCard(
  cardKey: 'k-$question',
  noteId: 'note:a',
  noteTitle: 'Deck',
  line: 0,
  question: question,
  answer: 'The answer to $question',
  isNew: isNew,
  intervals: const <String>['10 min', '1 day', '4 days', '2 weeks'],
);

void main() {
  late _MockCards cards;

  setUpAll(() {
    registerFallbackValue(FlashcardsText.everything);
  });

  setUp(() {
    cards = _MockCards();
    final library = _MockLibrary();
    when(library.cards).thenReturn(cards);
    getIt.registerSingleton<KansoLibrary>(library);
  });

  tearDown(() async {
    await getIt.reset();
  });

  void serve(List<ReviewCard> due) {
    when(
      () => cards.due(
        scope: any(named: 'scope'),
        limit: any(named: 'limit'),
        utcOffsetMinutes: any(named: 'utcOffsetMinutes'),
      ),
    ).thenAnswer((_) async => due);
    when(
      () => cards.answer(
        noteId: any(named: 'noteId'),
        cardKey: any(named: 'cardKey'),
        grade: any(named: 'grade'),
        utcOffsetMinutes: any(named: 'utcOffsetMinutes'),
      ),
    ).thenAnswer((_) async => const Answered(dueMs: 1, waiting: 0));
  }

  testWidgets('a tap shows the answer, then Good records a 3', (tester) async {
    serve(<ReviewCard>[_card('Q one'), _card('Q two')]);
    await tester.pumpWidget(const KansoTestApp(home: FlashcardReviewPage()));
    await tester.pumpAndSettle();

    expect(find.text('Q one'), findsOneWidget);
    expect(find.text('1 of 2 · 2 new'), findsOneWidget);
    expect(find.text('The answer to Q one'), findsNothing);

    await tester.tap(find.text(FlashcardsText.showAnswer));
    await tester.pumpAndSettle();
    expect(find.text('The answer to Q one'), findsOneWidget);
    expect(find.text('4 days'), findsOneWidget);

    await tester.tap(find.text(FlashcardsText.good));
    await tester.pumpAndSettle();
    verify(
      () => cards.answer(
        noteId: 'note:a',
        cardKey: 'k-Q one',
        grade: 3,
        utcOffsetMinutes: any(named: 'utcOffsetMinutes'),
      ),
    ).called(1);
    expect(find.text('Q two'), findsOneWidget);
    expect(find.text('The answer to Q two'), findsNothing);
  });

  testWidgets('Again brings the card back, and the end says so', (
    tester,
  ) async {
    serve(<ReviewCard>[_card('Only')]);
    await tester.pumpWidget(const KansoTestApp(home: FlashcardReviewPage()));
    await tester.pumpAndSettle();

    await tester.tap(find.text(FlashcardsText.showAnswer));
    await tester.pumpAndSettle();
    await tester.tap(find.text(FlashcardsText.again));
    await tester.pumpAndSettle();
    expect(find.text('Only'), findsOneWidget, reason: 'asked again');

    await tester.tap(find.text(FlashcardsText.showAnswer));
    await tester.pumpAndSettle();
    await tester.tap(find.text(FlashcardsText.easy));
    await tester.pumpAndSettle();
    expect(find.text(FlashcardsText.doneForToday), findsOneWidget);
    expect(find.text(FlashcardsText.reviewMore), findsOneWidget);
  });

  testWidgets('with no cards it explains how to write one', (tester) async {
    serve(const <ReviewCard>[]);
    await tester.pumpWidget(const KansoTestApp(home: FlashcardReviewPage()));
    await tester.pumpAndSettle();
    expect(find.text(FlashcardsText.noCardsTitle), findsOneWidget);
  });
}
