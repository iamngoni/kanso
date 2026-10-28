// Kanso: import_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// Importing into the open library, and opening the documents the system
/// hands over.
///
/// The streams do not fail: the library reports a failure as an
/// [ImportEvent.failed], and a stream ends after its last event.
abstract interface class ImportRepository {
  /// Where a phone can import from, in the order to list them.
  List<ImportSource> sources();

  /// What the file or folder at [path] could be, best first, as the `kind`
  /// names the import takes. Empty when it is none of them.
  Either<ApplicationError, List<String>> kindsFor(String path);

  /// What importing [path] as a [kind] would do: progress, then one summary
  /// (with `dryRun` set) or one failure. Nothing is written.
  Stream<ImportEvent> plan(String kind, String path);

  /// Imports [path] as a [kind]: progress, then one summary or one failure.
  /// The notes written reach the library's changes like any other.
  Stream<ImportEvent> run(String kind, String path);

  /// Opens the Markdown or text [files] into the Inbox. [staging] is where
  /// the library keeps its working copies of the ones that name a document.
  Future<Either<ApplicationError, OpenedDocuments>> openDocuments(
    List<OpenedFile> files,
    String staging,
  );
}
