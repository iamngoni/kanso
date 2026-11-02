// Kanso — rust_agent_tasks_repository.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/agent_tasks.dart' as bridge;
import '../../../bridge/api/ai.dart' show KansoAi;
import '../../../utils/kanso_error_mapping.dart';
import '../../data/agent_card.dart';
import '../../data/agent_review.dart';
import '../../data/agent_task.dart';
import '../../data/agent_task_prefs.dart';
import '../../data/application_error.dart';
import '../abs/agent_tasks_repository.dart';

/// [AgentTasksRepository] over the Rust core.
@LazySingleton(as: AgentTasksRepository)
class RustAgentTasksRepository implements AgentTasksRepository {
  RustAgentTasksRepository(KansoAi ai) : _tasks = ai.agentTasks();

  final bridge.KansoAgentTasks _tasks;

  AgentTaskPrefs _prefs(bridge.AgentTaskPrefs p) => AgentTaskPrefs(
    enabled: p.enabled,
    newNotes: p.newNotes,
    edits: p.edits,
    moves: p.moves,
    merges: p.merges,
    timeLimitMinutes: p.timeLimitMinutes,
    maxNotes: p.maxNotes,
    askBeforeReadingAll: p.askBeforeReadingAll,
  );

  @override
  Future<Either<ApplicationError, AgentTaskPrefs>> prefs() =>
      attempt(() async => _prefs(await _tasks.prefs()));

  @override
  Future<Either<ApplicationError, AgentTaskPrefs>> savePrefs(
    AgentTaskPrefs prefs,
  ) => attempt(
    () async => _prefs(
      await _tasks.setPrefs(
        prefs: bridge.AgentTaskPrefs(
          enabled: prefs.enabled,
          newNotes: prefs.newNotes,
          edits: prefs.edits,
          moves: prefs.moves,
          merges: prefs.merges,
          timeLimitMinutes: prefs.timeLimitMinutes,
          maxNotes: prefs.maxNotes,
          askBeforeReadingAll: prefs.askBeforeReadingAll,
        ),
      ),
    ),
  );

  @override
  Future<Either<ApplicationError, int>> start({
    required String goal,
    List<String> noteIds = const <String>[],
    String newNoteBook = '',
    bool confirmedAll = false,
  }) => attempt(
    () => _tasks.start(
      goal: goal,
      noteIds: noteIds,
      newNoteBook: newNoteBook,
      confirmedAll: confirmedAll,
    ),
  );

  @override
  Future<Either<ApplicationError, List<AgentTask>>> tasks() =>
      attempt(() async {
        final rows = await _tasks.tasks();
        return <AgentTask>[
          for (final row in rows)
            AgentTask(
              id: row.id,
              goal: row.goal,
              phase: AgentTaskPhase.of(row.phase),
              detail: row.detail,
              line: row.line,
              lastMessage: row.lastMessage,
              notesRead: row.notesRead,
              changes: row.changes,
              elapsedSeconds: row.elapsedSecs,
            ),
        ];
      });

  @override
  Future<Either<ApplicationError, Unit>> stop(int id) => attempt(() async {
    await _tasks.stop(id: id);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> reply(int id, String text) =>
      attempt(() async {
        await _tasks.reply(id: id, text: text);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> discard(int id) => attempt(() async {
    await _tasks.discard(id: id);
    return unit;
  });

  @override
  Future<Either<ApplicationError, AgentReview>> review(int id) =>
      attempt(() async {
        final review = await _tasks.review(id: id);
        return AgentReview(
          header: review.header,
          fromAgent: review.fromAgent,
          warnings: review.warnings,
          cards: <AgentCard>[
            for (final card in review.cards)
              AgentCard(
                index: card.index,
                kind: AgentCardKind.of(card.kind),
                title: card.title,
                book: card.book,
                ticked: card.ticked,
                changedSince: card.changedSince,
                outside: card.outside,
                published: card.published,
                lines: card.lines,
              ),
          ],
        );
      });

  @override
  Future<Either<ApplicationError, String>> apply({
    required int id,
    required List<int> ticked,
    required List<int> keepOriginals,
  }) => attempt(
    () => _tasks.apply(
      id: id,
      ticked: ticked,
      keep: keepOriginals,
      nowMs: DateTime.now().millisecondsSinceEpoch,
    ),
  );

  @override
  Future<Either<ApplicationError, String>> undo(int id) => attempt(
    () => _tasks.undo(id: id, nowMs: DateTime.now().millisecondsSinceEpoch),
  );
}
