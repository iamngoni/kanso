// Kanso — agent_review_bloc_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/agent_tasks/state/agent_review/agent_review_bloc.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../helpers/fixture_agent_tasks_repository.dart';

void main() {
  group('AgentReviewBloc', () {
    late FixtureAgentTasksRepository repo;
    late AgentReviewBloc bloc;

    setUp(() async {
      repo = FixtureAgentTasksRepository();
      bloc = AgentReviewBloc(repo)..add(const AgentReviewEvent.opened(3));
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
    });

    test('opens with every card the core ticked', () {
      expect(bloc.state.ticked, <int>{0, 1, 2});
      expect(bloc.state.review!.cards, hasLength(3));
    });

    test('applies only what is ticked, with the merges to keep', () async {
      bloc
        ..add(const AgentReviewEvent.cardToggled(1, on: false))
        ..add(const AgentReviewEvent.keepToggled(2, on: true))
        ..add(const AgentReviewEvent.applied());
      await bloc.stream.firstWhere((s) => s.applied != null);
      expect(repo.appliedTicked, <int>[0, 2]);
      expect(repo.appliedKeep, <int>[2]);
      expect(bloc.state.applied, 'Applied 2 changes from Kanso AI.');
    });

    test('a second Apply does nothing', () async {
      bloc
        ..add(const AgentReviewEvent.applied())
        ..add(const AgentReviewEvent.applied());
      await bloc.stream.firstWhere((s) => s.applied != null);
      await Future<void>.delayed(Duration.zero);
      expect(repo.calls.where((c) => c.startsWith('apply')), hasLength(1));
    });

    test('undo takes the changes back and clears the offer', () async {
      bloc.add(const AgentReviewEvent.applied());
      await bloc.stream.firstWhere((s) => s.applied != null);
      bloc.add(const AgentReviewEvent.undone());
      await bloc.stream.firstWhere((s) => s.applied == null);
      expect(repo.calls, contains('undo:3'));
      expect(bloc.state.undone, 'Undone.');
    });
  });
}
