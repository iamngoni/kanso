// Kanso — publish_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/publish.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// The mobile publish operations over the native Rust facade.
abstract interface class PublishRepository {
  Future<Either<ApplicationError, List<PublishExpiryOption>>> expiryOptions();

  Future<Either<ApplicationError, PublishNoteInfo>> note(String id);

  Future<Either<ApplicationError, PublishNotebookInfo>> notebook(String id);

  Future<Either<ApplicationError, PublishLink>> publishNote({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  });

  Future<Either<ApplicationError, PublishLink>> publishNotebook({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  });

  Future<Either<ApplicationError, Unit>> unpublishNote(String id);

  Future<Either<ApplicationError, Unit>> unpublishNotebook(String id);
}
