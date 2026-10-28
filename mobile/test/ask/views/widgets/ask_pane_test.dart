// Kanso — ask_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/ask/state/ask/ask_bloc.dart';
import 'package:kanso/ask/views/widgets/ask_pane.dart';
import 'package:kanso/ask/views/widgets/citation_chip.dart';
import 'package:kanso/core/models/data/ai_note_ref.dart';
import 'package:kanso/core/models/data/ask_update.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../../helpers/ai_blocs.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_routed.dart';

const _pricing = AiNoteRef(id: 'n1', title: 'Pricing');
const _plan = AiNoteRef(id: 'n:2');

Future<FixtureAiRepository> _pump(
  WidgetTester tester, {
  bool enabled = true,
  bool inSheet = false,
}) async {
  final ai = FixtureAiRepository(settings: aiSettings(enabled: enabled));
  addTearDown(ai.close);
  final bloc = AskBloc(ai)..add(const AskEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
  router = await pumpRouted(
    tester,
    BlocProvider<AskBloc>.value(
      value: bloc,
      child: AskPane(inSheet: inSheet),
    ),
  );
  return ai;
}

late GoRouter router;

Future<void> _ask(WidgetTester tester, String question) async {
  await tester.enterText(find.byType(EditableText), question);
  await tester.pump();
  await tester.tap(find.bySemanticsLabel('Ask'));
  await tester.pumpAndSettle();
}

void main() {
  group('AskPane', () {
    testWidgets('an empty conversation says what Ask does and where it goes', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Ask your notes'), findsWidgets);
      expect(
        find.textContaining('answered from your own notes'),
        findsOneWidget,
      );
      expect(find.text(fixturePrivacy), findsOneWidget);
    });

    testWidgets('the privacy line goes once a question is asked', (
      tester,
    ) async {
      await _pump(tester);
      await _ask(tester, 'What does it cost?');
      expect(find.text(fixturePrivacy), findsNothing);
      expect(find.text('What does it cost?'), findsOneWidget);
      expect(find.text('Searching your notes…'), findsOneWidget);
    });

    testWidgets(
      'with AI off there is nothing to ask, and a way to turn it on',
      (
        tester,
      ) async {
        await _pump(tester, enabled: false);
        expect(find.text('AI is off'), findsOneWidget);
        expect(find.byType(EditableText), findsNothing);
        await tester.tap(find.text('Open AI settings'));
        await tester.pumpAndSettle();
        expect(topOf(router), '/settings/ai');
      },
    );

    testWidgets('shows the answer as Markdown with citations that open notes', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'What does it cost?');
      ai.askUpdates
        ..add(const AskUpdate.thinking())
        ..add(
          const AskUpdate.done(
            answer:
                '## Cost\n\nIt costs **40** a month, see '
                '[Pricing](kanso://open/note/n1).\n\n- one\n- two',
            used: <AiNoteRef>[_pricing, _plan],
          ),
        );
      await tester.pumpAndSettle();
      expect(
        find.textContaining('It costs', findRichText: true),
        findsOneWidget,
      );
      expect(find.text('Cost'), findsOneWidget);
      expect(find.text('•'), findsNWidgets(2));
      await tester.tap(find.widgetWithText(CitationChip, 'Pricing'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/note/n1');
    });

    testWidgets('lists the notes used, opened by a tap', (tester) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(
        const AskUpdate.done(answer: 'A.', used: <AiNoteRef>[_pricing, _plan]),
      );
      await tester.pumpAndSettle();
      expect(find.text('Used 2 notes'), findsOneWidget);
      expect(find.text('Pricing'), findsNothing);
      await tester.tap(find.text('Used 2 notes'));
      await tester.pumpAndSettle();
      expect(find.text('Pricing'), findsOneWidget);
      expect(find.text('Untitled'), findsOneWidget);
      await tester.tap(find.text('Untitled'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/note/n%3A2');
    });

    testWidgets('says when nothing matches and when the answer was cut off', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'one');
      ai.askUpdates.add(const AskUpdate.nothingMatches());
      await tester.pumpAndSettle();
      expect(
        find.text('Nothing in your notes matches this question.'),
        findsOneWidget,
      );
      await _ask(tester, 'two');
      ai.askUpdates
        ..add(const AskUpdate.answer(text: 'Half'))
        ..add(const AskUpdate.cutOff());
      await tester.pumpAndSettle();
      expect(
        find.text('The answer was cut off before it finished.'),
        findsOneWidget,
      );
    });

    testWidgets('an error is a card with one action that asks again', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(
        const AskUpdate.error(message: 'The key was rejected.'),
      );
      await tester.pumpAndSettle();
      expect(find.text('Couldn’t answer'), findsOneWidget);
      expect(find.text('The key was rejected.'), findsOneWidget);
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(ai.asked, <String>['q', 'q']);
    });

    testWidgets('Stop ends the answer and Clear empties the conversation', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(const AskUpdate.answer(text: 'Half'));
      await tester.pumpAndSettle();
      await tester.tap(find.bySemanticsLabel('Stop'));
      await tester.pumpAndSettle();
      // The core is told once the stream has let go.
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      await tester.pumpAndSettle();
      expect(ai.stops, 1);
      expect(find.text('Stopped.'), findsOneWidget);
      expect(find.text('Half'), findsOneWidget);
      await tester.tap(find.bySemanticsLabel('Clear conversation'));
      await tester.pumpAndSettle();
      expect(find.text('Half'), findsNothing);
      expect(ai.clears, 1);
    });

    testWidgets('an answer draws no image and no HTML', (tester) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(
        const AskUpdate.done(
          answer:
              'See ![chart](https://x.test/c.png) <img src=x> '
              '[click](javascript:alert)',
          used: <AiNoteRef>[],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(Image), findsNothing);
      expect(
        find.textContaining('<img src=x>', findRichText: true),
        findsOneWidget,
      );
      expect(find.byType(CitationChip), findsNothing);
    });
  });
}
