// Kanso — learn_sheets_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/ai.dart';
import 'package:kanso/core/bridge/api/cards.dart';
import 'package:kanso/core/bridge/api/cards_types.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet.dart';
import 'package:kanso/flashcards/views/widgets/make_flashcards_sheet.dart';
import 'package:kanso/flashcards/views/widgets/quiz_sheet.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';
import 'package:kanso/injection.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/kanso_test_app.dart';

class _MockLibrary extends Mock implements KansoLibrary {}

class _MockCards extends Mock implements KansoCards {}

class _MockAi extends Mock implements KansoAi {}

class _MockRun extends Mock implements KansoAiRun {}

final AppLocalizationsEn l10n = AppLocalizationsEn();

/// How tall the sheet's panel is (the sheet widget itself fills the screen).
double _panelHeight(WidgetTester tester) => tester
    .getSize(
      find
          .descendant(
            of: find.byType(KansoBottomSheet),
            matching: find.byType(AnimatedContainer),
          )
          .first,
    )
    .height;

Future<void> _open(
  WidgetTester tester,
  Future<void> Function(BuildContext context) open,
) async {
  await tester.pumpWidget(
    KansoTestApp(
      home: Builder(
        builder: (context) => Center(
          child: GestureDetector(
            onTap: () => open(context),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open'));
  await tester.pumpAndSettle();
}

void main() {
  late _MockCards cards;
  late _MockRun run;

  setUpAll(() {
    registerFallbackValue(_MockRun());
  });

  setUp(() {
    cards = _MockCards();
    run = _MockRun();
    final library = _MockLibrary();
    when(library.cards).thenReturn(cards);
    final ai = _MockAi();
    when(ai.newRun).thenReturn(run);
    getIt
      ..registerSingleton<KansoLibrary>(library)
      ..registerSingleton<KansoAi>(ai);
  });

  tearDown(() async {
    await getIt.reset();
  });

  group('Make Flashcards sheet', () {
    void suggest(List<SuggestedCard> found) {
      when(
        () => cards.suggest(
          noteId: any(named: 'noteId'),
          run: any(named: 'run'),
        ),
      ).thenAnswer(
        (_) => Stream<SuggestEvent>.value(SuggestEvent.done(cards: found)),
      );
    }

    const found = <SuggestedCard>[
      SuggestedCard(
        question: 'Capital of France?',
        answer: 'Paris',
        line: 0,
        sourceMissing: false,
      ),
      SuggestedCard(
        question: 'What do cats do?',
        answer: 'Sleep',
        sourceMissing: true,
      ),
    ];

    testWidgets('opens as one tall sheet with a handle and the cards', (
      tester,
    ) async {
      suggest(found);
      await _open(tester, (c) => showMakeFlashcards(c, 'note:a'));

      expect(find.byKey(KansoBottomSheet.handleKey), findsOneWidget);
      // Not a sheet inside a sheet.
      expect(find.byType(KansoBottomSheet), findsOneWidget);
      expect(find.text('Capital of France?'), findsOneWidget);
      expect(find.text(l10n.flashcardsSourceNotFound), findsOneWidget);
      // A card whose source is missing starts unticked.
      expect(find.text(l10n.flashcardsAddCards(1)), findsOneWidget);

      final screen = tester.view.physicalSize / tester.view.devicePixelRatio;
      final height = _panelHeight(tester);
      expect(height, closeTo(screen.height * 0.78, 1));
    });

    testWidgets('a drag up on the handle expands it, a drag down settles it', (
      tester,
    ) async {
      suggest(found);
      await _open(tester, (c) => showMakeFlashcards(c, 'note:a'));
      final resting = _panelHeight(tester);

      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, -120),
      );
      await tester.pumpAndSettle();
      final expanded = _panelHeight(tester);
      expect(expanded, greaterThan(resting));

      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, 150),
      );
      await tester.pumpAndSettle();
      expect(find.text('Capital of France?'), findsOneWidget);
      expect(
        _panelHeight(tester),
        closeTo(resting, 1),
      );

      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, 150),
      );
      await tester.pumpAndSettle();
      expect(find.text('Capital of France?'), findsNothing);
    });

    testWidgets('adds the ticked cards, with an edit, as it did', (
      tester,
    ) async {
      suggest(found);
      when(
        () => cards.addCards(
          noteId: any(named: 'noteId'),
          cards: any(named: 'cards'),
        ),
      ).thenAnswer((_) async => 1);
      await _open(tester, (c) => showMakeFlashcards(c, 'note:a'));

      await tester.tap(find.bySemanticsLabel('Edit card').first);
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(EditableText).first, 'Capital?');
      await tester.tap(find.text(l10n.noteListDone));
      await tester.pumpAndSettle();
      expect(find.text('Capital?'), findsOneWidget);

      await tester.tap(find.text(l10n.flashcardsAddCards(1)));
      await tester.pumpAndSettle();

      final sent =
          verify(
                () => cards.addCards(
                  noteId: 'note:a',
                  cards: captureAny(named: 'cards'),
                ),
              ).captured.single
              as List<CardPair>;
      expect(sent, hasLength(1));
      expect(sent.single.question, 'Capital?');
      expect(sent.single.answer, 'Paris');
      expect(find.text('Capital?'), findsNothing, reason: 'the sheet closed');
    });
  });

  group('Quiz Me sheet', () {
    const items = <QuizItem>[
      QuizItem(question: 'Why Paris?', noteId: 'note:a', noteTitle: 'Notes'),
    ];

    testWidgets('asks a question in a sheet and grades the answer', (
      tester,
    ) async {
      when(
        () => cards.quiz(
          noteIds: any(named: 'noteIds'),
          run: any(named: 'run'),
        ),
      ).thenAnswer(
        (_) => Stream<QuizEvent>.value(const QuizEvent.questions(items: items)),
      );
      when(
        () => cards.grade(
          question: any(named: 'question'),
          noteId: any(named: 'noteId'),
          answer: any(named: 'answer'),
          run: any(named: 'run'),
        ),
      ).thenAnswer(
        (_) => Stream<GradeEvent>.value(
          const GradeEvent.graded(
            verdict: QuizVerdict.right,
            label: 'Correct',
            comment: 'It is the capital.',
          ),
        ),
      );
      await _open(tester, (c) => showQuizMe(c, <String>['note:a']));

      expect(find.byKey(KansoBottomSheet.handleKey), findsOneWidget);
      expect(find.text('Why Paris?'), findsOneWidget);
      expect(find.text(l10n.flashcardsQuizProgress(1, 1)), findsOneWidget);

      await tester.enterText(find.byType(EditableText).first, 'Capital');
      await tester.pump();
      await tester.tap(find.text(l10n.flashcardsQuizCheck));
      await tester.pumpAndSettle();
      expect(find.text('It is the capital.'), findsOneWidget);

      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, 200),
      );
      await tester.pumpAndSettle();
      expect(find.text('Why Paris?'), findsNothing);
    });

    testWidgets('a failed model is told inside the sheet', (tester) async {
      when(
        () => cards.quiz(
          noteIds: any(named: 'noteIds'),
          run: any(named: 'run'),
        ),
      ).thenAnswer(
        (_) => Stream<QuizEvent>.value(const QuizEvent.failed(message: 'nope')),
      );
      await _open(tester, (c) => showQuizMe(c, <String>['note:a']));

      expect(find.byKey(KansoBottomSheet.handleKey), findsOneWidget);
      expect(find.text(l10n.settingsClose), findsOneWidget);
      expect(find.text('Why Paris?'), findsNothing);
    });
  });
}
