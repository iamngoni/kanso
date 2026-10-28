// Kanso — fixture_themes_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/icon_theme_import_result.dart';
import 'package:kanso/core/models/data/installed_icon_theme.dart';
import 'package:kanso/core/models/data/installed_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/core/models/data/theme_file_notice.dart';
import 'package:kanso/core/models/data/theme_import_result.dart';
import 'package:kanso/core/models/repos/abs/themes_repository.dart';

import 'fixture_renderer_repository.dart' show fixtureRuns;
import 'theme_tokens.dart';

/// The themes [FixtureThemesRepository] has unless a test gives others:
/// Kanso Light and Dark, built in.
List<InstalledTheme> defaultFixtureThemes() => <InstalledTheme>[
  for (final palette in <KansoPalette>[
    KansoPalette.kansoLight,
    KansoPalette.kansoDark,
  ])
    fixtureTheme(palette),
];

/// The installed theme that [palette] stands for, built in unless [origin]
/// says otherwise.
InstalledTheme fixtureTheme(
  KansoPalette palette, {
  InstalledThemeOrigin origin = InstalledThemeOrigin.builtin,
}) {
  return InstalledTheme(
    id: palette.id,
    name: palette.name,
    author: 'Kanso',
    license: 'MIT',
    dark: palette.dark,
    origin: origin,
    tokens: tokensFor(palette),
  );
}

/// A themes repository that does not need the Rust core: it holds the themes
/// it is given, and fits a combination by composing their tokens, the way
/// the desktop's `Tokens::fit` starts: the interface from the first theme,
/// the syntax colours from the second and the preview colours from the
/// third (the real fit also adjusts colours to stay readable, which tests of
/// the app do not depend on).
///
/// Imports succeed or fail as [importFailure] says, adding a theme named
/// after the file to [themes].
class FixtureThemesRepository implements ThemesRepository {
  FixtureThemesRepository({
    List<InstalledTheme>? themes,
    this.themeNotices = const <ThemeFileNotice>[],
    List<InstalledIconTheme>? iconThemeList,
    this.additionalThemes,
  }) : themes = themes ?? defaultFixtureThemes(),
       icons = iconThemeList ?? <InstalledIconTheme>[];

  /// The installed themes.
  final List<InstalledTheme> themes;

  /// More themes read from a separate fixture, as gallery themes are read
  /// from the gallery's folder by the Rust themes facade.
  final List<InstalledTheme> Function()? additionalThemes;

  /// The theme files that were left out.
  List<ThemeFileNotice> themeNotices;

  /// The imported icon themes.
  final List<InstalledIconTheme> icons;

  /// The combinations it was asked to fit, in order.
  final List<(String, String?, String?)> fits = <(String, String?, String?)>[];

  /// The paths it was asked to import as themes, in order.
  final List<String> importedThemePaths = <String>[];

  /// The paths it was asked to import as icon themes, in order.
  final List<String> importedIconThemePaths = <String>[];

  /// What draws diagrams.
  DiagramRenderer renderer = DiagramRenderer.pele;

  /// The diagram renderers it was told, in order.
  final List<DiagramRenderer> renderersSet = <DiagramRenderer>[];

  /// What it answers to an import instead of importing.
  ApplicationError? importFailure;

  /// When set, [list] waits for it, so a test can look at the screen while
  /// the themes are being read.
  Future<void>? listGate;

  /// What it answers to [list] instead of the themes.
  ApplicationError? listFailure;

  InstalledTheme? _find(String id) {
    for (final theme in <InstalledTheme>[
      ...themes,
      ...?additionalThemes?.call(),
    ]) {
      if (theme.id == id) {
        return theme;
      }
    }
    return null;
  }

  @override
  Future<Either<ApplicationError, List<InstalledTheme>>> list() async {
    await listGate;
    final error = listFailure;
    if (error != null) {
      return Left<ApplicationError, List<InstalledTheme>>(error);
    }
    return Right<ApplicationError, List<InstalledTheme>>(
      <InstalledTheme>[...themes, ...?additionalThemes?.call()],
    );
  }

  @override
  Future<Either<ApplicationError, List<ThemeFileNotice>>> notices() async =>
      Right<ApplicationError, List<ThemeFileNotice>>(themeNotices);

  @override
  Future<Either<ApplicationError, ThemeTokens>> fit(
    String ui,
    String? syntax,
    String? preview,
  ) async {
    fits.add((ui, syntax, preview));
    final interface = _find(ui);
    final source = _find(syntax ?? ui);
    final shown = _find(preview ?? ui);
    if (interface == null || source == null || shown == null) {
      return const Left<ApplicationError, ThemeTokens>(
        ApplicationError.notFound(message: 'No such theme.'),
      );
    }
    final base = interface.tokens;
    return Right<ApplicationError, ThemeTokens>(
      ThemeTokens(
        name: base.name,
        dark: base.dark,
        ui: base.ui,
        caret: base.caret,
        selection: base.selection,
        selectionAlpha: base.selectionAlpha,
        syntax: source.tokens.syntax,
        palette: shown.tokens.palette,
      ),
    );
  }

  @override
  Future<Either<ApplicationError, List<StyledRun>>> styledRuns(
    String text, {
    required int start,
    required int end,
    required String ui,
    String? syntax,
  }) async => Right<ApplicationError, List<StyledRun>>(
    fixtureRuns(text.substring(start, end), offset: start),
  );

  @override
  Future<Either<ApplicationError, ThemeImportResult>> importTheme(
    String path,
  ) async {
    importedThemePaths.add(path);
    final error = importFailure;
    if (error != null) {
      return Left<ApplicationError, ThemeImportResult>(error);
    }
    final id = _stem(path);
    themes.add(
      fixtureTheme(
        fakePalette(id, dark: true).copyWith(name: _title(id)),
        origin: InstalledThemeOrigin.own,
      ),
    );
    return Right<ApplicationError, ThemeImportResult>(
      ThemeImportResult(ids: <String>[id], names: <String>[_title(id)]),
    );
  }

  @override
  Future<Either<ApplicationError, List<InstalledIconTheme>>>
  iconThemes() async => Right<ApplicationError, List<InstalledIconTheme>>(
    List<InstalledIconTheme>.of(icons),
  );

  @override
  Future<Either<ApplicationError, IconThemeImportResult>> importIconTheme(
    String path,
  ) async {
    importedIconThemePaths.add(path);
    final error = importFailure;
    if (error != null) {
      return Left<ApplicationError, IconThemeImportResult>(error);
    }
    final id = _stem(path);
    icons.add(
      InstalledIconTheme(
        id: id,
        name: _title(id),
        author: 'Ana',
        license: 'MIT',
        origin: InstalledThemeOrigin.own,
      ),
    );
    return Right<ApplicationError, IconThemeImportResult>(
      IconThemeImportResult(
        ids: <String>[id],
        names: <String>[_title(id)],
        summary: '12 icons imported.',
      ),
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> setDiagramRenderer(
    DiagramRenderer renderer,
  ) async {
    this.renderer = renderer;
    renderersSet.add(renderer);
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, DiagramRenderer>> diagramRenderer() async =>
      Right<ApplicationError, DiagramRenderer>(renderer);
}

String _stem(String path) {
  final name = path.split('/').last;
  final dot = name.indexOf('.');
  return dot <= 0 ? name : name.substring(0, dot);
}

String _title(String id) => id.isEmpty
    ? id
    : '${id[0].toUpperCase()}${id.substring(1).replaceAll('-', ' ')}';
