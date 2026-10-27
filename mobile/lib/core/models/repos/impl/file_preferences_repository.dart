// Kanso — file_preferences_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../data/application_error.dart';
import '../../data/library_folder.dart';
import '../../data/preferences.dart';
import '../abs/preferences_repository.dart';

/// The version of the file's layout, for a later change to read.
const int preferencesFileVersion = 1;

/// [PreferencesRepository] over `preferences.json` in the library folder.
///
/// A save writes a temporary file beside it and renames that over the
/// preferences, so a crash or a full disk leaves the old file whole.
@LazySingleton(as: PreferencesRepository)
class FilePreferencesRepository implements PreferencesRepository {
  FilePreferencesRepository(LibraryFolder folder)
    : _file = File(folder.preferencesPath);

  final File _file;
  Preferences? _current;
  Future<void> _queue = Future<void>.value();

  @override
  Future<Either<ApplicationError, Preferences>> load() {
    return _serialised(
      () async => Right<ApplicationError, Preferences>(await _read()),
    );
  }

  @override
  Future<Either<ApplicationError, Preferences>> update(
    Preferences Function(Preferences current) change,
  ) {
    return _serialised(() async {
      final next = change(await _read());
      try {
        await _write(next);
      } on FileSystemException catch (error) {
        return Left<ApplicationError, Preferences>(
          ApplicationError.storage(message: error.message),
        );
      }
      _current = next;
      return Right<ApplicationError, Preferences>(next);
    });
  }

  Future<T> _serialised<T>(Future<T> Function() action) {
    final result = _queue.then((_) => action());
    _queue = result.then<void>((_) {}, onError: (Object _) {});
    return result;
  }

  Future<Preferences> _read() async {
    final cached = _current;
    if (cached != null) {
      return cached;
    }
    final read = await _readFile();
    _current = read;
    return read;
  }

  Future<Preferences> _readFile() async {
    try {
      if (!_file.existsSync()) {
        return const Preferences();
      }
      final json = jsonDecode(await _file.readAsString());
      if (json is! Map<String, dynamic>) {
        return const Preferences();
      }
      return Preferences.fromJson(json);
    } on Object {
      // Unreadable or not ours: start again from the defaults. The next
      // change replaces the file.
      return const Preferences();
    }
  }

  Future<void> _write(Preferences preferences) async {
    await _file.parent.create(recursive: true);
    final temporary = File('${_file.path}.tmp');
    final json = <String, dynamic>{
      'version': preferencesFileVersion,
      ...preferences.toJson(),
    };
    await temporary.writeAsString(jsonEncode(json), flush: true);
    await temporary.rename(_file.path);
  }
}
