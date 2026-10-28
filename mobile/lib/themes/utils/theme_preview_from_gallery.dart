// Kanso — theme_preview_from_gallery.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../../core/bridge/api/types.dart';
import '../../core/utils/palette_from_tokens.dart';
import '../models/data/theme_preview.dart';

/// The preview of the gallery theme [item] before it is installed, drawn
/// from the five colours the signed catalog gives for its card; null for a
/// theme that has none (an icon theme).
///
/// The catalog has no syntax colours, so the lines of Markdown are drawn from
/// the ones it has: headings, links and code in the accent colour, and
/// markers and quotes as the text colour faded toward the editor's. Once the
/// theme is installed its card is drawn from its real tokens
/// (`themePreviewFromBuiltin`).
ThemePreview? themePreviewFromGallery(GalleryItem item) {
  final colors = item.headline;
  if (colors == null) {
    return null;
  }
  final editor = opaqueColor(colors.editor);
  final text = opaqueColor(colors.text);
  final accent = opaqueColor(colors.accent);
  Color faded(double toward) => Color.lerp(text, editor, toward)!;
  return ThemePreview(
    id: item.id,
    name: item.name,
    dark: item.dark,
    editor: editor,
    text: text,
    accent: accent,
    heading: accent,
    marker: faded(0.5),
    quote: faded(0.35),
    link: accent,
    url: Color.lerp(accent, editor, 0.3)!,
    code: accent,
    strong: text,
    emphasis: text,
    builtIn: false,
  );
}
