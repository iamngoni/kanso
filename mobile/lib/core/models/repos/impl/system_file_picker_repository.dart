// Kanso — system_file_picker_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:file_picker/file_picker.dart';
import 'package:injectable/injectable.dart';

import '../../data/application_error.dart';
import '../abs/file_picker_repository.dart';

/// [FilePickerRepository] over the system's document picker (the Files sheet
/// on iOS, the storage access framework on Android).
///
/// Any file can be chosen: `.tgz` and `.json` have no type that iOS and
/// Android agree on, so the importer is the one to say what it cannot read.
@LazySingleton(as: FilePickerRepository)
class SystemFilePickerRepository implements FilePickerRepository {
  @override
  Future<Either<ApplicationError, String?>> pickFile() async {
    try {
      final picked = await FilePicker.pickFile();
      return Right<ApplicationError, String?>(picked?.path);
    } on Exception catch (error) {
      return Left<ApplicationError, String?>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
