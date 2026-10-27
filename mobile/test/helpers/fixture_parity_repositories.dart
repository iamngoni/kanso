// Kanso — fixture_parity_repositories.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/backups/models/repos/abs/backups_repository.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/history/models/repos/abs/history_repository.dart';
import 'package:kanso/replace/models/repos/abs/replace_repository.dart';

/// Repeatable note-history, replacement and backup data for widget tests.
class FixtureHistoryRepository implements HistoryRepository {
  final List<(String, String)> restores = <(String, String)>[];

  @override
  Future<Either<ApplicationError, NoteHistory>> history(String id) async =>
      const Right<ApplicationError, NoteHistory>(
        NoteHistory(
          currentTitle: 'Field notes',
          currentBody: 'A changed line.\nA second line.',
          versions: <NoteVersion>[
            NoteVersion(
              rev: 'rev-old',
              savedAt: 1793000000000,
              origin: 'ui',
              originLabel: 'Edited in Kanso',
              title: 'Field notes',
              body: 'An earlier line.\nA second line.',
            ),
          ],
        ),
      );

  @override
  Future<Either<ApplicationError, List<DiffHunk>>> diff(
    String id,
    String rev,
  ) async => const Right<ApplicationError, List<DiffHunk>>(<DiffHunk>[
    DiffHunk(
      lines: <DiffLine>[
        DiffLine(
          kind: DiffKind.removed,
          text: 'An earlier line.',
          oldLine: 1,
          changed: <Range16>[],
        ),
        DiffLine(
          kind: DiffKind.added,
          text: 'A changed line.',
          newLine: 1,
          changed: <Range16>[],
        ),
      ],
      oldText: 'An earlier line.',
      added: 1,
      removed: 1,
      oldStart: 1,
      oldEnd: 1,
    ),
  ]);

  @override
  Future<Either<ApplicationError, Saved>> restore({
    required String id,
    required String rev,
    required String baseRev,
  }) async {
    restores.add((id, rev));
    return const Right<ApplicationError, Saved>(Saved(id: 'note', rev: 'r3'));
  }
}

/// A predictable replace preview and an undoable batch for widget tests.
class FixtureReplaceRepository implements ReplaceRepository {
  final List<PlannedNote> applied = <PlannedNote>[];
  final List<String> undone = <String>[];

  @override
  Future<Either<ApplicationError, ReplacePreview>> preview(
    ReplaceOptions options,
  ) async => Right<ApplicationError, ReplacePreview>(
    options.find.isEmpty
        ? const ReplacePreview(totalMatches: 0, notes: <ReplaceNote>[])
        : const ReplacePreview(
            totalMatches: 2,
            notes: <ReplaceNote>[
              ReplaceNote(
                id: 'note-one',
                title: 'Field notes',
                rev: 'r1',
                trashed: false,
                count: 2,
                matches: <ReplaceMatch>[
                  ReplaceMatch(
                    line: 'A needle in this line.',
                    startUtf16: 2,
                    endUtf16: 8,
                    lineNumber: 3,
                  ),
                  ReplaceMatch(
                    line: 'The needle is here.',
                    startUtf16: 4,
                    endUtf16: 10,
                    lineNumber: 8,
                  ),
                ],
              ),
            ],
          ),
  );

  @override
  Future<Either<ApplicationError, ReplaceOutcome>> apply({
    required ReplaceOptions options,
    required String replacement,
    required List<PlannedNote> planned,
  }) async {
    applied.addAll(planned);
    return const Right<ApplicationError, ReplaceOutcome>(
      ReplaceOutcome(
        batch: 'batch-1',
        replaced: 2,
        notes: 1,
        skipped: 0,
        message: 'Replaced 2 matches in 1 note.',
      ),
    );
  }

  @override
  Future<Either<ApplicationError, ReplaceUndone>> undo(String batch) async {
    undone.add(batch);
    return const Right<ApplicationError, ReplaceUndone>(
      ReplaceUndone(restored: 1, skipped: 0, message: 'Put back 1 note.'),
    );
  }
}

/// A mutable backup list for the Backups screen's widget tests.
class FixtureBackupsRepository implements BackupsRepository {
  FixtureBackupsRepository()
    : files = <BackupFile>[
        const BackupFile(
          path: '/library/backups/kanso-1793000000000.db',
          madeAtMs: 1793000000000,
          manual: false,
          bytes: 524288,
        ),
      ];

  final List<BackupFile> files;
  int dailyChecks = 0;
  int manualCount = 0;

  @override
  Future<Either<ApplicationError, BackupFile?>> backUpIfDue() async {
    dailyChecks++;
    return const Right<ApplicationError, BackupFile?>(null);
  }

  @override
  Future<Either<ApplicationError, BackupFile>> backUpNow() async {
    manualCount++;
    final file = BackupFile(
      path: '/library/backups/kanso-manual-$manualCount.db',
      madeAtMs: 1793000000000 + manualCount,
      manual: true,
      bytes: 1024,
    );
    files.insert(0, file);
    return Right<ApplicationError, BackupFile>(file);
  }

  @override
  Future<Either<ApplicationError, List<BackupFile>>> backups() async =>
      Right<ApplicationError, List<BackupFile>>(List<BackupFile>.of(files));

  @override
  Future<Either<ApplicationError, Unit>> delete(String path) async {
    files.removeWhere((file) => file.path == path);
    return const Right<ApplicationError, Unit>(unit);
  }
}
