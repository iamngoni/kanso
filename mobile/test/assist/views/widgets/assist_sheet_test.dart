// Kanso — assist_sheet_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/assist/models/data/assist_placement.dart';
import 'package:kanso/assist/state/assist/assist_bloc.dart';
import 'package:kanso/assist/views/widgets/assist_sheet.dart';
import 'package:kanso/core/models/data/assist_update.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_routed.dart';

class _Applied {
  final List<(AssistPlacement, String)> calls = <(AssistPlacement, String)>[];
  bool accepts = true;
}

Future<(FixtureAiRepository, _Applied)> _pump(
  WidgetTester tester, {
  String selection = 'we ship on friday',
  bool? hasSelection,
  bool summarizeNote = false,
  bool withApply = true,
}) async {
  final ai = FixtureAiRepository();
  addTearDown(ai.close);
  final applied = _Applied();
  final bloc = AssistBloc(ai);
  closeBlocLater(bloc);
  if (summarizeNote) {
    bloc.add(const AssistEvent.summaryRequested('n1'));
  }
  await pumpRouted(
    tester,
    BlocProvider<AssistBloc>.value(
      value: bloc,
      child: Align(
        alignment: Alignment.bottomCenter,
        child: AssistSheet(
          noteId: 'n1',
          selection: selection,
          hasSelection: hasSelection ?? selection.isNotEmpty,
          before: 'Earlier.',
          summarizeNote: summarizeNote,
          onApply: withApply
              ? (placement, text) {
                  applied.calls.add((placement, text));
                  return applied.accepts;
                }
              : null,
        ),
      ),
    ),
  );
  return (ai, applied);
}

Future<void> _choose(WidgetTester tester, String label) async {
  await tester.tap(find.text(label));
  await _settle(tester);
}

/// Lets an update reach the bloc, and the bloc’s state the screen.
Future<void> _settle(WidgetTester tester) async {
  await tester.pump();
  await tester.pump();
}

/// Waits for a stream to let go, which takes a real turn of the event loop.
Future<void> _stopped(WidgetTester tester) async {
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await _settle(tester);
}

