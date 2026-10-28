// Kanso — backups_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// The library's database backups, kept inside the library's own folder (so
/// on a phone they are out of the device's backups with the rest of it).
abstract interface class BackupsRepository {
  /// Makes the daily backup when the newest is more than a day old, and
  /// drops the daily ones beyond the last 14; the ones made on request stay.
  /// The new backup, or null when one was made within the day.
  Future<Either<ApplicationError, BackupFile?>> backUpIfDue();

  /// Makes a backup now. It stays until it is [delete]d.
  Future<Either<ApplicationError, BackupFile>> backUpNow();

  /// The backups, newest first.
  Future<Either<ApplicationError, List<BackupFile>>> backups();

  /// Deletes the backup made on request at [path], as [backups] gave it. The
  /// daily ones, which the schedule keeps, and anything else are an
  /// [InvalidError].
  Future<Either<ApplicationError, Unit>> delete(String path);
}
