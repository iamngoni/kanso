// Kanso — theme_preview_from_installed.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/models/data/installed_theme.dart';
import '../../core/utils/palette_from_tokens.dart';
import '../models/data/theme_preview.dart';

/// The preview of the installed [theme]: the colours of its tokens, as the
/// app would draw a few lines of Markdown, or a rendered note, in it.
ThemePreview themePreviewFromInstalled(InstalledTheme theme) {
  final palette = paletteFromTokens(theme.id, theme.tokens);
  final colors = palette.colors;
  final syntax = palette.syntax;
  final rows = <String, int?>{
    for (final row in theme.tokens.syntax) row.token: row.color,
  };
  final quote = rows['quote'];
  final strong = rows['strong'];
  final emphasis = rows['emphasis'];
  return ThemePreview(
    id: theme.id,
    name: theme.name,
    dark: theme.dark,
    editor: colors.editor,
    text: colors.text,
    accent: colors.accent,
    heading: syntax.heading,
    marker: syntax.marker,
    quote: quote == null ? syntax.marker : opaqueColor(quote),
    link: syntax.link,
    url: syntax.url,
    code: syntax.code,
    strong: strong == null ? colors.text : opaqueColor(strong),
    emphasis: emphasis == null ? colors.text : opaqueColor(emphasis),
    origin: theme.origin,
    author: theme.author,
    palette: palette,
  );
}