void main() {
  group('AssistSheet', () {
    testWidgets('lists the five actions for a selection', (tester) async {
      await _pump(tester);
      for (final label in <String>[
        'Improve Writing',
        'Make Shorter',
        'Fix Spelling and Grammar',
        'Continue Writing',
        'Summarize Selection',
      ]) {
        expect(find.text(label), findsOneWidget, reason: label);
      }
      expect(find.text('Summarize this note'), findsNothing);
    });

    testWidgets('with only a caret: Continue Writing and the note summary', (
      tester,
    ) async {
      await _pump(tester, selection: 'Text before.', hasSelection: false);
      expect(find.text('Continue Writing'), findsOneWidget);
      expect(find.text('Improve Writing'), findsNothing);
      expect(find.text('Summarize this note'), findsOneWidget);
    });

    testWidgets('thinks, streams, then Accept replaces the selection', (
      tester,
    ) async {
      final (ai, applied) = await _pump(tester);
      await _choose(tester, 'Improve Writing');
      expect(find.text('Improved'), findsOneWidget);
      expect(find.text('Thinking…'), findsOneWidget);
      expect(find.text('Stop'), findsOneWidget);

      ai.assistUpdates.add(const AssistUpdate.text(text: 'We ship on'));
      await _settle(tester);
      expect(find.text('We ship on'), findsOneWidget);
      expect(find.text('Thinking…'), findsNothing);

      ai.assistUpdates.add(
        const AssistUpdate.done(suggestion: 'We ship on Friday.'),
      );
      await _settle(tester);
      expect(find.text('Stop'), findsNothing);
      await tester.tap(find.text('Accept'));
      await _settle(tester);
      expect(applied.calls, <(AssistPlacement, String)>[
        (AssistPlacement.replace, 'We ship on Friday.'),
      ]);
    });

    testWidgets('a continuation goes below the text', (tester) async {
      final (ai, applied) = await _pump(tester);
      await _choose(tester, 'Continue Writing');
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: 'And then.'));
      await _settle(tester);
      await tester.tap(find.text('Accept'));
      await _settle(tester);
      expect(applied.calls.single.$1, AssistPlacement.insertBelow);
    });

    testWidgets(
      'a stopped rewrite cannot be accepted, a stopped continuation can',
      (
        tester,
      ) async {
        final (ai, applied) = await _pump(tester);
        await _choose(tester, 'Make Shorter');
        ai.assistUpdates.add(const AssistUpdate.text(text: 'We'));
        await _settle(tester);
        await tester.tap(find.text('Stop'));
        await _stopped(tester);
        await _settle(tester);
        expect(
          find.text('Stopped. A partial rewrite can’t be applied.'),
          findsOneWidget,
        );
        await tester.tap(find.text('Accept'));
        await _settle(tester);
        expect(applied.calls, isEmpty);

        await tester.tap(find.text('Discard'));
        await _settle(tester);
        await _choose(tester, 'Continue Writing');
        ai.assistUpdates.add(const AssistUpdate.text(text: 'And'));
        await _settle(tester);
        await tester.tap(find.text('Stop'));
        await _stopped(tester);
        await _settle(tester);
        expect(find.text('Stopped.'), findsOneWidget);
        await tester.tap(find.text('Accept'));
        await _settle(tester);
        expect(applied.calls.single, (AssistPlacement.insertBelow, 'And'));
      },
    );

    testWidgets('says when the note is kept out of AI', (tester) async {
      final (ai, _) = await _pump(tester);
      await _choose(tester, 'Improve Writing');
      ai.assistUpdates.add(const AssistUpdate.keptOut());
      await _settle(tester);
      expect(find.text('This note is kept out of AI.'), findsOneWidget);
      expect(find.text('Accept'), findsNothing);
      expect(find.text('Close'), findsOneWidget);
    });

    testWidgets('shows an error in the core’s words, and nothing to accept', (
      tester,
    ) async {
      final (ai, applied) = await _pump(tester);
      await _choose(tester, 'Fix Spelling and Grammar');
      ai.assistUpdates.add(const AssistUpdate.error(message: 'Offline.'));
      await _settle(tester);
      expect(find.text('Offline.'), findsOneWidget);
      await tester.tap(find.text('Accept'));
      await _settle(tester);
      expect(applied.calls, isEmpty);
    });

    testWidgets('keeps the card and says so if the text changed', (
      tester,
    ) async {
      final (ai, applied) = await _pump(tester);
      applied.accepts = false;
      await _choose(tester, 'Improve Writing');
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: 'Better.'));
      await _settle(tester);
      await tester.tap(find.text('Accept'));
      await _settle(tester);
      expect(
        find.text('The text changed, so the suggestion wasn’t applied.'),
        findsOneWidget,
      );
      expect(find.text('Better.'), findsOneWidget);
    });

    testWidgets('Discard goes back to the actions', (tester) async {
      final (ai, _) = await _pump(tester);
      await _choose(tester, 'Improve Writing');
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: 'Better.'));
      await _settle(tester);
      await tester.tap(find.text('Discard'));
      await _settle(tester);
      expect(find.text('Improve Writing'), findsOneWidget);
    });

    testWidgets('a note summary starts at once, and can only be copied here', (
      tester,
    ) async {
      final (ai, _) = await _pump(
        tester,
        summarizeNote: true,
        withApply: false,
      );
      await _settle(tester);
      expect(find.text('Summary'), findsOneWidget);
      expect(find.text('Thinking…'), findsOneWidget);
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: 'Short.'));
      await _settle(tester);
      expect(find.text('Short.'), findsOneWidget);
      expect(find.text('Copy'), findsOneWidget);
      expect(find.text('Accept'), findsNothing);
    });
  });
}
