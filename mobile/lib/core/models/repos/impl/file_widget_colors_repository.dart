// Kanso — file_widget_colors_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../configs/kanso_colors.dart';
import '../../../services/widget_reloader.dart';
import '../../../utils/widget_colors_json.dart';
import '../../data/application_error.dart';
import '../../data/library_folder.dart';
import '../abs/widget_colors_repository.dart';

/// [WidgetColorsRepository] over `widget.json` in the library folder.
///
/// The file is written to a temporary one that is renamed over it, so a
/// widget reading it never sees half a file.
@LazySingleton(as: WidgetColorsRepository)
class FileWidgetColorsRepository implements WidgetColorsRepository {
  FileWidgetColorsRepository(LibraryFolder folder, this._reloader)
    : _file = File(folder.widgetColorsPath);

  final File _file;
  final WidgetReloader _reloader;

  @override
  Future<Either<ApplicationError, bool>> publish({
    required KansoColors light,
    required KansoColors dark,
  }) async {
    final text = widgetColorsJson(light: light, dark: dark);
    try {
      if (_file.existsSync() && await _file.readAsString() == text) {
        return const Right<ApplicationError, bool>(false);
      }
      final temporary = File('${_file.path}.tmp');
      await temporary.writeAsString(text, flush: true);
      await temporary.rename(_file.path);
      await _reloader.reload();
      return const Right<ApplicationError, bool>(true);
    } on FileSystemException catch (error) {
      return Left<ApplicationError, bool>(
        ApplicationError.storage(message: error.message),
      );
    } on Object catch (error) {
      return Left<ApplicationError, bool>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
