// Kanso — rust_digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/ai.dart';
import '../../../bridge/api/library.dart';
import '../../../bridge/api/overview_types.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/digest_repository.dart';

/// [DigestRepository] over the Rust core's library.
@LazySingleton(as: DigestRepository)
class RustDigestRepository implements DigestRepository {
  RustDigestRepository(this._library, this._ai);

  final KansoLibrary _library;
  final KansoAi _ai;
  KansoAiRun? _run;

  @override
  Future<Either<ApplicationError, OpenLoopsBoard>> openLoops() =>
      attempt(_library.openLoops);

  @override
  Future<Either<ApplicationError, bool>> closeLoop({
    required OpenLoopItem item,
    required LoopAction action,
  }) => attempt(
    () => _library.closeLoop(
      noteId: item.noteId,
      key: item.key,
      line: item.line,
      action: action,
    ),
  );

  @override
  Future<Either<ApplicationError, bool>> doneLoop({
    required String noteId,
    required String key,
    required int line,
  }) => attempt(
    () => _library.closeLoop(
      noteId: noteId,
      key: key,
      line: line,
      action: LoopAction.done,
    ),
  );

  @override
  OverviewToday? overviewSnapshot() => _library.overviewSnapshot();

  @override
  Future<Either<ApplicationError, OverviewToday>> overviewToday() =>
      attempt(_library.overviewToday);

  @override
  Future<Either<ApplicationError, OverviewWeek>> overviewWeek({
    String? from,
  }) => attempt(() => _library.overviewWeek(from: from));

  @override
  Future<Either<ApplicationError, void>> putCalendarEvents(String eventsJson) =>
      attempt(
        () => _library.overviewPutCalendarEvents(eventsJson: eventsJson),
      );

  @override
  Future<Either<ApplicationError, String>> createMeetingNote({
    required String eventJson,
    required String context,
  }) => attempt(
    () => _library.overviewCreateMeetingNote(
      eventJson: eventJson,
      context: context,
    ),
  );

  @override
  Future<Either<ApplicationError, void>> linkMeetingNote({
    required String noteId,
    required String eventJson,
  }) => attempt(
    () => _library.overviewLinkMeetingNote(
      noteId: noteId,
      eventJson: eventJson,
    ),
  );

  @override
  Future<Either<ApplicationError, void>> hidePlace(String noteId) =>
      attempt(() => _library.overviewHidePlace(noteId: noteId));

  @override
  Future<Either<ApplicationError, void>> snoozeWaiting(String key) =>
      attempt(() => _library.overviewSnoozeWaiting(key: key));

  @override
  Future<Either<ApplicationError, void>> dismissPast(String noteId) =>
      attempt(() => _library.overviewDismissPast(noteId: noteId));

  @override
  List<RevisitOption> revisitOptions() => _library.revisitOptions();

  @override
  Future<Either<ApplicationError, String>> setRevisit({
    required String noteId,
    required RevisitChoice choice,
  }) => attempt(() => _library.setRevisit(noteId: noteId, choice: choice));

  @override
  Future<Either<ApplicationError, void>> clearRevisit(String noteId) =>
      attempt(() => _library.clearRevisit(noteId: noteId));

  @override
  Future<Either<ApplicationError, void>> noteWriting({
    required String noteId,
    required int line,
    required int column,
    required int typedChars,
    required String body,
  }) => attempt(
    () => _library.noteWriting(
      noteId: noteId,
      line: line,
      column: column,
      typedChars: typedChars,
      body: body,
    ),
  );

  @override
  Future<Either<ApplicationError, ReviewSaved>> saveReview({
    required String from,
    required bool replace,
  }) => attempt(() => _library.saveReview(from: from, replace: replace));

  @override
  Future<Either<ApplicationError, BriefPrefs>> prefs() =>
      attempt(_library.briefPrefs);

  @override
  Future<Either<ApplicationError, BriefPrefs>> setPrefs(BriefPrefs prefs) =>
      attempt(() => _library.setBriefPrefs(prefs: prefs));

  @override
  Future<Either<ApplicationError, List<PlannedNotice>>> plan({
    required bool canNotify,
  }) => attempt(() => _library.briefPlan(canNotify: canNotify));

  /// A fresh request to stop, for the AI call about to start.
  KansoAiRun _newRun() => _run = _ai.newRun();

  // The AI calls block on the network; the bridge runs them on its worker
  // threads, so the UI isolate only awaits.
  @override
  Future<Either<ApplicationError, LoopScan>> findLoops() =>
      attempt(() => _ai.findOpenLoops(run: _newRun()));

  @override
  Future<Either<ApplicationError, int>> keepLoops(
    List<LoopProposal> picks,
  ) => attempt(() => _library.approveAiLoops(picks: picks));

  @override
  Future<Either<ApplicationError, AiSummary>> summariseOverview({
    required bool week,
    String? from,
  }) => attempt(
    () => _ai.summariseOverview(from: from, week: week, run: _newRun()),
  );

  @override
  void stopAi() => _run?.stop();
}
