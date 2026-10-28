// Kanso — real_icon_themes.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart' show Fake, setUpAll;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/kanso_icon_set.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';

import 'core_library.dart';
import 'fixture_renderer_repository.dart';

/// The built-in icon themes as the Rust core gives them, in the desktop's
/// order. Valid after [loadRealIconThemes] has run in a `setUpAll`.
late List<IconThemeRow> realIconThemes;

/// Every icon of each theme, by the theme's ID, as the core gives them.
late Map<String, List<ThemedIcon>> realIconThemeIcons;

/// The icon theme [id] as the app keeps it, from the core's SVGs.
KansoIconSet realIconSet(String id) {
  final row = realIconThemes.firstWhere((theme) => theme.id == id);
  return KansoIconSet(
    id: id,
    name: row.name,
    svgs: <String, String>{
      for (final icon in realIconThemeIcons[id]!)
        if (icon.svg != null) icon.name: icon.svg!,
    },
  );
}

/// Opens the Rust core and reads the built-in icon themes and their icons,
/// once for the whole test file. Call it in `main`, not inside a test.
void loadRealIconThemes() {
  setUpAll(() async {
    await initCoreForTests();
    final library = await openTestLibrary();
    final renderer = RustRendererRepository(library.renderer());
    realIconThemes = (await renderer.iconThemes()).getOrElse(
      () => throw StateError('The built-in icon themes did not load.'),
    );
    realIconThemeIcons = <String, List<ThemedIcon>>{
      for (final theme in realIconThemes)
        theme.id: (await renderer.iconThemeIcons(theme.id)).getOrElse(
          () => throw StateError('The icons of ${theme.id} did not load.'),
        ),
    };
  });
}

/// A renderer that answers the icon theme calls from the core's real themes,
/// so a bloc built on it picks icon themes without a library open.
class RealIconThemesRenderer extends Fake implements RendererRepository {
  @override
  Future<Either<ApplicationError, List<IconThemeRow>>> iconThemes() async =>
      Right<ApplicationError, List<IconThemeRow>>(realIconThemes);

  @override
  Future<Either<ApplicationError, List<ThemedIcon>>> iconThemeIcons(
    String id,
  ) async {
    final icons = realIconThemeIcons[id];
    return icons == null
        ? const Left<ApplicationError, List<ThemedIcon>>(
            ApplicationError.notFound(message: 'No such icon theme.'),
          )
        : Right<ApplicationError, List<ThemedIcon>>(icons);
  }
}

/// The fixture renderer of the page tests, with the core's real icon themes
/// in place of its two.
class RealIconThemesFixtureRenderer extends FixtureRendererRepository {
  final RealIconThemesRenderer _real = RealIconThemesRenderer();

  @override
  Future<Either<ApplicationError, List<IconThemeRow>>> iconThemes() async {
    await iconThemesGate?.future;
    return iconThemesResult ?? await _real.iconThemes();
  }

  @override
  Future<Either<ApplicationError, List<ThemedIcon>>> iconThemeIcons(
    String id,
  ) => _real.iconThemeIcons(id);
}
