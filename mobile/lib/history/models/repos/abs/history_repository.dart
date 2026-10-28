// Kanso — history_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// A note's saved versions: what the Rust core keeps of its past.
///
/// A locked note's versions are sealed. While the note is closed none of
/// these reads anything and each is a [LockedError]; while it is open the
/// versions its session can open come back, and no sealed text ever does.
abstract interface class HistoryRepository {
  /// The versions of the note [id], newest first, beside the note as it is
  /// now.
  Future<Either<ApplicationError, NoteHistory>> history(String id);

  /// What changed from the version [rev] to the note now, section by
  /// section; none when they are the same. A [NotFoundError] when that
  /// version is not kept.
  Future<Either<ApplicationError, List<DiffHunk>>> diff(String id, String rev);

  /// Sets the note's title and text to those of the version [rev]. The note
  /// must still be at [baseRev] (a [ConflictError] when it is not), and the
  /// text it has now stays in its history.
  Future<Either<ApplicationError, Saved>> restore({
    required String id,
    required String rev,
    required String baseRev,
  });
}
