// Kanso — fixture_agent_tasks_repository.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/agent_card.dart';
import 'package:kanso/core/models/data/agent_review.dart';
import 'package:kanso/core/models/data/agent_task.dart';
import 'package:kanso/core/models/data/agent_task_prefs.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/agent_task_notifier.dart';
import 'package:kanso/core/models/repos/abs/agent_tasks_repository.dart';

/// An [AgentTasksRepository] that remembers what it was asked.
class FixtureAgentTasksRepository implements AgentTasksRepository {
  FixtureAgentTasksRepository({
    List<AgentTask>? tasks,
    AgentReview? review,
    this.startError,
  }) : rows = tasks ?? <AgentTask>[],
       proposed = review ?? _review;

  List<AgentTask> rows;
  AgentReview proposed;
  String? startError;
  final List<String> calls = <String>[];
  final List<({String book, bool confirmedAll})> started =
      <({String book, bool confirmedAll})>[];
  List<int>? appliedTicked;
  List<int>? appliedKeep;
  AgentTaskPrefs saved = const AgentTaskPrefs();

  static const AgentReview _review = AgentReview(
    header: 'Proposed 2 new notes, 1 merge',
    cards: <AgentCard>[
      AgentCard(
        index: 0,
        kind: AgentCardKind.create,
        title: 'Lisbon plan',
        ticked: true,
        lines: <String>['# Lisbon', 'Day one'],
      ),
      AgentCard(
        index: 1,
        kind: AgentCardKind.edit,
        title: 'Packing',
        ticked: true,
        changedSince: true,
        lines: <String>['- socks', '+ sunscreen'],
      ),
      AgentCard(
        index: 2,
        kind: AgentCardKind.merge,
        title: 'Trip',
        ticked: true,
        lines: <String>['+ all of it'],
      ),
    ],
  );

  @override
  Future<Either<ApplicationError, AgentTaskPrefs>> prefs() async =>
      Right<ApplicationError, AgentTaskPrefs>(saved);

  @override
  Future<Either<ApplicationError, AgentTaskPrefs>> savePrefs(
    AgentTaskPrefs prefs,
  ) async {
    saved = prefs;
    return Right<ApplicationError, AgentTaskPrefs>(prefs);
  }

  @override
  Future<Either<ApplicationError, int>> start({
    required String goal,
    List<String> noteIds = const <String>[],
    String newNoteBook = '',
    bool confirmedAll = false,
  }) async {
    calls.add('start:$goal:${noteIds.join(',')}');
    started.add((book: newNoteBook, confirmedAll: confirmedAll));
    final error = startError;
    if (error != null) {
      return Left<ApplicationError, int>(
        ApplicationError.invalid(message: error),
      );
    }
    rows = <AgentTask>[
      ...rows,
      AgentTask(id: rows.length + 1, goal: goal, phase: AgentTaskPhase.running),
    ];
    return Right<ApplicationError, int>(rows.length);
  }

  @override
  Future<Either<ApplicationError, List<AgentTask>>> tasks() async =>
      Right<ApplicationError, List<AgentTask>>(List<AgentTask>.of(rows));

  @override
  Future<Either<ApplicationError, Unit>> stop(int id) async {
    calls.add('stop:$id');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> reply(int id, String text) async {
    calls.add('reply:$id:$text');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> discard(int id) async {
    calls.add('discard:$id');
    rows = rows.where((t) => t.id != id).toList();
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, AgentReview>> review(int id) async =>
      Right<ApplicationError, AgentReview>(proposed);

  @override
  Future<Either<ApplicationError, String>> apply({
    required int id,
    required List<int> ticked,
    required List<int> keepOriginals,
  }) async {
    appliedTicked = ticked;
    appliedKeep = keepOriginals;
    calls.add('apply:${ticked.length}');
    return Right<ApplicationError, String>(
      'Applied ${ticked.length} changes from Kanso AI.',
    );
  }

  @override
  Future<Either<ApplicationError, String>> undo(int id) async {
    calls.add('undo:$id');
    return const Right<ApplicationError, String>('Undone.');
  }
}

/// An [AgentTaskNotifier] that counts.
class RecordingAgentTaskNotifier implements AgentTaskNotifier {
  final List<int> paused_ = <int>[];

  @override
  Future<void> paused({required int id}) async => paused_.add(id);
}
