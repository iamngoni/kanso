// Kanso — rust_digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/ai.dart';
import '../../../bridge/api/library.dart';
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
  Future<Either<ApplicationError, BriefView>> brief() =>
      attempt(_library.brief);

  @override
  Future<Either<ApplicationError, ReviewView>> review({String? from}) =>
      attempt(() => _library.review(from: from));

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
  Future<Either<ApplicationError, AiSummary>> summariseBrief() =>
      attempt(() => _ai.summariseBrief(run: _newRun()));

  @override
  Future<Either<ApplicationError, AiSummary>> summariseReview({
    String? from,
  }) => attempt(() => _ai.summariseReview(from: from, run: _newRun()));

  @override
  void stopAi() => _run?.stop();
}
