// Kanso — fixture_file_picker_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/file_picker_repository.dart';

/// A file picker that chooses what the test says, without a native sheet:
/// the next of the paths it was given each time (null for a picker closed
/// without a choice), or [failure].
class FixtureFilePickerRepository implements FilePickerRepository {
  FixtureFilePickerRepository([Iterable<String?> paths = const <String?>[]])
    : _paths = List<String?>.of(paths);

  final List<String?> _paths;

  /// What it answers instead of a path.
  ApplicationError? failure;

  /// How many times it was asked.
  int asked = 0;

  /// Makes the next ask choose [path].
  void willPick(String? path) => _paths.add(path);

  @override
  Future<Either<ApplicationError, String?>> pickFile() async {
    asked++;
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, String?>(error);
    }
    return Right<ApplicationError, String?>(
      _paths.isEmpty ? null : _paths.removeAt(0),
    );
  }
}
