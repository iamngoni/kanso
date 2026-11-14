// Kanso — rust_embargo_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/embargo_types.dart';
import '../../../../core/bridge/api/library.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/embargo_repository.dart';

/// [EmbargoRepository] over the Rust core's library, at the phone's clock
/// and zone.
@LazySingleton(as: EmbargoRepository)
class RustEmbargoRepository implements EmbargoRepository {
  RustEmbargoRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, EmbargoView?>> embargo(String noteId) {
    final now = DateTime.now();
    return attempt(
      () => _library.embargo(
        noteId: noteId,
        nowUtcMs: now.millisecondsSinceEpoch,
        offsetMs: now.timeZoneOffset.inMilliseconds,
      ),
    );
  }
}
