// Kanso — real_themes.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart' show setUpAll;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/installed_icon_theme.dart';
import 'package:kanso/core/models/data/installed_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/core/models/data/theme_file_notice.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';

import 'core_library.dart';
import 'fixture_themes_repository.dart';

/// The built-in themes as the Rust core gives them, in the desktop's order.
/// Valid after [loadRealThemes] has run in a `setUpAll`.
late List<BuiltinTheme> realThemes;

/// The built-in themes by their id.
Map<String, BuiltinTheme> get realThemesById => <String, BuiltinTheme>{
  for (final theme in realThemes) theme.id: theme,
};

/// The built-in [theme] as the themes repository lists it.
InstalledTheme installedFromBuiltin(BuiltinTheme theme) => InstalledTheme(
  id: theme.id,
  name: theme.name,
  author: theme.author,
  license: theme.license,
  source: theme.source,
  dark: theme.dark,
  origin: InstalledThemeOrigin.builtin,
  tokens: theme.tokens,
);

/// A themes repository over the real built-in themes, without a library
/// open in the test (see [FixtureThemesRepository]).
FixtureThemesRepository realThemesRepository({
  List<ThemeFileNotice> notices = const <ThemeFileNotice>[],
  List<InstalledIconTheme> iconThemes = const <InstalledIconTheme>[],
}) => FixtureThemesRepository(
  themes: <InstalledTheme>[...realThemes.map(installedFromBuiltin)],
  themeNotices: notices,
  iconThemeList: List<InstalledIconTheme>.of(iconThemes),
);

/// The real built-in themes with two more among the first, so the pictures
/// of the theme picker show every origin: a copy of the built-in theme
/// [own] that the person made and one of [gallery] from the gallery.
FixtureThemesRepository showcaseThemesRepository({
  String own = 'nord',
  String gallery = 'dracula',
  List<ThemeFileNotice> notices = const <ThemeFileNotice>[],
  List<InstalledIconTheme> iconThemes = const <InstalledIconTheme>[],
}) {
  final built = realThemes.map(installedFromBuiltin).toList();
  InstalledTheme copy(String id, InstalledThemeOrigin origin, String name) {
    final source = built.firstWhere((theme) => theme.id == id);
    return source.copyWith(
      id: '$id-copy',
      name: name,
      origin: origin,
      tokens: ThemeTokens(
        name: name,
        dark: source.tokens.dark,
        ui: source.tokens.ui,
        caret: source.tokens.caret,
        selection: source.tokens.selection,
        selectionAlpha: source.tokens.selectionAlpha,
        syntax: source.tokens.syntax,
        palette: source.tokens.palette,
      ),
    );
  }

  return FixtureThemesRepository(
    themes: <InstalledTheme>[
      ...built.take(2),
      copy(own, InstalledThemeOrigin.own, 'My Nord'),
      copy(gallery, InstalledThemeOrigin.gallery, 'Dracula Plus'),
      ...built.skip(2),
    ],
    themeNotices: notices,
    iconThemeList: List<InstalledIconTheme>.of(iconThemes),
  );
}

/// The palette of the real theme [id], from its tokens.
KansoPalette realPalette(String id) {
  final theme = realThemesById[id]!;
  return paletteFromTokens(theme.id, theme.tokens);
}

/// Opens the Rust core and reads the built-in themes, once for the whole
/// test file. Call it in `main`, not inside a test: the core's calls need
/// the real clock a `testWidgets` body does not have.
void loadRealThemes() {
  setUpAll(() async {
    await initCoreForTests();
    final library = await openTestLibrary();
    final themes = await RustRendererRepository(
      library.renderer(),
    ).builtinThemes();
    realThemes = themes.getOrElse(
      () => throw StateError('The built-in themes did not load.'),
    );
  });
}
