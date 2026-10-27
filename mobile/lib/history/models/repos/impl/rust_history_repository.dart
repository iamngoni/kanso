// Kanso — rust_history_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/history_repository.dart';

/// [HistoryRepository] over the Rust core's library.
@LazySingleton(as: HistoryRepository)
class RustHistoryRepository implements HistoryRepository {
  RustHistoryRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, NoteHistory>> history(String id) =>
      attempt(() => _library.history(id: id));

  @override
  Future<Either<ApplicationError, List<DiffHunk>>> diff(
    String id,
    String rev,
  ) => attempt(() => _library.historyDiff(id: id, rev: rev));

  @override
  Future<Either<ApplicationError, Saved>> restore({
    required String id,
    required String rev,
    required String baseRev,
  }) => attempt(
    () => _library.restoreVersion(id: id, rev: rev, baseRev: baseRev),
  );
}
