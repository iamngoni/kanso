// Kanso — core_module.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:developer' as developer;
import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:image_picker/image_picker.dart';
import 'package:injectable/injectable.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:path_provider_foundation/path_provider_foundation.dart';

import '../bridge/api/account.dart';
import '../bridge/api/account_settings.dart';
import '../bridge/api/approvals.dart';
import '../bridge/api/gallery.dart';
import '../bridge/api/library.dart';
import '../bridge/api/locks.dart';
import '../bridge/api/renderer.dart';
import '../bridge/api/shared.dart';
import '../bridge/api/sync.dart';
import '../bridge/api/themes.dart';
import '../models/data/application_error.dart';
import '../models/data/folder_move.dart';
import '../models/data/library_folder.dart';
import '../services/library_folder_migration.dart';

/// Where the library lives: `kanso/` in the app's support folder
/// (`filesDir` on Android, the App Group container's
/// `Library/Application Support` on iOS), with `library.db` and
/// `preferences.json` in it.
///
/// The background sync opens the same file, so changing this changes it
/// there too.
const String libraryFolderName = 'kanso';

/// The App Group the app, its share extension and its App Intents share on
/// iOS. The library lives in its container so all of them can open it.
const String appGroupIdentifier = 'group.app.kanso.Kanso';

/// Opens the library and hands out its parts.
@module
abstract class CoreModule {
  /// The person's library folder, made if it is not there.
  @Environment('development')
  @Environment('production')
  @singleton
  @preResolve
  Future<LibraryFolder> libraryFolder(LibraryFolderMigration migration) async {
    final support = await getApplicationSupportDirectory();
    final own = p.join(support.path, libraryFolderName);
    var path = own;
    if (Platform.isIOS) {
      final container = await PathProviderFoundation().getContainerPath(
        appGroupIdentifier: appGroupIdentifier,
      );
      if (container == null) {
        // Without the group (an unsigned build) the app keeps its own
        // folder; the extension and the shortcuts cannot reach it.
        developer.log(
          'The App Group $appGroupIdentifier is not available.',
          name: 'kanso.library_folder',
        );
      } else {
        final shared = p.join(
          container,
          'Library',
          'Application Support',
          libraryFolderName,
        );
        final moved = await migration.move(from: own, to: shared);
        path = _afterMove(moved, own: own, shared: shared);
      }
    }
    await Directory(path).create(recursive: true);
    return LibraryFolder(path);
  }

  /// The folder to open once a move has been tried: the group's, unless the
  /// move failed before the library got there, when the app's own folder
  /// still has it and the move is tried again at the next start.
  String _afterMove(
    Either<ApplicationError, FolderMoveOutcome> moved, {
    required String own,
    required String shared,
  }) {
    return moved.fold((error) {
      developer.log(
        'Moving the library failed: ${error.message}',
        name: 'kanso.library_folder',
      );
      final there = File(p.join(shared, libraryFileName)).existsSync();
      return there ? shared : own;
    }, (_) => shared);
  }

  /// A new, empty folder for an integration test.
  @Environment('integration')
  @singleton
  @preResolve
  Future<LibraryFolder> temporaryLibraryFolder() async {
    final root = await getTemporaryDirectory();
    final folder = await root.createTemp('kanso_integration_');
    return LibraryFolder(folder.path);
  }

  /// The open library. `KansoCore.init` must have run first.
  @singleton
  @preResolve
  Future<KansoLibrary> library(LibraryFolder folder) =>
      KansoLibrary.open(path: folder.databasePath);

  @lazySingleton
  KansoLocks locks(KansoLibrary library) => library.locks();

  @lazySingleton
  KansoSync sync(KansoLibrary library) => library.syncing();

  @lazySingleton
  KansoAccount account(KansoLibrary library) => library.account();

  @lazySingleton
  KansoAccountSettings accountSettings(KansoLibrary library) =>
      library.accountSettings();

  @lazySingleton
  KansoApprovals approvals(KansoLibrary library) => library.approvals();

  @lazySingleton
  KansoRenderer renderer(KansoLibrary library) => library.renderer();

  @lazySingleton
  KansoThemes themes(KansoLibrary library) => library.themes();

  @lazySingleton
  KansoShared shared(KansoLibrary library) => library.shared();

  @lazySingleton
  KansoGallery gallery(KansoLibrary library) => library.gallery();

  @lazySingleton
  ImagePicker imagePicker() => ImagePicker();
}
