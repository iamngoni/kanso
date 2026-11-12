// Kanso — agent_session_page_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_session/agent_session_bloc.dart';
import 'package:kanso/agent_tasks/views/pages/agent_session_page.dart';
import 'package:kanso/agent_tasks/views/widgets/agent_transcript_view.dart';
import 'package:kanso/core/models/data/agent_session.dart';
import 'package:kanso/core/models/data/agent_task.dart';
import 'package:kanso/injection.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';
import '../../helpers/kanso_test_app.dart';

AgentSession _session({
  AgentTaskPhase phase = AgentTaskPhase.ready,
  bool canStop = false,
  bool canReview = true,
  bool canFollowUp = false,
}) => AgentSession(
  id: 4,
  key: 'k4',
  title: 'Merge my Lisbon notes',
  phase: phase,
  agent: 'Claude Agent',
  changes: 4,
  canStop: canStop,
  canReview: canReview,
  canFollowUp: canFollowUp,
);

AgentTranscriptItem _tool(String title, {AgentToolStatus? status}) =>
    AgentTranscriptItem.tool(
      title: title,
      kind: AgentToolKind.read,
      status: status ?? AgentToolStatus.completed,
    );

Future<FixtureAgentTasksRepository> _open(
  WidgetTester tester, {
  required AgentSession session,
  required List<AgentTranscriptItem> items,
}) async {
  final repo = FixtureAgentTasksRepository()
    ..sessionRows = <AgentSession>[session];
  repo.transcripts[session.key] = AgentTranscript(
    session: session,
    version: 1,
    items: items,
  );
  if (getIt.isRegistered<AgentSessionBloc>()) {
    getIt.unregister<AgentSessionBloc>();
  }
  getIt.registerFactory<AgentSessionBloc>(() => AgentSessionBloc(repo));
  addTearDown(() => getIt.unregister<AgentSessionBloc>());
  await tester.pumpWidget(
    KansoTestApp(home: AgentSessionPage(sessionKey: session.key)),
  );
  await tester.pump();
  await tester.pump();
  return repo;
}

