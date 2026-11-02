// Kanso — agent_review_page_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_review/agent_review_bloc.dart';
import 'package:kanso/agent_tasks/views/pages/agent_review_page.dart';
import 'package:kanso/core/views/widgets/kanso_page.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';
import '../../helpers/kanso_test_app.dart';

Future<(FixtureAgentTasksRepository, AgentReviewBloc)> _open(
  WidgetTester tester,
) async {
  final repo = FixtureAgentTasksRepository();
  final bloc = AgentReviewBloc(repo)..add(const AgentReviewEvent.opened(1));
  addTearDown(bloc.close);
  await tester.pumpWidget(
    KansoTestApp(
      home: BlocProvider<AgentReviewBloc>.value(
        value: bloc,
        child: const KansoPage(body: AgentReviewView()),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return (repo, bloc);
}

void main() {
  testWidgets('shows the first change and its position', (tester) async {
    await _open(tester);
    expect(find.text('Lisbon plan'), findsOneWidget);
    expect(find.text('1 of 3'), findsOneWidget);
    expect(find.text('Apply 3 changes'), findsOneWidget);
  });

  testWidgets('swiping moves to the next change, which warns it changed', (
    tester,
  ) async {
    await _open(tester);
    await tester.fling(find.text('Lisbon plan'), const Offset(-500, 0), 2000);
    await tester.pumpAndSettle();
    expect(find.text('2 of 3'), findsOneWidget);
    expect(find.text('Packing'), findsOneWidget);
    expect(find.textContaining('changed since the AI read it'), findsOneWidget);
  });

  testWidgets('unticking a change updates Apply, and Apply then Undo work', (
    tester,
  ) async {
    final (repo, bloc) = await _open(tester);
    bloc.add(const AgentReviewEvent.cardToggled(0, on: false));
    await tester.pumpAndSettle();
    expect(find.text('Apply 2 changes'), findsOneWidget);

    await tester.tap(find.text('Apply 2 changes'));
    await tester.pumpAndSettle();
    expect(repo.appliedTicked, <int>[1, 2]);
    expect(find.text('Applied 2 changes from Kanso AI.'), findsOneWidget);

    await tester.tap(find.text('Undo'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('undo:1'));
    expect(find.text('Applied 2 changes from Kanso AI.'), findsNothing);
    // The undo offer ends by itself after 30 seconds.
    await tester.pump(agentUndoFor);
  });

  testWidgets('a merge has a switch to keep the original notes', (
    tester,
  ) async {
    final (repo, bloc) = await _open(tester);
    await tester.fling(find.text('Lisbon plan'), const Offset(-500, 0), 2000);
    await tester.pumpAndSettle();
    await tester.fling(find.text('Packing'), const Offset(-500, 0), 2000);
    await tester.pumpAndSettle();
    expect(find.text('Keep the original notes'), findsOneWidget);
    expect(find.text('The original notes move to the Trash.'), findsOneWidget);
    bloc.add(const AgentReviewEvent.keepToggled(2, on: true));
    await tester.pumpAndSettle();
    expect(
      find.text('The original notes stay where they are.'),
      findsOneWidget,
    );
    bloc.add(const AgentReviewEvent.applied());
    await tester.pumpAndSettle();
    expect(repo.appliedKeep, <int>[2]);
    await tester.pump(agentUndoFor);
  });
}
