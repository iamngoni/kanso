// Kanso: fixture_import_repositories.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart' show Fake;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/import/models/data/import_pick.dart';
import 'package:kanso/import/models/data/opened_batch.dart';
import 'package:kanso/import/models/repos/abs/import_files_repository.dart';
import 'package:kanso/import/models/repos/abs/import_repository.dart';

/// A readable export and the report used by import-flow widget tests.
const ImportPick sampleImportPick = ImportPick(
  path: '/cache/Export.zip',
  name: 'Export.zip',
);

/// The result the preview and the completed import show.
const ImportSummary sampleImportSummary = ImportSummary(
  kind: 'obsidian',
  dryRun: true,
  found: ImportFound(notes: 3, notebooks: 1, tags: 2, attachments: 1),
  empty: false,
  lines: <String>[
    '3 notes will be imported',
    '1 notebook will be created',
  ],
  skipped: <String>['1 unsupported file will be skipped'],
  rejected: <String>[],
  firstNote: 'first-note',
);

/// The import sources and answers a screen needs, with calls recorded.
class FixtureImportRepository extends Fake implements ImportRepository {
  /// Creates a repository that can preview and run an Obsidian export.
  FixtureImportRepository({
    this.planEvents = const <ImportEvent>[
      ImportEvent.done(sampleImportSummary),
    ],
    this.runEvents = const <ImportEvent>[
      ImportEvent.done(sampleImportSummary),
    ],
    this.kinds = const <String>['obsidian'],
    this.openedResult = const Right<ApplicationError, OpenedDocuments>(
      OpenedDocuments(problems: <String>[]),
    ),
  });

  /// Events returned while reading an export.
  List<ImportEvent> planEvents;

  /// Events returned while writing an export.
  List<ImportEvent> runEvents;

  /// Export kinds the given file is recognized as.
  List<String> kinds;

  /// The result of opening documents handed to Kanso by another app.
  Either<ApplicationError, OpenedDocuments> openedResult;

  /// Calls made to read and import an export.
  final List<String> calls = <String>[];

  @override
  List<ImportSource> sources() => const <ImportSource>[
    ImportSource(
      kind: 'obsidian',
      label: 'Obsidian',
      hint: 'An Obsidian export or vault',
      picksFiles: true,
      picksFolders: true,
      extensions: <String>['zip'],
    ),
    ImportSource(
      kind: 'markdown',
      label: 'Markdown',
      hint: 'Markdown files and folders',
      picksFiles: true,
      picksFolders: true,
      extensions: <String>['md', 'markdown', 'txt'],
    ),
  ];

  @override
  Either<ApplicationError, List<String>> kindsFor(String path) =>
      Right<ApplicationError, List<String>>(kinds);

  @override
  Stream<ImportEvent> plan(String kind, String path) {
    calls.add('plan $kind $path');
    return Stream<ImportEvent>.fromIterable(planEvents);
  }

  @override
  Stream<ImportEvent> run(String kind, String path) {
    calls.add('run $kind $path');
    return Stream<ImportEvent>.fromIterable(runEvents);
  }

  @override
  Future<Either<ApplicationError, OpenedDocuments>> openDocuments(
    List<OpenedFile> files,
    String staging,
  ) async {
    calls.add('openDocuments ${files.length}');
    return openedResult;
  }
}

/// A file picker that returns the selected export and records its request.
class FixtureImportFilesRepository extends Fake
    implements ImportFilesRepository {
  /// Creates a picker that returns [pick], or `null` when cancelled.
  FixtureImportFilesRepository({
    this.pick = sampleImportPick,
    this.openedBatch = const OpenedBatch(
      files: <OpenedFile>[],
      staging: '/cache/opened',
    ),
  });

  /// The selection the file and folder pickers return.
  ImportPick? pick;

  /// Documents waiting to be handed to the library.
  OpenedBatch openedBatch;

  /// Picker requests and cleanup calls made by the flow.
  final List<String> calls = <String>[];

  @override
  Stream<Unit> get arrivals => const Stream<Unit>.empty();

  @override
  Future<Either<ApplicationError, ImportPick?>> pickFile(
    List<String> extensions,
  ) async {
    calls.add('pickFile ${extensions.join(',')}');
    return Right<ApplicationError, ImportPick?>(pick);
  }

  @override
  Future<Either<ApplicationError, ImportPick?>> pickFolder() async {
    calls.add('pickFolder');
    return Right<ApplicationError, ImportPick?>(pick);
  }

  @override
  Future<Either<ApplicationError, Unit>> clearPicked() async {
    calls.add('clearPicked');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, OpenedBatch>> takeOpened() async {
    calls.add('takeOpened');
    return Right<ApplicationError, OpenedBatch>(openedBatch);
  }

  @override
  Future<Either<ApplicationError, Unit>> discardOpened() async {
    calls.add('discardOpened');
    return const Right<ApplicationError, Unit>(unit);
  }
}
