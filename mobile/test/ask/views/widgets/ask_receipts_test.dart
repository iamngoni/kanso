// Kanso — ask_receipts_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/ask/state/ask/ask_bloc.dart';
import 'package:kanso/ask/utils/open_cited_note.dart';
import 'package:kanso/ask/utils/superscript.dart';
import 'package:kanso/ask/views/widgets/ask_pane.dart';
import 'package:kanso/core/models/data/ai_note_ref.dart';
import 'package:kanso/core/models/data/ask_closest.dart';
import 'package:kanso/core/models/data/ask_receipt.dart';
import 'package:kanso/core/models/data/ask_scope.dart';
import 'package:kanso/core/models/data/ask_time_note.dart';
import 'package:kanso/core/models/data/ask_update.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../../helpers/ai_blocs.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_routed.dart';

const _pricing = AiNoteRef(id: 'n1', title: 'Pricing');
const _receipt = AskReceipt(
  n: 1,
  noteId: 'n1',
  title: 'Pricing',
  label: 'Pricing · line 3 · 14 Oct',
  start: 2,
  end: 2,
  quote: 'The yearly plan is 40 dollars.',
);

late GoRouter router;

Future<FixtureAiRepository> _pump(WidgetTester tester) async {
  final ai = FixtureAiRepository(settings: aiSettings(enabled: true));
  addTearDown(ai.close);
  final bloc = AskBloc(ai)..add(const AskEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
  router = await pumpRouted(
    tester,
    BlocProvider<AskBloc>.value(value: bloc, child: const AskPane()),
  );
  return ai;
}

Future<void> _ask(WidgetTester tester, String question) async {
  await tester.enterText(find.byType(EditableText), question);
  await tester.pump();
  await tester.tap(find.bySemanticsLabel('Ask'));
  await tester.pumpAndSettle();
}

void main() {
  group('superscript', () {
    test('writes every digit', () {
      expect(superscript(0), '⁰');
      expect(superscript(12), '¹²');
      expect(superscript(1234567890), '¹²³⁴⁵⁶⁷⁸⁹⁰');
    });
  });

  group('citedLinesLocation', () {
    test('opens the note at the lines, from 0', () {
      expect(
        citedLinesLocation('n:1', 2, 4),
        '${citedNoteLocation('n:1')}?line=2&lineEnd=4',
      );
    });
  });

  group('Ask with receipts', () {
    testWidgets('a receipt is a mark, a Sources row and a sheet of the quote', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'What is the yearly plan?');
      ai.askUpdates.add(
        const AskUpdate.done(
          answer: 'It is 40. [1](kanso-cite:1)',
          used: <AiNoteRef>[_pricing],
          receipts: <AskReceipt>[_receipt],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Sources'), findsOneWidget);
      expect(find.text('Pricing · line 3 · 14 Oct'), findsOneWidget);
      // The mark and the Sources row both show the number.
      expect(find.text('¹'), findsWidgets);
      await tester.tap(find.text('Pricing · line 3 · 14 Oct'));
      await tester.pumpAndSettle();
      expect(find.text('The yearly plan is 40 dollars.'), findsOneWidget);
      expect(find.text('Open note'), findsOneWidget);
    });

    testWidgets('a sentence that failed its check says it was not found', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(
        const AskUpdate.done(
          answer: '[It is free.](kanso-unverified:1)',
          used: <AiNoteRef>[_pricing],
          banner: 'Most of this answer could not be checked.',
          unverified: 1,
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.text('Most of this answer could not be checked.'),
        findsOneWidget,
      );
      expect(find.text('It is free.'), findsOneWidget);
    });

    testWidgets('the notes not holding the answer says so, with the closest', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(
        const AskUpdate.done(
          answer: '',
          used: <AiNoteRef>[],
          neverWrote: true,
          closest: <AskClosest>[AskClosest(noteId: 'n1', title: 'Pricing')],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('You never wrote that down.'), findsOneWidget);
      expect(find.text('Closest notes'), findsOneWidget);
      expect(find.text('Pricing'), findsOneWidget);
      expect(find.text('Save as note'), findsNothing);
    });

    testWidgets('the time chip can be dropped, and an answer saved', (
      tester,
    ) async {
      final ai = await _pump(tester);
      await _ask(tester, 'what did I write last month?');
      ai.askUpdates
        ..add(const AskUpdate.time(time: AskTimeNote(label: 'Written June')))
        ..add(
          const AskUpdate.done(answer: 'A lot.', used: <AiNoteRef>[_pricing]),
        );
      await tester.pumpAndSettle();
      expect(find.text('Written June'), findsOneWidget);
      await tester.tap(find.text('Save as note'));
      await tester.pumpAndSettle();
      expect(ai.saves, 1);
      expect(find.text('Saved as a note.'), findsOneWidget);
    });

    testWidgets('nothing in a narrow scope offers all notes', (tester) async {
      final ai = await _pump(tester);
      await _ask(tester, 'q');
      ai.askUpdates.add(const AskUpdate.nothingMatches(narrow: true));
      await tester.pumpAndSettle();
      expect(find.text('Search all notes'), findsOneWidget);
      await tester.tap(find.text('Search all notes'));
      await tester.pumpAndSettle();
      expect(ai.asked, <String>['q', 'q']);
    });

    testWidgets('the scope sheet lists what can be asked about', (
      tester,
    ) async {
      final ai = await _pump(tester);
      ai.scopes = const <AskScope>[
        AskScope.everything,
        AskScope(kind: 'book', value: 'b1', label: 'Work'),
      ];
      await tester.tap(find.text('All notes'));
      await tester.pumpAndSettle();
      expect(find.text('Work'), findsOneWidget);
      await tester.tap(find.text('Work'));
      await tester.pumpAndSettle();
      expect(find.text('Work'), findsOneWidget);
    });
  });
}
