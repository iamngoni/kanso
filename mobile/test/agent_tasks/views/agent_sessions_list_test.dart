// Kanso — agent_sessions_list_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_session/agent_session_bloc.dart';
import 'package:kanso/agent_tasks/state/agent_tasks/agent_tasks_bloc.dart';
import 'package:kanso/agent_tasks/utils/session_text.dart';
import 'package:kanso/agent_tasks/views/pages/agent_tasks_page.dart';
import 'package:kanso/core/models/data/agent_session.dart';
import 'package:kanso/core/models/data/agent_task.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/injection.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';
import '../../helpers/kanso_test_app.dart';

final int _now = DateTime.now().millisecondsSinceEpoch;

AgentSession _session({
  required int id,
  required String title,
  required AgentTaskPhase phase,
  AgentSessionDot dot = AgentSessionDot.idle,
  int agoMs = 0,
  bool unread = false,
  int changes = 0,
  bool canStop = false,
  bool canReview = false,
}) => AgentSession(
  id: id,
  key: 'k$id',
  title: title,
  phase: phase,
  agent: 'Claude Agent',
  dot: dot,
  unread: unread,
  lastActivityMs: _now - agoMs,
  changes: changes,
  canStop: canStop,
  canReview: canReview,
);

Future<FixtureAgentTasksRepository> _open(
  WidgetTester tester,
  List<AgentSession> sessions,
) async {
  final repo = FixtureAgentTasksRepository()..sessionRows = sessions;
  if (getIt.isRegistered<AgentTasksBloc>()) {
    getIt.unregister<AgentTasksBloc>();
  }
  if (getIt.isRegistered<AgentSessionBloc>()) {
    getIt.unregister<AgentSessionBloc>();
  }
  getIt
    ..registerFactory<AgentTasksBloc>(
      () => AgentTasksBloc(repo, RecordingAgentTaskNotifier()),
    )
    ..registerFactory<AgentSessionBloc>(() => AgentSessionBloc(repo));
  addTearDown(() {
    getIt
      ..unregister<AgentTasksBloc>()
      ..unregister<AgentSessionBloc>();
  });
  await tester.pumpWidget(const KansoTestApp(home: AgentTasksPage()));
  await tester.pump();
  await tester.pump();
  return repo;
}

void main() {
  testWidgets('each session is a row: title, agent · status · time', (
    tester,
  ) async {
    await _open(tester, <AgentSession>[
      _session(
        id: 1,
        title: 'Merge my Lisbon notes',
        phase: AgentTaskPhase.ready,
        dot: AgentSessionDot.needsYou,
        agoMs: 2 * 60 * 1000,
        changes: 4,
        canReview: true,
      ),
      _session(
        id: 2,
        title: 'Tidy the inbox',
        phase: AgentTaskPhase.nothing,
        agoMs: 30 * 3600 * 1000,
      ),
    ]);
    expect(find.text('Merge my Lisbon notes'), findsOneWidget);
    expect(
      find.text('Claude Agent · 4 changes to review · 2m ago'),
      findsOneWidget,
    );
    expect(find.text('Claude Agent · Done · yesterday'), findsOneWidget);
  });

  testWidgets('rows keep the order the core sorted them in', (tester) async {
    await _open(tester, <AgentSession>[
      _session(id: 3, title: 'Newest', phase: AgentTaskPhase.nothing),
      _session(
        id: 1,
        title: 'Oldest',
        phase: AgentTaskPhase.nothing,
        agoMs: 5 * 86400 * 1000,
      ),
    ]);
    final newest = tester.getTopLeft(find.text('Newest')).dy;
    final oldest = tester.getTopLeft(find.text('Oldest')).dy;
    expect(newest, lessThan(oldest));
    expect(find.text('Claude Agent · Done · 5d ago'), findsOneWidget);
  });

  testWidgets('an unread session shows the unread dot, a read one does not', (
    tester,
  ) async {
    await _open(tester, <AgentSession>[
      _session(
        id: 1,
        title: 'Fresh',
        phase: AgentTaskPhase.nothing,
        unread: true,
      ),
      _session(id: 2, title: 'Seen', phase: AgentTaskPhase.nothing),
    ]);
    expect(
      find.byKey(const ValueKey<String>('agent-session-unread')),
      findsOne,
    );
  });

  testWidgets('a working row can be stopped; a row to review has Review', (
    tester,
  ) async {
    final repo = await _open(tester, <AgentSession>[
      _session(
        id: 1,
        title: 'Working one',
        phase: AgentTaskPhase.running,
        dot: AgentSessionDot.working,
        canStop: true,
      ),
      _session(
        id: 2,
        title: 'Ready one',
        phase: AgentTaskPhase.ready,
        dot: AgentSessionDot.needsYou,
        changes: 1,
        canReview: true,
      ),
    ]);
    expect(find.text('Stop'), findsOneWidget);
    expect(find.text('Review'), findsOneWidget);
    await tester.tap(find.text('Stop'));
    await tester.pump();
    await tester.pump();
    expect(repo.calls, contains('stop:1'));
  });

  testWidgets('tapping a row opens the session', (tester) async {
    final repo = await _open(tester, <AgentSession>[
      _session(id: 1, title: 'Open me', phase: AgentTaskPhase.nothing),
    ]);
    await tester.tap(find.text('Open me'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    expect(repo.calls, contains('transcript:k1'));
  });

  testWidgets('no sessions shows the empty state', (tester) async {
    await _open(tester, <AgentSession>[]);
    expect(find.text('No agent tasks yet'), findsOneWidget);
  });

  group('session words', () {
    late AppLocalizations l10n;
    setUp(() async {
      l10n = await AppLocalizations.delegate.load(const Locale('en'));
    });

    test('time steps match the desktop list', () {
      String at(int secs) => sessionWhen(l10n, secs * 1000, 0);
      expect(at(0), 'now');
      expect(at(59), 'now');
      expect(at(60), '1m ago');
      expect(at(3599), '59m ago');
      expect(at(3600), '1h ago');
      expect(at(86399), '23h ago');
      expect(at(86400), 'yesterday');
      expect(at(172799), 'yesterday');
      expect(at(172800), '2d ago');
    });

    test('a time in the future reads as now', () {
      expect(sessionWhen(l10n, 0, 5000), 'now');
    });

    test('a failed session with changes still says to review', () {
      final session = _session(
        id: 1,
        title: 't',
        phase: AgentTaskPhase.failed,
        changes: 2,
      );
      expect(sessionStatus(l10n, session), '2 changes to review');
    });
  });

  testWidgets('a long press offers rename and delete for that row', (
    tester,
  ) async {
    final repo = await _open(tester, <AgentSession>[
      _session(id: 1, title: 'Tidy the inbox', phase: AgentTaskPhase.nothing),
    ]);
    await tester.longPress(find.text('Tidy the inbox'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Rename'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).last, 'Inbox, tidy');
    await tester.pump();
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('renameSession:k1:Inbox, tidy'));
    await tester.longPress(find.text('Tidy the inbox'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete session'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete session').last);
    await tester.pumpAndSettle();
    expect(repo.calls, contains('deleteSession:k1'));
  });
}
