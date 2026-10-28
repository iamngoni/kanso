// Kanso — rust_publish_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/publish.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/publish_repository.dart';

/// Publish repository backed by the open native library.
@LazySingleton(as: PublishRepository)
class RustPublishRepository implements PublishRepository {
  RustPublishRepository(this._library);

  final KansoLibrary _library;

  Future<KansoPublisher> get _publisher => _library.publisher();

  @override
  Future<Either<ApplicationError, List<PublishExpiryOption>>> expiryOptions() =>
      attempt(KansoPublisher.expiryOptions);

  @override
  Future<Either<ApplicationError, PublishNoteInfo>> note(String id) =>
      attempt(() async => (await _publisher).note(id: id));

  @override
  Future<Either<ApplicationError, PublishNotebookInfo>> notebook(String id) =>
      attempt(() async => (await _publisher).notebook(id: id));

  @override
  Future<Either<ApplicationError, PublishLink>> publishNote({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  }) => attempt(
    () async => (await _publisher).publishNote(
      id: id,
      expiry: expiry,
      palette: palette,
      options: options,
    ),
  );

  @override
  Future<Either<ApplicationError, PublishLink>> publishNotebook({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  }) => attempt(
    () async => (await _publisher).publishNotebook(
      id: id,
      expiry: expiry,
      palette: palette,
      options: options,
    ),
  );

  @override
  Future<Either<ApplicationError, Unit>> unpublishNote(String id) =>
      attempt(() async {
        await (await _publisher).unpublishNote(id: id);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> unpublishNotebook(String id) =>
      attempt(() async {
        await (await _publisher).unpublishNotebook(id: id);
        return unit;
      });
}