void main() {
  testWidgets('the transcript shows each kind of item', (tester) async {
    await _open(
      tester,
      session: _session(),
      items: <AgentTranscriptItem>[
        const AgentTranscriptItem.user('Merge my Lisbon notes'),
        const AgentTranscriptItem.thought(
          text: 'Hmm, three notes',
          seconds: 12,
        ),
        _tool('Read Lisbon plan'),
        const AgentTranscriptItem.agent('I found **three** notes.'),
        const AgentTranscriptItem.permission(
          title: 'Read /Users/me/plan.md',
          answer: AgentPermissionAnswer.allowedForSession,
        ),
        const AgentTranscriptItem.proposed(
          round: 1,
          counts: '1 new note, 1 edit',
          state: AgentRoundState.toReview,
        ),
        const AgentTranscriptItem.notice('Stopped'),
      ],
    );
    expect(find.text('Merge my Lisbon notes'), findsWidgets);
    expect(find.text('Thought for 12s'), findsOneWidget);
    expect(find.text('Read Lisbon plan'), findsOneWidget);
    expect(find.textContaining('three', findRichText: true), findsWidgets);
    expect(
      find.text('Allowed for this session: Read /Users/me/plan.md'),
      findsOneWidget,
    );
    expect(find.text('Round 1 · 1 new note, 1 edit'), findsOneWidget);
    expect(find.text('Review'), findsOneWidget);
    expect(find.text('Stopped'), findsOneWidget);
  });

  testWidgets('thinking is collapsed until it is opened', (tester) async {
    await _open(
      tester,
      session: _session(),
      items: const <AgentTranscriptItem>[
        AgentTranscriptItem.thought(text: 'Secret reasoning', seconds: 3),
      ],
    );
    expect(find.text('Secret reasoning'), findsNothing);
    await tester.tap(find.text('Thought for 3s'));
    await tester.pump();
    expect(find.text('Secret reasoning'), findsOneWidget);
  });

  testWidgets('three tools in a row fold into one line that opens', (
    tester,
  ) async {
    await _open(
      tester,
      session: _session(),
      items: <AgentTranscriptItem>[
        _tool('Read A'),
        _tool('Read B'),
        _tool('Read C'),
      ],
    );
    expect(find.text('Used 3 tools'), findsOneWidget);
    expect(find.text('Read A'), findsNothing);
    await tester.tap(find.text('Used 3 tools'));
    await tester.pump();
    expect(find.text('Read A'), findsOneWidget);
    expect(find.text('Read C'), findsOneWidget);
  });

  testWidgets('two tools stay as rows', (tester) async {
    await _open(
      tester,
      session: _session(),
      items: <AgentTranscriptItem>[_tool('Read A'), _tool('Read B')],
    );
    expect(find.text('Read A'), findsOneWidget);
    expect(find.text('Read B'), findsOneWidget);
    expect(find.textContaining('Used'), findsNothing);
  });

  testWidgets('a running tool shows its working mark', (tester) async {
    await _open(
      tester,
      session: _session(),
      items: <AgentTranscriptItem>[
        _tool('Search notes', status: AgentToolStatus.inProgress),
      ],
    );
    expect(find.byKey(const ValueKey<String>('agent-tool-running')), findsOne);
  });

  testWidgets('a rejected permission is recorded as rejected', (tester) async {
    await _open(
      tester,
      session: _session(),
      items: const <AgentTranscriptItem>[
        AgentTranscriptItem.permission(
          title: 'Delete notes',
          answer: AgentPermissionAnswer.rejected,
        ),
      ],
    );
    expect(find.text('Rejected: Delete notes'), findsOneWidget);
  });

  testWidgets('only the last proposal offers Review', (tester) async {
    await _open(
      tester,
      session: _session(),
      items: const <AgentTranscriptItem>[
        AgentTranscriptItem.proposed(
          round: 1,
          counts: '2 edits',
          state: AgentRoundState.toReview,
        ),
        AgentTranscriptItem.proposed(
          round: 2,
          counts: '1 edit',
          state: AgentRoundState.toReview,
        ),
      ],
    );
    expect(find.text('Review'), findsOneWidget);
  });

  testWidgets('an applied round says so and offers Undo', (tester) async {
    final repo = await _open(
      tester,
      session: _session(
        phase: AgentTaskPhase.applied,
        canReview: false,
      ).copyWith(detail: '4'),
      items: const <AgentTranscriptItem>[
        AgentTranscriptItem.proposed(
          round: 1,
          counts: '4 edits',
          state: AgentRoundState.applied,
        ),
      ],
    );
    expect(find.text('Applied 4 edits'), findsOneWidget);
    await tester.tap(find.text('Undo'));
    await tester.pump();
    await tester.pump();
    expect(repo.calls, contains('undo:4'));
  });

  testWidgets('a discarded round has no buttons', (tester) async {
    await _open(
      tester,
      session: _session(phase: AgentTaskPhase.discarded, canReview: false),
      items: const <AgentTranscriptItem>[
        AgentTranscriptItem.proposed(
          round: 1,
          counts: '2 edits',
          state: AgentRoundState.discarded,
        ),
      ],
    );
    expect(find.text('Discarded 2 edits'), findsOneWidget);
    expect(find.text('Review'), findsNothing);
    expect(find.text('Undo'), findsNothing);
  });

  testWidgets('the composer sends an answer to the same session', (
    tester,
  ) async {
    final repo = await _open(
      tester,
      session: _session(
        phase: AgentTaskPhase.asking,
        canStop: true,
        canReview: false,
        canFollowUp: true,
      ),
      items: const <AgentTranscriptItem>[
        AgentTranscriptItem.agent('Which notebook?'),
      ],
    );
    await tester.enterText(find.byType(EditableText), 'Travel');
    await tester.pump();
    await tester.tap(find.bySemanticsLabel('Send message'));
    await tester.pump();
    await tester.pump();
    expect(repo.calls, contains('followUp:k4:Travel'));
    // The field is empty again.
    expect(
      tester.widget<EditableText>(find.byType(EditableText)).controller.text,
      isEmpty,
    );
  });

  testWidgets('a refused message shows why', (tester) async {
    final repo = await _open(
      tester,
      session: _session(
        phase: AgentTaskPhase.asking,
        canStop: true,
        canReview: false,
        canFollowUp: true,
      ),
      items: const <AgentTranscriptItem>[],
    );
    repo.followUpError = "The agent isn't waiting for a message yet.";
    await tester.enterText(find.byType(EditableText), 'Hello');
    await tester.pump();
    await tester.tap(find.bySemanticsLabel('Send message'));
    await tester.pump();
    await tester.pump();
    expect(find.textContaining("isn't waiting"), findsOneWidget);
  });

  testWidgets('while the AI works the composer says so and offers Stop', (
    tester,
  ) async {
    final repo = await _open(
      tester,
      session: _session(
        phase: AgentTaskPhase.running,
        canStop: true,
        canReview: false,
      ),
      items: const <AgentTranscriptItem>[],
    );
    expect(
      find.text('The AI is working. You can write when it asks.'),
      findsOneWidget,
    );
    expect(find.byType(EditableText), findsNothing);
    await tester.tap(find.bySemanticsLabel('Stop'));
    await tester.pump();
    await tester.pump();
    expect(repo.calls, contains('stop:4'));
  });

  testWidgets('a session that ended says so in place of the field', (
    tester,
  ) async {
    await _open(
      tester,
      session: _session(phase: AgentTaskPhase.nothing, canReview: false),
      items: const <AgentTranscriptItem>[],
    );
    expect(find.text('This session has ended.'), findsOneWidget);
    expect(find.byType(EditableText), findsNothing);
  });

  testWidgets('a session that is gone says so', (tester) async {
    final repo = FixtureAgentTasksRepository();
    if (getIt.isRegistered<AgentSessionBloc>()) {
      getIt.unregister<AgentSessionBloc>();
    }
    getIt.registerFactory<AgentSessionBloc>(() => AgentSessionBloc(repo));
    addTearDown(() => getIt.unregister<AgentSessionBloc>());
    await tester.pumpWidget(
      const KansoTestApp(home: AgentSessionPage(sessionKey: 'nope')),
    );
    await tester.pump();
    await tester.pump();
    expect(find.text('This session is no longer here.'), findsOneWidget);
  });

  group('folding tool calls', () {
    test('runs of three or more fold, shorter runs stay rows', () {
      AgentToolItem tool(String t) => AgentToolItem(
        title: t,
        kind: AgentToolKind.read,
        status: AgentToolStatus.completed,
      );
      const user = AgentUserItem('hi');
      final blocks = AgentTranscriptView.blocksOf(<AgentTranscriptItem>[
        tool('1'),
        tool('2'),
        user,
        tool('3'),
        tool('4'),
        tool('5'),
      ]);
      expect(blocks, hasLength(4));
      expect(blocks[0], isA<AgentToolItem>());
      expect(blocks[1], isA<AgentToolItem>());
      expect(blocks[2], user);
      expect(blocks[3], isA<List<AgentToolItem>>());
      expect(blocks[3] as List<AgentToolItem>, hasLength(3));
    });
  });

  testWidgets('the menu renames the session', (tester) async {
    final repo = await _open(
      tester,
      session: _session(),
      items: const <AgentTranscriptItem>[],
    );
    await tester.tap(find.bySemanticsLabel('Session actions'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Rename'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText), '  Lisbon, done  ');
    await tester.pump();
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('renameSession:k4:Lisbon, done'));
  });

  testWidgets('the menu deletes the session only once confirmed', (
    tester,
  ) async {
    final repo = await _open(
      tester,
      session: _session(),
      items: const <AgentTranscriptItem>[],
    );
    await tester.tap(find.bySemanticsLabel('Session actions'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete session'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(repo.calls.where((c) => c.startsWith('deleteSession')), isEmpty);
    await tester.tap(find.bySemanticsLabel('Session actions'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete session'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete session').last);
    await tester.pumpAndSettle();
    expect(repo.calls, contains('deleteSession:k4'));
  });
}
