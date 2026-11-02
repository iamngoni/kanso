// Kanso — rust_digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/digest_repository.dart';

/// [DigestRepository] over the Rust core's library.
@LazySingleton(as: DigestRepository)
class RustDigestRepository implements DigestRepository {
  RustDigestRepository(this._library);

  final KansoLibrary _library;

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
}
