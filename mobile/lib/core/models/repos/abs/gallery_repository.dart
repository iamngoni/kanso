// Kanso — gallery_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The theme gallery: the community's colour and icon themes, and the ones
/// installed from it. The index is verified by the Rust core exactly as the
/// desktop does before anything is installed.
abstract interface class GalleryRepository {
  /// The gallery's items. Without [refresh] the cached index (the last one
  /// fetched) answers, with no network; with it the index is fetched again.
  Future<Either<ApplicationError, List<GalleryItem>>> index({
    bool refresh = false,
  });

  /// Installs the item [id]; the items now.
  Future<Either<ApplicationError, List<GalleryItem>>> install(String id);

  /// Removes the installed item [id]; the items now.
  Future<Either<ApplicationError, List<GalleryItem>>> uninstall(String id);

  /// The installed colour themes, fitted for drawing, for the picker.
  Future<Either<ApplicationError, List<BuiltinTheme>>> installedThemes();

  /// The ids of the installed colour themes that change: removed, withdrawn
  /// by the catalog when the index is fetched, installed, or installed again
  /// in new colours. Whoever shows a theme listens, so a change reaches it
  /// wherever it was made.
  Stream<String> themeChanges();

  /// The tokens of the installed colour theme [id].
  Future<Either<ApplicationError, ThemeTokens>> installedThemeTokens(String id);
}
