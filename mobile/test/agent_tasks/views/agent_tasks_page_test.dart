// Kanso — agent_tasks_page_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_tasks/agent_tasks_bloc.dart';
import 'package:kanso/agent_tasks/views/pages/agent_tasks_page.dart';
import 'package:kanso/injection.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';
import '../../helpers/kanso_test_app.dart';

Future<FixtureAgentTasksRepository> _open(
  WidgetTester tester, {
  List<String> noteIds = const <String>[],
}) async {
  final repo = FixtureAgentTasksRepository();
  if (getIt.isRegistered<AgentTasksBloc>()) {
    getIt.unregister<AgentTasksBloc>();
  }
  getIt.registerFactory<AgentTasksBloc>(
    () => AgentTasksBloc(repo, RecordingAgentTaskNotifier()),
  );
  addTearDown(() => getIt.unregister<AgentTasksBloc>());
  await tester.pumpWidget(KansoTestApp(home: AgentTasksPage(noteIds: noteIds)));
  await tester.pumpAndSettle();
  await tester.enterText(find.byType(EditableText).first, 'Tidy my notes');
  await tester.pump();
  return repo;
}

void main() {
  testWidgets('a task that may read every notebook asks first', (tester) async {
    final repo = await _open(tester);
    await tester.tap(find.text('Start'));
    await tester.pumpAndSettle();
    expect(find.text('Let the AI read all your notebooks?'), findsOneWidget);
    expect(repo.started, isEmpty);

    await tester.tap(find.text('Read all notebooks'));
    await tester.pump(const Duration(milliseconds: 300));
    expect(repo.started, hasLength(1));
    expect(repo.started.single.confirmedAll, isTrue);
    // New notes go in the Inbox unless another notebook is picked.
    expect(repo.started.single.book, '');
  });

  testWidgets('cancelling the question starts nothing', (tester) async {
    final repo = await _open(tester);
    await tester.tap(find.text('Start'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(repo.started, isEmpty);
  });

  testWidgets('chosen notes need no question', (tester) async {
    final repo = await _open(tester, noteIds: <String>['n1']);
    await tester.tap(find.text('Start'));
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.text('Let the AI read all your notebooks?'), findsNothing);
    expect(repo.started.single.confirmedAll, isFalse);
  });

  testWidgets('the page shows where new notes go, the Inbox by default', (
    tester,
  ) async {
    await _open(tester);
    expect(find.text('New notes go in'), findsOneWidget);
    expect(find.text('Inbox'), findsOneWidget);
  });
}
