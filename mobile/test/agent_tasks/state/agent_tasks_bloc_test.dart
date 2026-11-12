// Kanso — agent_tasks_bloc_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_tasks/agent_tasks_bloc.dart';
import 'package:kanso/core/models/data/agent_task.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';

Future<void> _settle() => Future<void>.delayed(Duration.zero);

void main() {
  group('AgentTasksBloc', () {
    late FixtureAgentTasksRepository repo;
    late RecordingAgentTaskNotifier notifier;
    late AgentTasksBloc bloc;

    setUp(() async {
      repo = FixtureAgentTasksRepository();
      notifier = RecordingAgentTaskNotifier();
      bloc = AgentTasksBloc(repo, notifier)
        ..add(const AgentTasksEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
    });

    test('starting a task lists it and passes the chosen notes', () async {
      bloc.add(
        const AgentTasksEvent.goalSent('Merge', noteIds: <String>['n1']),
      );
      await bloc.stream.firstWhere((s) => s.sessions.length == 1);
      expect(repo.calls, contains('start:Merge:n1'));
      expect(bloc.state.sessions.single.phase, AgentTaskPhase.running);
      expect(bloc.state.starting, isFalse);
      expect(bloc.state.started, 1);
    });

    test('where new notes go and the confirmation reach the core', () async {
      bloc.add(
        const AgentTasksEvent.goalSent(
          'Merge',
          newNoteBook: 'book:x',
          confirmedAll: true,
        ),
      );
      await bloc.stream.firstWhere((s) => s.sessions.length == 1);
      expect(repo.started.single, (book: 'book:x', confirmedAll: true));
    });

    test('a refused start keeps the sentence for the page to word', () async {
      repo.startError = 'Agent tasks are turned off in Settings.';
      bloc.add(const AgentTasksEvent.goalSent('Merge'));
      await bloc.stream.firstWhere((s) => s.trouble != null);
      expect(bloc.state.trouble, contains('turned off'));
      bloc.add(const AgentTasksEvent.troubleCleared());
      await bloc.stream.firstWhere((s) => s.trouble == null);
    });

    test('events run in the order they were sent', () async {
      bloc
        ..add(const AgentTasksEvent.goalSent('One'))
        ..add(const AgentTasksEvent.stopped(1))
        ..add(const AgentTasksEvent.dismissed(1));
      await bloc.stream.firstWhere(
        (s) => s.sessions.isEmpty && repo.calls.length == 3,
      );
      expect(repo.calls, <String>[
        'start:One:',
        'stop:1',
        'discard:1',
      ]);
    });

    test('a task that stops to be reviewed is announced once', () async {
      repo.rows = <AgentTask>[
        const AgentTask(id: 7, goal: 'g', phase: AgentTaskPhase.running),
      ];
      bloc.add(const AgentTasksEvent.polled());
      await bloc.stream.firstWhere((s) => s.sessions.isNotEmpty);
      repo.rows = <AgentTask>[
        const AgentTask(
          id: 7,
          goal: 'g',
          phase: AgentTaskPhase.ready,
          changes: 2,
        ),
      ];
      bloc
        ..add(const AgentTasksEvent.polled())
        ..add(const AgentTasksEvent.polled());
      await bloc.stream.firstWhere(
        (s) => s.sessions.any((t) => t.phase == AgentTaskPhase.ready),
      );
      await _settle();
      expect(notifier.paused_, <int>[7]);
    });
  });
}
