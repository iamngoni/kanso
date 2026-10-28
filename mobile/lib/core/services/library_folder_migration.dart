// Kanso — library_folder_migration.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:developer' as developer;
import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter/foundation.dart';
import 'package:injectable/injectable.dart';
import 'package:path/path.dart' as p;

import '../models/data/application_error.dart';
import '../models/data/folder_move.dart';
import '../models/data/library_folder.dart';

/// Moves the library from the app's own folder into the App Group container
/// on iOS, where the share extension and the App Intents can reach it.
///
/// Nothing is ever deleted before it is in the new place: an entry is
/// renamed, or, when the rename is refused, copied whole and only then
/// removed. The database's journal files go before the database, and the
/// database before the rest, so an interrupted move that is begun again
/// pairs them again. A name both places have is left where it is.
@lazySingleton
class LibraryFolderMigration {
  /// Moves the library in [from] into [to], if [from] has one and [to] does
  /// not. [to] is made if it is not there.
  Future<Either<ApplicationError, FolderMoveOutcome>> move({
    required String from,
    required String to,
  }) async {
    try {
      final decision = decideFolderMove(
        currentHasLibrary: File(p.join(to, libraryFileName)).existsSync(),
        oldHasLibrary: File(p.join(from, libraryFileName)).existsSync(),
      );
      switch (decision) {
        case FolderMoveDecision.nothing:
          return const Right<ApplicationError, FolderMoveOutcome>(
            FolderMoveOutcome.nothingToMove,
          );
        case FolderMoveDecision.keepCurrent:
          developer.log(
            'Both $from and $to have a library; $to is used and $from '
            'is left as it is.',
            name: 'kanso.library_folder',
          );
          return const Right<ApplicationError, FolderMoveOutcome>(
            FolderMoveOutcome.keptCurrent,
          );
        case FolderMoveDecision.move:
          final copied = await _moveInto(Directory(from), Directory(to));
          return Right<ApplicationError, FolderMoveOutcome>(
            copied ? FolderMoveOutcome.copied : FolderMoveOutcome.moved,
          );
      }
    } on FileSystemException catch (error) {
      return Left<ApplicationError, FolderMoveOutcome>(
        ApplicationError.storage(message: error.message),
      );
    }
  }

  /// Renames [entity] to [newPath]. Overridden by tests to refuse.
  @visibleForTesting
  Future<void> renameEntity(FileSystemEntity entity, String newPath) async {
    await entity.rename(newPath);
  }

  /// Moves what is in [source] into [target], and removes [source] once it
  /// is empty. Whether anything had to be copied.
  Future<bool> _moveInto(Directory source, Directory target) async {
    await target.create(recursive: true);
    final entries = await source.list(followLinks: false).toList();
    entries.sort(
      (a, b) => _rank(p.basename(a.path)).compareTo(_rank(p.basename(b.path))),
    );
    var copied = false;
    for (final entry in entries) {
      final destination = p.join(target.path, p.basename(entry.path));
      final taken = FileSystemEntity.typeSync(destination, followLinks: false);
      if (taken == FileSystemEntityType.notFound) {
        copied |= await _place(entry, destination);
      } else if (entry is Directory &&
          taken == FileSystemEntityType.directory) {
        copied |= await _moveInto(entry, Directory(destination));
      } else {
        developer.log(
          '$destination is there already; ${entry.path} is left as it is.',
          name: 'kanso.library_folder',
        );
      }
    }
    if (source.listSync().isEmpty) {
      await source.delete();
    }
    return copied;
  }

  /// The order entries move in: the journal files, the database, the rest.
  int _rank(String name) {
    if (name == '$libraryFileName-wal' ||
        name == '$libraryFileName-shm' ||
        name == '$libraryFileName-journal') {
      return 0;
    }
    return name == libraryFileName ? 1 : 2;
  }

  /// Puts [entry] at [destination], which is free. Whether it was copied.
  Future<bool> _place(FileSystemEntity entry, String destination) async {
    try {
      await renameEntity(entry, destination);
      return false;
    } on FileSystemException catch (error) {
      developer.log(
        'Renaming ${entry.path} was refused (${error.message}); '
        'copying it.',
        name: 'kanso.library_folder',
      );
    }
    await _copy(entry, destination);
    await _remove(entry);
    return true;
  }

  Future<void> _copy(FileSystemEntity entry, String destination) async {
    if (entry is Directory) {
      await Directory(destination).create(recursive: true);
      await for (final child in entry.list(followLinks: false)) {
        await _copy(child, p.join(destination, p.basename(child.path)));
      }
    } else if (entry is Link) {
      await Link(destination).create(await entry.target());
    } else if (entry is File) {
      // Copied under another name and renamed, so a crash never leaves half
      // a file where the whole one is looked for.
      final partial = '$destination.moving';
      await entry.copy(partial);
      await File(partial).rename(destination);
    }
  }

  Future<void> _remove(FileSystemEntity entry) async {
    await entry.delete(recursive: true);
  }
}
