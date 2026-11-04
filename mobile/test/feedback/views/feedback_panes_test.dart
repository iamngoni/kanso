// Kanso — feedback_panes_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/feedback/models/data/feedback_entry.dart';
import 'package:kanso/feedback/models/data/feedback_result.dart';
import 'package:kanso/feedback/state/my_feedback/my_feedback_bloc.dart';
import 'package:kanso/feedback/state/send_feedback/send_feedback_bloc.dart';
import 'package:kanso/feedback/views/widgets/my_feedback_pane.dart';
import 'package:kanso/feedback/views/widgets/send_feedback_pane.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/fixture_feedback_repository.dart';
import '../../helpers/pump_routed.dart';

Future<FixtureFeedbackRepository> _pumpSend(WidgetTester tester) async {
  final repository = FixtureFeedbackRepository();
  final bloc = SendFeedbackBloc(repository, FixtureFeedbackScreenshots())
    ..add(const SendFeedbackEvent.started());
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<SendFeedbackBloc>.value(
      value: bloc,
      child: const SendFeedbackPane(),
    ),
  );
  await tester.pump();
  return repository;
}

Future<FixtureFeedbackRepository> _pumpMine(
  WidgetTester tester,
  List<FeedbackEntry> entries,
) async {
  final repository = FixtureFeedbackRepository(entries: entries);
  final bloc = MyFeedbackBloc(repository)..add(const MyFeedbackEvent.started());
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<MyFeedbackBloc>.value(
      value: bloc,
      child: const MyFeedbackPane(),
    ),
  );
  await tester.pumpAndSettle();
  return repository;
}

void main() {
  group('SendFeedbackPane', () {
    testWidgets('Send waits for ten characters, then sends the draft', (
      tester,
    ) async {
      final repository = await _pumpSend(tester);
      // A refusal keeps the page, so nothing pops.
      repository.result = const FeedbackSendResult(
        outcome: FeedbackOutcome.rejected,
        field: 'text',
      );
      expect(find.text('Idea'), findsOneWidget);
      expect(find.text('Question'), findsOneWidget);

      await tester.tap(find.text('Send'));
      await tester.pump();
      expect(repository.sent, isEmpty);

      await tester.enterText(
        find.byType(EditableText).first,
        'Export cuts off tables',
      );
      await tester.pump();
      await tester.tap(find.text('Send'));
      await tester.pump();
      expect(repository.sent, hasLength(1));
      expect(repository.sent.single.text, 'Export cuts off tables');
    });

    testWidgets('Show what is sent shows the report', (tester) async {
      await _pumpSend(tester);
      await tester.enterText(
        find.byType(EditableText).first,
        'Export cuts off tables',
      );
      await tester.pump();
      await tester.ensureVisible(find.text('Show what’s sent'));
      await tester.tap(find.text('Show what’s sent'));
      await tester.pumpAndSettle();
      expect(find.text('Hide what’s sent'), findsOneWidget);
      expect(find.textContaining('Export cuts off tables'), findsWidgets);
    });

    testWidgets('the credit name shows only when credit is on', (
      tester,
    ) async {
      await _pumpSend(tester);
      expect(find.text('Credit me by name'), findsOneWidget);
      expect(find.text('Your name'), findsNothing);
    });
  });

  group('MyFeedbackPane', () {
    testWidgets('no reports says so', (tester) async {
      await _pumpMine(tester, const <FeedbackEntry>[]);
      expect(find.text('No feedback yet'), findsOneWidget);
    });

    testWidgets('each report shows its status', (tester) async {
      await _pumpMine(tester, <FeedbackEntry>[
        sampleFeedbackEntry(
          id: 'fb_b',
          preview: 'Dark mode for the graph',
          chip: FeedbackChip.releasedIn,
          release: '0.9.2',
        ),
        sampleFeedbackEntry(chip: FeedbackChip.filedAs, number: 241),
      ]);
      expect(find.text('Dark mode for the graph'), findsOneWidget);
      expect(find.text('Released in 0.9.2'), findsOneWidget);
      expect(find.text('Export to PDF cuts off tables'), findsOneWidget);
      expect(find.text('Filed as #241'), findsOneWidget);
    });

    testWidgets('looking at the list marks the reports seen', (tester) async {
      final repository = await _pumpMine(tester, <FeedbackEntry>[
        sampleFeedbackEntry(),
      ]);
      await tester.pumpWidget(const SizedBox());
      await tester.pump();
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      expect(repository.seen, 1);
    });
  });
}
