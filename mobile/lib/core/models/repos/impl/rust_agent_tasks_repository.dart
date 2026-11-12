// Kanso — rust_agent_tasks_repository.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/agent_tasks.dart' as bridge;
import '../../../bridge/api/ai.dart' show KansoAi;
import '../../../bridge/api/types.dart' show KansoError;
import '../../../utils/kanso_error_mapping.dart';
import '../../data/agent_card.dart';
import '../../data/agent_review.dart';
import '../../data/agent_session.dart';
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

  AgentSession _session(bridge.SessionView s) => AgentSession(
    id: s.id,
    key: s.key,
    title: s.title,
    agent: s.agent,
    phase: AgentTaskPhase.of(s.phase),
    detail: s.detail,
    dot: AgentSessionDot.of(s.dot),
    unread: s.unread,
    lastActivityMs: s.lastActivityMs,
    notesRead: s.notesRead,
    changes: s.changes,
    canStop: s.canStop,
    canReview: s.canReview,
    canFollowUp: s.canFollowUp,
  );

  AgentTranscriptItem _item(bridge.TranscriptItem i) => switch (i.kind) {
    'user' => AgentTranscriptItem.user(i.text),
    'agent' => AgentTranscriptItem.agent(i.text),
    'thought' => AgentTranscriptItem.thought(
      text: i.text,
      seconds: i.thoughtSecs,
    ),
    'tool' => AgentTranscriptItem.tool(
      title: i.title,
      kind: AgentToolKind.of(i.toolKind),
      status: AgentToolStatus.of(i.status),
    ),
    'permission' => AgentTranscriptItem.permission(
      title: i.title,
      answer: AgentPermissionAnswer.of(i.answer),
    ),
    'proposed' => AgentTranscriptItem.proposed(
      round: i.round,
      counts: i.counts,
      state: AgentRoundState.of(i.state),
    ),
    _ => AgentTranscriptItem.notice(i.text),
  };

  @override
  Future<Either<ApplicationError, List<AgentSession>>> sessions() => attempt(
    () async => <AgentSession>[
      for (final row in await _tasks.agentSessions()) _session(row),
    ],
  );

  @override
  Future<Either<ApplicationError, AgentTranscript>> transcript(String key) =>
      attempt(() async {
        final found = await _tasks.agentTranscript(key: key);
        if (found == null) {
          throw const KansoError.notFound();
        }
        return AgentTranscript(
          session: _session(found.session),
          version: found.version,
          items: <AgentTranscriptItem>[for (final i in found.items) _item(i)],
        );
      });

  @override
  Future<Either<ApplicationError, Unit>> followUp(String key, String text) =>
      attempt(() async {
        await _tasks.agentFollowUp(key: key, text: text);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> endSession(String key) =>
      attempt(() async {
        await _tasks.agentEndSession(key: key);
        return unit;
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
