// Kanso — theme_preview_from_installed.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../core/bridge/api/types.dart' show BuiltinTheme, SyntaxStyleRow;
import '../../core/models/data/installed_theme.dart';
import '../../core/models/data/installed_theme_origin.dart';
import '../../core/utils/palette_from_tokens.dart';
import '../models/data/theme_preview.dart';

/// The preview of [theme], drawn from the tokens the core fitted for it.
ThemePreview themePreviewFromInstalled(InstalledTheme theme) {
  final palette = paletteFromTokens(theme.id, theme.tokens);
  final syntax = palette.syntax;
  final styles = <String, SyntaxStyleRow>{
    for (final row in theme.tokens.syntax) row.token: row,
  };
  Color token(String name, Color fallback) {
    final color = styles[name]?.color;
    return color == null ? fallback : opaqueColor(color);
  }

  return ThemePreview(
    id: theme.id,
    name: theme.name,
    dark: theme.dark,
    editor: palette.colors.editor,
    text: palette.colors.text,
    accent: palette.colors.accent,
    heading: syntax.heading,
    marker: syntax.marker,
    quote: token('quote', syntax.marker),
    link: syntax.link,
    url: syntax.url,
    code: syntax.code,
    strong: token('strong', syntax.text),
    emphasis: token('emphasis', syntax.text),
    palette: palette,
    origin: theme.origin,
    author: theme.author,
  );
}

/// The preview of a built-in theme, or a gallery theme the older renderer
/// call returned, drawn from its fitted tokens.
ThemePreview themePreviewFromBuiltin(
  BuiltinTheme theme, {
  bool builtIn = true,
}) => themePreviewFromInstalled(
  InstalledTheme(
    id: theme.id,
    name: theme.name,
    author: theme.author,
    license: theme.license,
    dark: theme.dark,
    origin: builtIn
        ? InstalledThemeOrigin.builtin
        : InstalledThemeOrigin.gallery,
    tokens: theme.tokens,
    source: theme.source,
  ),
);
