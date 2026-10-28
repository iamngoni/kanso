// Kanso — library_folder.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:path/path.dart' as p;

part 'library_folder.freezed.dart';

/// The file name of the library's database.
const String libraryFileName = 'library.db';

/// The file name of the preferences, beside the database.
const String preferencesFileName = 'preferences.json';

/// The name of the folder shared items are left in, beside the database.
const String inboxFolderName = 'inbox';

/// The file the widgets read their colours from, beside the database.
const String widgetColorsFileName = 'widget.json';

/// The folder the library lives in: its database, the preferences, the
/// folder of shared items waiting to be saved and the widgets' colours.
///
/// It is in the app's private storage, and kept out of device backups.
@freezed
abstract class LibraryFolder with _$LibraryFolder {
  const factory LibraryFolder(String path) = _LibraryFolder;

  const LibraryFolder._();

  /// The library's database.
  String get databasePath => p.join(path, libraryFileName);

  /// The preferences file.
  String get preferencesPath => p.join(path, preferencesFileName);

  /// The folder the share extension, the shortcuts and the share activity
  /// leave shared items in (`mobile/docs/capture.md`).
  String get inboxPath => p.join(path, inboxFolderName);

  /// The widgets' colours.
  String get widgetColorsPath => p.join(path, widgetColorsFileName);
}
