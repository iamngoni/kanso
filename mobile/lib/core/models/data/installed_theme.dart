// Kanso — installed_theme.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart' show ThemeTokens;
import 'installed_theme_origin.dart';

part 'installed_theme.freezed.dart';

/// A theme the phone can draw in: one that comes with Kanso, one the person
/// imported, or one from the gallery.
///
/// [tokens] are the theme as its file says, not yet fitted to a syntax or a
/// preview theme (that is `ThemesRepository.fit`).
@freezed
abstract class InstalledTheme with _$InstalledTheme {
  const factory InstalledTheme({
    /// The theme file's name without `.json`: `nord`.
    required String id,

    /// The name people see: `Nord`.
    required String name,

    /// Who made the theme; empty when the file does not say.
    required String author,

    /// The theme's licence; empty when the file does not say.
    required String license,

    /// Whether the theme is for a dark interface.
    required bool dark,

    /// Where the theme came from.
    required InstalledThemeOrigin origin,

    /// The theme's colours.
    required ThemeTokens tokens,

    /// Where the theme was taken from, when it was.
    String? source,
  }) = _InstalledTheme;
}
