// Kanso — themes_repository_impl.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/types.dart' show DiagramRenderer, ThemeTokens;
import '../../data/application_error.dart';
import '../../data/icon_theme_import_result.dart';
import '../../data/installed_icon_theme.dart';
import '../../data/installed_theme.dart';
import '../../data/theme_file_notice.dart';
import '../../data/theme_import_result.dart';
import '../abs/themes_repository.dart';

/// [ThemesRepository] over the Rust core's themes.
///
// TODO(iamngoni): wire this to the generated bridge. The core gives a `Themes`
/// handle from `Library.themes()` with `list`, `notices`, `fit(ui, syntax,
/// preview)`, `styledRuns`, `import(path)`, `iconThemes` and
/// `importIconTheme(path)`, and `Renderer.setDiagramRenderer`. Each method
/// below maps the bridge's types to this app's data classes (use `attempt`
/// from `kanso_error_mapping.dart` for the error mapping), and the
/// diagram renderer is remembered here until the core can say it.
@LazySingleton(as: ThemesRepository)
class ThemesRepositoryImpl implements ThemesRepository {
  ThemesRepositoryImpl();

  @override
  Future<Either<ApplicationError, List<InstalledTheme>>> list() =>
      throw UnimplementedError('Wire to Themes.list.');

  @override
  Future<Either<ApplicationError, List<ThemeFileNotice>>> notices() =>
      throw UnimplementedError('Wire to Themes.notices.');

  @override
  Future<Either<ApplicationError, ThemeTokens>> fit(
    String ui,
    String syntax,
    String preview,
  ) => throw UnimplementedError('Wire to Themes.fit.');

  @override
  Future<Either<ApplicationError, ThemeImportResult>> importTheme(
    String path,
  ) => throw UnimplementedError('Wire to Themes.import.');

  @override
  Future<Either<ApplicationError, List<InstalledIconTheme>>> iconThemes() =>
      throw UnimplementedError('Wire to Themes.iconThemes.');

  @override
  Future<Either<ApplicationError, IconThemeImportResult>> importIconTheme(
    String path,
  ) => throw UnimplementedError('Wire to Themes.importIconTheme.');

  @override
  Future<Either<ApplicationError, Unit>> setDiagramRenderer(
    DiagramRenderer renderer,
  ) => throw UnimplementedError('Wire to Renderer.setDiagramRenderer.');

  @override
  Future<Either<ApplicationError, DiagramRenderer>> diagramRenderer() =>
      throw UnimplementedError('Wire to Renderer.diagramRenderer.');
}
