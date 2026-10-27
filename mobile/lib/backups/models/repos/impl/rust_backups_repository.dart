// Kanso — rust_backups_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/backups_repository.dart';

/// [BackupsRepository] over the Rust core's library.
@LazySingleton(as: BackupsRepository)
class RustBackupsRepository implements BackupsRepository {
  RustBackupsRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, BackupFile?>> backUpIfDue() =>
      attempt(_library.backupIfDue);

  @override
  Future<Either<ApplicationError, BackupFile>> backUpNow() =>
      attempt(_library.backUpNow);

  @override
  Future<Either<ApplicationError, List<BackupFile>>> backups() =>
      attempt(_library.backups);

  @override
  Future<Either<ApplicationError, Unit>> delete(String path) =>
      attempt(() async {
        await _library.deleteBackup(path: path);
        return unit;
      });
}
