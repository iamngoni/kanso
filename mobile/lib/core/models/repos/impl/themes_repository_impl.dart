// Kanso — themes_repository_impl.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/renderer.dart';
import '../../../bridge/api/themes.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../../data/icon_theme_import_result.dart';
import '../../data/installed_icon_theme.dart';
import '../../data/installed_theme.dart';
import '../../data/installed_theme_origin.dart';
import '../../data/theme_file_notice.dart';
import '../../data/theme_import_result.dart';
import '../abs/themes_repository.dart';

/// [ThemesRepository] over the library's shared Rust theme facade.
@LazySingleton(as: ThemesRepository)
class ThemesRepositoryImpl implements ThemesRepository {
  ThemesRepositoryImpl(this._themes);

  final KansoThemes _themes;

  @override
  Future<Either<ApplicationError, List<InstalledTheme>>> list() =>
      attempt(() async => (await _themes.list()).map(_installed).toList());

  @override
  Future<Either<ApplicationError, List<ThemeFileNotice>>> notices() => attempt(
    () async => (await _themes.notices()).map(_notice).toList(growable: false),
  );

  @override
  Future<Either<ApplicationError, ThemeTokens>> fit(
    String ui,
    String? syntax,
    String? preview,
  ) => attempt(
    () => _themes.fit(ui: ui, syntax: syntax, preview: preview),
  );

  @override
  Future<Either<ApplicationError, List<StyledRun>>> styledRuns(
    String text, {
    required int start,
    required int end,
    required String ui,
    String? syntax,
  }) => attempt(
    () => _themes.styledRuns(
      text: text,
      start: start,
      end: end,
      ui: ui,
      syntax: syntax,
    ),
  );

  @override
  Future<Either<ApplicationError, ThemeImportResult>> importTheme(
    String path,
  ) => attempt(() async {
    final imported = await _themes.import_(path: path);
    return ThemeImportResult(ids: imported.ids, names: imported.names);
  });

  @override
  Future<Either<ApplicationError, List<InstalledIconTheme>>> iconThemes() =>
      attempt(
        () async => (await _themes.iconThemes())
            .map(_installedIconTheme)
            .toList(growable: false),
      );

  @override
  Future<Either<ApplicationError, IconThemeImportResult>> importIconTheme(
    String path,
  ) => attempt(() async {
    final imported = await _themes.importIconTheme(path: path);
    return IconThemeImportResult(
      ids: imported.ids,
      names: imported.names,
      summary: imported.summary,
    );
  });

  @override
  Future<Either<ApplicationError, Unit>> setDiagramRenderer(
    DiagramRenderer renderer,
  ) => attempt(() async {
    await KansoRenderer.setDiagramRenderer(renderer: renderer);
    return unit;
  });

  @override
  Future<Either<ApplicationError, DiagramRenderer>> diagramRenderer() =>
      attempt(KansoRenderer.diagramRenderer);

  InstalledTheme _installed(ThemeRow row) => InstalledTheme(
    id: row.id,
    name: row.name,
    author: row.author,
    license: row.license,
    dark: row.dark,
    origin: _origin(row.origin),
    tokens: row.tokens,
    source: row.source,
  );

  ThemeFileNotice _notice(ThemeNotice notice) => ThemeFileNotice(
    file: notice.file,
    sentence: notice.sentence,
  );

  InstalledIconTheme _installedIconTheme(InstalledIconThemeRow row) =>
      InstalledIconTheme(
        id: row.id,
        name: row.name,
        author: row.author,
        license: row.license,
        origin: _origin(row.origin),
      );

  InstalledThemeOrigin _origin(ThemeOrigin origin) => switch (origin) {
    ThemeOrigin.builtin => InstalledThemeOrigin.builtin,
    ThemeOrigin.own => InstalledThemeOrigin.own,
    ThemeOrigin.gallery => InstalledThemeOrigin.gallery,
  };
}
