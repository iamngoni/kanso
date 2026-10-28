// Kanso — gallery_loading_grid.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// Where the gallery's themes will be while the index is read: still cards.
class GalleryLoadingGrid extends StatelessWidget {
  const GalleryLoadingGrid({super.key});

  /// How many cards are drawn.
  static const int cards = 2;

  /// How tall each is.
  static const double cardHeight = 190;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      label: context.l10n.galleryLoading,
      container: true,
      child: Column(
        children: <Widget>[
          for (var i = 0; i < cards; i++) ...<Widget>[
            Container(
              height: cardHeight,
              padding: const EdgeInsets.all(KansoSpacing.sm),
              decoration: BoxDecoration(
                color: colors.elevated,
                borderRadius: KansoRadii.largeBorder,
                border: Border.all(color: colors.divider),
              ),
              child: const Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  KansoSkeleton(height: 10),
                  SizedBox(height: KansoSpacing.xs),
                  KansoSkeleton(width: 90, height: 10),
                  SizedBox(height: KansoSpacing.xs),
                  KansoSkeleton(width: 60, height: 10),
                  Spacer(),
                  KansoSkeleton(width: 120),
                  SizedBox(height: KansoSpacing.xs),
                  KansoSkeleton(width: 80),
                ],
              ),
            ),
            if (i < cards - 1) const SizedBox(height: KansoSpacing.sm),
          ],
        ],
      ),
    );
  }
}
