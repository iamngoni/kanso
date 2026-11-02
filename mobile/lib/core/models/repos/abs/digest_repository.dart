// Kanso — digest_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The Rust core's open loops, morning brief and weekly review.
///
/// Locked notes and notes in locked notebooks are in none of them.
abstract interface class DigestRepository {
  /// The open loops across the notes, newest notes first.
  Future<Either<ApplicationError, OpenLoopsBoard>> openLoops();

  /// Closes a loop; false when it is gone from its note already.
  Future<Either<ApplicationError, bool>> closeLoop({
    required OpenLoopItem item,
    required LoopAction action,
  });

  /// The morning brief, worked out now.
  Future<Either<ApplicationError, BriefView>> brief();

  /// The review of the week holding [from] (`YYYY-MM-DD`), or this week.
  Future<Either<ApplicationError, ReviewView>> review({String? from});

  /// Saves the review of the week from [from] as a Journal note.
  Future<Either<ApplicationError, ReviewSaved>> saveReview({
    required String from,
    required bool replace,
  });

  /// The brief and review settings.
  Future<Either<ApplicationError, BriefPrefs>> prefs();

  /// Saves the settings; what was kept.
  Future<Either<ApplicationError, BriefPrefs>> setPrefs(BriefPrefs prefs);

  /// The notifications to hand the system for the next week.
  Future<Either<ApplicationError, List<PlannedNotice>>> plan({
    required bool canNotify,
  });
}
