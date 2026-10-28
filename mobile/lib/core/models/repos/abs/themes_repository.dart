// Kanso — themes_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart'
    show DiagramRenderer, StyledRun, ThemeTokens;
import '../../data/application_error.dart';
import '../../data/icon_theme_import_result.dart';
import '../../data/installed_icon_theme.dart';
import '../../data/installed_theme.dart';
import '../../data/theme_file_notice.dart';
import '../../data/theme_import_result.dart';

/// The themes the Rust core knows: the ones that come with Kanso, the ones
/// in the themes folder and the gallery's; fitting a combination of them;
/// importing more; the icon themes; and what draws diagrams, which the core
/// is told when it changes and at start.
abstract interface class ThemesRepository {
  /// Every installed theme, the built-in ones first in the desktop's order,
  /// then the person's own and the gallery's.
  Future<Either<ApplicationError, List<InstalledTheme>>> list();

  /// The theme files that were left out and why, one per file.
  Future<Either<ApplicationError, List<ThemeFileNotice>>> notices();

  /// The tokens to draw with when the interface is in the theme [ui], the
  /// Markdown source in [syntax] and the rendered note in [preview]: the
  /// desktop's `Tokens::fit`, which adjusts the colours of the three
  /// together so text stays readable on the surfaces it lands on.
  Future<Either<ApplicationError, ThemeTokens>> fit(
    String ui,
    String? syntax,
    String? preview,
  );

  /// The source runs styled with the fitted UI and syntax themes.
  Future<Either<ApplicationError, List<StyledRun>>> styledRuns(
    String text, {
    required int start,
    required int end,
    required String ui,
    String? syntax,
  });

  /// Imports the theme package (a folder or `.tgz`) or Zed theme (`.json`)
  /// at [path] into the themes folder.
  Future<Either<ApplicationError, ThemeImportResult>> importTheme(String path);

  /// Every icon theme that was imported or installed from the gallery.
  Future<Either<ApplicationError, List<InstalledIconTheme>>> iconThemes();

  /// Imports the icon theme at [path] (a Zed extension folder or archive, a
  /// theme package carrying one, or a Kanso icon theme folder).
  Future<Either<ApplicationError, IconThemeImportResult>> importIconTheme(
    String path,
  );

  /// Sets what draws Mermaid diagrams, here and in exports.
  Future<Either<ApplicationError, Unit>> setDiagramRenderer(
    DiagramRenderer renderer,
  );

  /// What draws Mermaid diagrams now.
  Future<Either<ApplicationError, DiagramRenderer>> diagramRenderer();
}
