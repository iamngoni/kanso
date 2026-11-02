// Kanso — agent_tasks_repository.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/agent_review.dart';
import '../../data/agent_task.dart';
import '../../data/agent_task_prefs.dart';
import '../../data/application_error.dart';

/// Agent tasks: the AI reads notes and proposes changes, and nothing is
/// written until the person approves them. Locked notes, the trash and
/// `#ai-exclude` notes are never readable.
abstract interface class AgentTasksRepository {
  /// Settings › AI › Agent tasks.
  Future<Either<ApplicationError, AgentTaskPrefs>> prefs();

  Future<Either<ApplicationError, AgentTaskPrefs>> savePrefs(
    AgentTaskPrefs prefs,
  );

  /// Starts a task and gives its ID. With no [noteIds] it may read every
  /// unlocked note, which needs [confirmedAll] when Settings ask first. New
  /// notes go in [newNoteBook], the Inbox when it is empty.
  Future<Either<ApplicationError, int>> start({
    required String goal,
    List<String> noteIds,
    String newNoteBook,
    bool confirmedAll,
  });

  /// Every task this session, oldest first.
  Future<Either<ApplicationError, List<AgentTask>>> tasks();

  Future<Either<ApplicationError, Unit>> stop(int id);

  Future<Either<ApplicationError, Unit>> reply(int id, String text);

  /// Forgets a task and what it proposed.
  Future<Either<ApplicationError, Unit>> discard(int id);

  Future<Either<ApplicationError, AgentReview>> review(int id);

  /// Writes the cards at [ticked] in one step; the sentence to show. The
  /// merges at [keepOriginals] leave their source notes where they are.
  Future<Either<ApplicationError, String>> apply({
    required int id,
    required List<int> ticked,
    required List<int> keepOriginals,
  });

  /// Takes back what [apply] wrote.
  Future<Either<ApplicationError, String>> undo(int id);
}
