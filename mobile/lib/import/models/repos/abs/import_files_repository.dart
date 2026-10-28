// Kanso: import_files_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/import_pick.dart';
import '../../data/opened_batch.dart';

/// The phone's side of getting files: the system's pickers, and the
/// documents other apps open in Kanso.
abstract interface class ImportFilesRepository {
  /// Asks the person for a file (one of [extensions], without the dot, when
  /// the list is not empty) and copies it into the app's cache. `null` when
  /// they cancel.
  Future<Either<ApplicationError, ImportPick?>> pickFile(
    List<String> extensions,
  );

  /// Asks the person for a folder and copies it, with what is in it, into
  /// the app's cache. `null` when they cancel.
  Future<Either<ApplicationError, ImportPick?>> pickFolder();

  /// Removes what the pickers copied into the cache.
  Future<Either<ApplicationError, Unit>> clearPicked();

  /// The documents the system opened in the app since the last call, and
  /// forgets them.
  Future<Either<ApplicationError, OpenedBatch>> takeOpened();

  /// Removes the copies [takeOpened] made, once they are read.
  Future<Either<ApplicationError, Unit>> discardOpened();

  /// An event each time the system opens another document in the app.
  Stream<Unit> get arrivals;
}
