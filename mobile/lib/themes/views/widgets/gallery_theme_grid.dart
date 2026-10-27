// Kanso — gallery_theme_grid.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../models/data/theme_preview.dart';
import '../../utils/theme_preview_from_gallery.dart';
import 'gallery_theme_card.dart';

/// The gallery's cards side by side as far as the width allows (one on a
/// phone, two or three on a tablet), each as wide as the room shared out.
///
/// [installed] has the previews of the installed themes, drawn from their
/// tokens, by id; a theme not in it is drawn from the catalog's colours.
class GalleryThemeGrid extends StatelessWidget {
  const GalleryThemeGrid({
    required this.items,
    required this.installed,
    required this.working,
    required this.onInstall,
    required this.onRemove,
    super.key,
  });

  final List<GalleryItem> items;
  final Map<String, ThemePreview> installed;

  /// The ids of the items being installed, updated or removed.
  final Set<String> working;
  final ValueChanged<String> onInstall;
  final ValueChanged<String> onRemove;

  /// The least width of a card before another column would be too narrow.
  static const double cardMinWidth = 300;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        const gap = KansoSpacing.sm;
        final columns = math.max(
          1,
          ((constraints.maxWidth + gap) / (cardMinWidth + gap)).floor(),
        );
        final width = (constraints.maxWidth - gap * (columns - 1)) / columns;
        return Wrap(
          spacing: gap,
          runSpacing: gap,
          children: <Widget>[
            for (final item in items)
              SizedBox(
                width: width,
                child: GalleryThemeCard(
                  key: ValueKey<String>(item.id),
                  item: item,
                  preview: installed[item.id] ?? themePreviewFromGallery(item),
                  working: working.contains(item.id),
                  onInstall: () => onInstall(item.id),
                  onRemove: () => onRemove(item.id),
                ),
              ),
          ],
        );
      },
    );
  }
}
