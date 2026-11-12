// Kanso — agent_session_bloc_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_session/agent_session_bloc.dart';
import 'package:kanso/core/models/data/agent_task.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';

void main() {
  group('AgentSessionBloc', () {
    late FixtureAgentTasksRepository repo;
    late AgentSessionBloc bloc;

    setUp(() async {
      repo = FixtureAgentTasksRepository(
        tasks: <AgentTask>[
          const AgentTask(id: 1, goal: 'Tidy', phase: AgentTaskPhase.asking),
        ],
      );
      bloc = AgentSessionBloc(repo)
        ..add(const AgentSessionEvent.started('session-1'));
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
    });

    test('opening a session loads its transcript', () {
      expect(bloc.state.transcript?.session.title, 'Tidy');
      expect(bloc.state.gone, isFalse);
      expect(repo.calls, contains('transcript:session-1'));
    });

    test('a follow-up goes to the same session and refreshes', () async {
      bloc.add(const AgentSessionEvent.followUpSent('Travel'));
      await bloc.stream.firstWhere((s) => !s.sending && s.trouble == null);
      await Future<void>.delayed(Duration.zero);
      expect(repo.calls, contains('followUp:session-1:Travel'));
      expect(bloc.state.sending, isFalse);
    });

    test('a refused follow-up keeps the core sentence', () async {
      repo.followUpError = "The agent isn't waiting for a message yet.";
      bloc.add(const AgentSessionEvent.followUpSent('Hello'));
      await bloc.stream.firstWhere((s) => s.trouble != null);
      expect(bloc.state.trouble, contains("isn't waiting"));
      bloc.add(const AgentSessionEvent.troubleCleared());
      await bloc.stream.firstWhere((s) => s.trouble == null);
    });

    test('stop and end reach the core with the right ids', () async {
      bloc
        ..add(const AgentSessionEvent.stopped())
        ..add(const AgentSessionEvent.ended());
      await pumpEventQueue();
      expect(
        repo.calls,
        containsAllInOrder(<String>['stop:1', 'endSession:session-1']),
      );
    });

    test('dismissing discards the task', () async {
      bloc.add(const AgentSessionEvent.dismissed());
      await bloc.stream.firstWhere((s) => s.gone);
      expect(repo.calls, contains('discard:1'));
    });

    test('a session that is no longer listed is gone', () async {
      repo.rows = <AgentTask>[];
      bloc.add(const AgentSessionEvent.polled());
      await bloc.stream.firstWhere((s) => s.gone);
    });

    test('undo reaches the core', () async {
      bloc.add(const AgentSessionEvent.undone());
      await pumpEventQueue();
      expect(repo.calls, contains('undo:1'));
    });
  });
}
