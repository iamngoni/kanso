// Kanso — rust_replace_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/replace_repository.dart';

/// [ReplaceRepository] over the Rust core's library.
@LazySingleton(as: ReplaceRepository)
class RustReplaceRepository implements ReplaceRepository {
  RustReplaceRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, ReplacePreview>> preview(
    ReplaceOptions options,
  ) => attempt(() => _library.replacePreview(options: options));

  @override
  Future<Either<ApplicationError, ReplaceOutcome>> apply({
    required ReplaceOptions options,
    required String replacement,
    required List<PlannedNote> planned,
  }) => attempt(
    () => _library.replaceApply(
      options: options,
      replacement: replacement,
      planned: planned,
    ),
  );

  @override
  Future<Either<ApplicationError, ReplaceUndone>> undo(String batch) =>
      attempt(() => _library.replaceUndo(batch: batch));
}
