// Kanso — theme_preview_from_gallery.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../../core/bridge/api/types.dart';
import '../../core/configs/kanso_palette.dart';
import '../../core/models/data/installed_theme_origin.dart';
import '../../core/utils/palette_from_tokens.dart';
import '../models/data/theme_preview.dart';

/// The preview of a gallery theme before it is installed, drawn from the
/// five colours the signed catalog gives for its card; null for an icon
/// theme.
///
/// The catalog has no syntax colours, so the lines of Markdown use its
/// accent and text colours. Once installed, its card uses the theme's tokens.
ThemePreview? themePreviewFromGallery(GalleryItem item) {
  final colors = item.headline;
  if (colors == null) {
    return null;
  }
  final base = item.dark ? KansoPalette.kansoDark : KansoPalette.kansoLight;
  final editor = opaqueColor(colors.editor);
  final text = opaqueColor(colors.text);
  final accent = opaqueColor(colors.accent);
  final sidebar = opaqueColor(colors.sidebar);
  final list = opaqueColor(colors.list);
  final marker = Color.lerp(text, editor, 0.5)!;
  final url = Color.lerp(accent, editor, 0.3)!;
  int rgb(Color color) => color.toARGB32() & 0x00FFFFFF;
  final palette = base.copyWith(
    id: item.id,
    name: item.name,
    dark: item.dark,
    colors: base.colors.copyWith(
      editor: editor,
      text: text,
      accent: accent,
      link: accent,
      sidebar: sidebar,
      list: list,
    ),
    syntax: base.syntax.copyWith(
      text: text,
      heading: accent,
      blockMarker: marker,
      marker: marker,
      listMarker: marker,
      task: marker,
      hr: marker,
      link: accent,
      url: url,
      code: accent,
    ),
    preview: PaletteDto(
      dark: item.dark,
      background: colors.editor,
      text: colors.text,
      muted: rgb(marker),
      accent: colors.accent,
      border: base.preview.border,
      surface: colors.list,
      heading: colors.accent,
      quote: rgb(marker),
      taskDone: rgb(marker),
      codeText: colors.text,
      inlineCodeBackground: colors.list,
      inlineCodeText: colors.text,
      tableHeaderBackground: colors.sidebar,
    ),
  );
  return ThemePreview(
    id: item.id,
    name: item.name,
    dark: item.dark,
    editor: editor,
    text: text,
    accent: accent,
    heading: accent,
    marker: marker,
    quote: marker,
    link: accent,
    url: url,
    code: accent,
    strong: text,
    emphasis: text,
    palette: palette,
    origin: InstalledThemeOrigin.gallery,
    author: item.author,
  );
}
