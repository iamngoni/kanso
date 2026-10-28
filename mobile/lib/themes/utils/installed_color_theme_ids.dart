// Kanso — installed_color_theme_ids.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';

/// The ids of the colour themes in [items] that are installed and drawn:
/// the ones the picker's Installed row lists. A theme the catalog withdrew
/// after it was edited stays on the phone but is not drawn, so it is not
/// among them; neither is an icon theme.
Set<String> installedColorThemeIds(List<GalleryItem> items) {
  return <String>{
    for (final item in items)
      if (item.installed &&
          !item.disabled &&
          item.kind == GalleryKind.colorTheme)
        item.id,
  };
}
