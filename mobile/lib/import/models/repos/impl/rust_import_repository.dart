// Kanso: rust_import_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/import.dart' as bridge;
import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/cancelable_stream.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/import_repository.dart';

/// [ImportRepository] over the Rust core's importers.
@LazySingleton(as: ImportRepository)
class RustImportRepository implements ImportRepository {
  RustImportRepository(this._library);

  final KansoLibrary _library;

  @override
  List<ImportSource> sources() => bridge.importSources();

  @override
  Either<ApplicationError, List<String>> kindsFor(String path) =>
      attemptSync(() => bridge.importKindsFor(path: path));

  @override
  Stream<ImportEvent> plan(String kind, String path) =>
      cancelableStream(_library.importPlan(kind: kind, path: path));

  @override
  Stream<ImportEvent> run(String kind, String path) =>
      cancelableStream(_library.import_(kind: kind, path: path));

  @override
  Future<Either<ApplicationError, OpenedDocuments>> openDocuments(
    List<OpenedFile> files,
    String staging,
  ) => attempt(() => _library.openDocuments(files: files, staging: staging));
}
