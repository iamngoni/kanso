// Kanso — file_picker_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';

/// The system's file picker, so what needs a file from the phone does not
/// depend on the native sheet and a test can stand in for it.
// A repository of one call, like the others, so a test can replace it.
// ignore: one_member_abstracts
abstract interface class FilePickerRepository {
  /// Asks the person for one file of any kind and gives the path of a copy
  /// the app can read; null when they closed the picker without choosing.
  Future<Either<ApplicationError, String?>> pickFile();
}
