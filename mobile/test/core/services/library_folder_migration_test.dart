// Kanso — library_folder_migration_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show BoolAny, Glados2, any;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/folder_move.dart';
import 'package:kanso/core/services/library_folder_migration.dart';
import 'package:path/path.dart' as p;

import '../../helpers/property.dart';

/// A migration whose renames are refused, as when two folders are on
/// different volumes.
class _RefusingMigration extends LibraryFolderMigration {
  int refused = 0;

  @override
  Future<void> renameEntity(FileSystemEntity entity, String newPath) async {
    refused++;
    throw FileSystemException('Cross-device link.', newPath);
  }
}

void main() {
  late Directory root;
  late String old;
  late String current;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('kanso_move_');
    old = p.join(root.path, 'old', 'kanso');
    current = p.join(root.path, 'group', 'kanso');
  });

  tearDown(() => root.delete(recursive: true));

  void write(String folder, String name, String text) {
    final file = File(p.join(folder, name));
    file.parent.createSync(recursive: true);
    file.writeAsStringSync(text);
  }

  String? read(String folder, String name) {
    final file = File(p.join(folder, name));
    return file.existsSync() ? file.readAsStringSync() : null;
  }

  void seedLibrary(String folder, String tag) {
    write(folder, 'library.db', 'db $tag');
    write(folder, 'library.db-wal', 'wal $tag');
    write(folder, 'library.db-shm', 'shm $tag');
    write(folder, 'preferences.json', 'prefs $tag');
    write(folder, 'inbox/abc/drop.json', 'drop $tag');
  }

  FolderMoveOutcome outcomeOf(Either<ApplicationError, FolderMoveOutcome> r) =>
      r.getOrElse(() => throw StateError('failed: $r'));

  group('decideFolderMove', () {
    test('is a table of four cases', () {
      FolderMoveDecision decide({required bool current, required bool old}) =>
          decideFolderMove(currentHasLibrary: current, oldHasLibrary: old);
      expect(decide(current: false, old: true), FolderMoveDecision.move);
      expect(decide(current: true, old: true), FolderMoveDecision.keepCurrent);
      expect(decide(current: true, old: false), FolderMoveDecision.nothing);
      expect(decide(current: false, old: false), FolderMoveDecision.nothing);
    });

    Glados2<bool, bool>(any.bool, any.bool, kansoExploreConfig()).test(
      'moves only a library that has nowhere to go, and never replaces one',
      (currentHas, oldHas) {
        final decision = decideFolderMove(
          currentHasLibrary: currentHas,
          oldHasLibrary: oldHas,
        );
        expect(
          decision == FolderMoveDecision.move,
          oldHas && !currentHas,
        );
        if (currentHas) {
          expect(decision, isNot(FolderMoveDecision.move));
        }
        if (!oldHas) {
          expect(decision, FolderMoveDecision.nothing);
        }
      },
    );
  });

  group('LibraryFolderMigration', () {
    test('moves everything of the old folder into a new place', () async {
      seedLibrary(old, 'old');
      final outcome = outcomeOf(
        await LibraryFolderMigration().move(from: old, to: current),
      );
      expect(outcome, FolderMoveOutcome.moved);
      for (final name in <String>[
        'library.db',
        'library.db-wal',
        'library.db-shm',
        'preferences.json',
        'inbox/abc/drop.json',
      ]) {
        expect(read(current, name), isNotNull, reason: name);
        expect(read(old, name), isNull, reason: name);
      }
      expect(read(current, 'library.db'), 'db old');
      expect(read(current, 'library.db-wal'), 'wal old');
      // The emptied folder goes.
      expect(Directory(old).existsSync(), isFalse);
    });

    test(
      'moves into a folder the group already has, without a library',
      () async {
        seedLibrary(old, 'old');
        write(current, 'widget.json', 'colors');
        write(current, 'inbox/def/drop.json', 'waiting');
        final outcome = outcomeOf(
          await LibraryFolderMigration().move(from: old, to: current),
        );
        expect(outcome, FolderMoveOutcome.moved);
        expect(read(current, 'library.db'), 'db old');
        expect(read(current, 'widget.json'), 'colors');
        // Both inboxes' drops are there.
        expect(read(current, 'inbox/def/drop.json'), 'waiting');
        expect(read(current, 'inbox/abc/drop.json'), 'drop old');
        expect(Directory(old).existsSync(), isFalse);
      },
    );

    test(
      'with both libraries, keeps the new and leaves the old as it was',
      () async {
        seedLibrary(old, 'old');
        seedLibrary(current, 'new');
        final outcome = outcomeOf(
          await LibraryFolderMigration().move(from: old, to: current),
        );
        expect(outcome, FolderMoveOutcome.keptCurrent);
        expect(read(current, 'library.db'), 'db new');
        expect(read(old, 'library.db'), 'db old');
        expect(read(old, 'library.db-wal'), 'wal old');
        expect(read(old, 'preferences.json'), 'prefs old');
      },
    );

    test('with only the new library, does nothing', () async {
      seedLibrary(current, 'new');
      final outcome = outcomeOf(
        await LibraryFolderMigration().move(from: old, to: current),
      );
      expect(outcome, FolderMoveOutcome.nothingToMove);
      expect(read(current, 'library.db'), 'db new');
      expect(Directory(old).existsSync(), isFalse);
    });

    test('with neither, does nothing and makes nothing', () async {
      final outcome = outcomeOf(
        await LibraryFolderMigration().move(from: old, to: current),
      );
      expect(outcome, FolderMoveOutcome.nothingToMove);
      expect(Directory(current).existsSync(), isFalse);
    });

    test('an old folder without a library is left alone', () async {
      write(old, 'preferences.json', 'prefs old');
      final outcome = outcomeOf(
        await LibraryFolderMigration().move(from: old, to: current),
      );
      expect(outcome, FolderMoveOutcome.nothingToMove);
      expect(read(old, 'preferences.json'), 'prefs old');
    });

    test('when renaming is refused, copies and then removes the old', () async {
      seedLibrary(old, 'old');
      final migration = _RefusingMigration();
      final outcome = outcomeOf(await migration.move(from: old, to: current));
      expect(outcome, FolderMoveOutcome.copied);
      expect(migration.refused, greaterThan(0));
      expect(read(current, 'library.db'), 'db old');
      expect(read(current, 'library.db-wal'), 'wal old');
      expect(read(current, 'preferences.json'), 'prefs old');
      expect(read(current, 'inbox/abc/drop.json'), 'drop old');
      expect(Directory(old).existsSync(), isFalse);
      // No half-copied file is left under its working name.
      final leftovers = Directory(current)
          .listSync(recursive: true)
          .where((entity) => entity.path.endsWith('.moving'));
      expect(leftovers, isEmpty);
    });

    test('a name both places have is left in the old folder', () async {
      seedLibrary(old, 'old');
      write(current, 'preferences.json', 'prefs new');
      final outcome = outcomeOf(
        await LibraryFolderMigration().move(from: old, to: current),
      );
      expect(outcome, FolderMoveOutcome.moved);
      expect(read(current, 'preferences.json'), 'prefs new');
      expect(read(old, 'preferences.json'), 'prefs old');
      expect(read(current, 'library.db'), 'db old');
    });

    test('moving again after it was done changes nothing', () async {
      seedLibrary(old, 'old');
      final migration = LibraryFolderMigration();
      await migration.move(from: old, to: current);
      final again = outcomeOf(await migration.move(from: old, to: current));
      expect(again, FolderMoveOutcome.nothingToMove);
      expect(read(current, 'library.db'), 'db old');
    });

    test('a move that began and stopped is finished by the next one', () async {
      // The journal files went first, then the process stopped.
      seedLibrary(old, 'old');
      File(p.join(current, 'library.db-wal'))
        ..parent.createSync(recursive: true)
        ..writeAsStringSync('wal old');
      File(p.join(old, 'library.db-wal')).deleteSync();
      File(p.join(old, 'library.db-shm')).deleteSync();
      final outcome = outcomeOf(
        await LibraryFolderMigration().move(from: old, to: current),
      );
      expect(outcome, FolderMoveOutcome.moved);
      expect(read(current, 'library.db'), 'db old');
      expect(read(current, 'library.db-wal'), 'wal old');
    });
  });
}
