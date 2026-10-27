// Kanso — gallery_theme_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/theme_preview.dart';
import '../../utils/gallery_source_label.dart';
import 'theme_code_preview.dart';

/// A theme in the gallery: a preview of Markdown in its colours (or, for an
/// icon theme, a tile), its name with a badge for who made it, its author,
/// version, installs (when the catalog counts them) and source, and the one
/// action that fits: Install, Update, or Installed with a way to remove it.
class GalleryThemeCard extends StatelessWidget {
  const GalleryThemeCard({
    required this.item,
    required this.working,
    required this.onInstall,
    required this.onRemove,
    this.preview,
    super.key,
  });

  final GalleryItem item;

  /// The theme in its colours: from the catalog's headline colours, or from
  /// its tokens once it is installed. Null for an icon theme.
  final ThemePreview? preview;

  /// An install, update or removal of this theme is running.
  final bool working;

  /// What Install and Update do.
  final VoidCallback onInstall;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final typography = context.typography;
    final shown = preview;
    final source = item.source;
    final facts = <String>[
      l10n.galleryBy(item.author),
      l10n.galleryVersion(item.version),
      if (item.installs > 0) l10n.galleryInstalls(item.installs),
    ].join(' · ');
    final String primaryLabel;
    final bool primaryEnabled;
    if (working) {
      primaryLabel = item.installed
          ? (item.updateAvailable ? l10n.galleryUpdating : l10n.galleryRemoving)
          : l10n.galleryInstalling;
      primaryEnabled = false;
    } else if (!item.runsHere && !item.installed) {
      primaryLabel = l10n.galleryNeedsNewer;
      primaryEnabled = false;
    } else if (item.installed && item.updateAvailable) {
      primaryLabel = l10n.galleryUpdate;
      primaryEnabled = true;
    } else if (item.installed) {
      primaryLabel = l10n.galleryInstalled;
      primaryEnabled = false;
    } else {
      primaryLabel = l10n.galleryInstall;
      primaryEnabled = true;
    }
    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: colors.divider),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          if (shown != null)
            ThemeCodePreview(theme: shown)
          else
            Container(
              height: 96,
              alignment: Alignment.center,
              color: colors.fill,
              child: KansoIcon(
                KansoIcons.shapes,
                size: KansoSizes.icon * 2,
                color: colors.textSecondary,
              ),
            ),
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                Row(
                  children: <Widget>[
                    Expanded(
                      child: Text(
                        item.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.listTitle,
                      ),
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    KansoBadge(
                      label: item.official
                          ? l10n.galleryBadgeOfficial
                          : l10n.galleryBadgeCommunity,
                      tone: item.official
                          ? KansoBadgeTone.accent
                          : KansoBadgeTone.neutral,
                    ),
                  ],
                ),
                if (item.kind == GalleryKind.iconTheme || item.disabled)
                  Padding(
                    padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                    child: Wrap(
                      spacing: KansoSpacing.xs,
                      children: <Widget>[
                        if (item.kind == GalleryKind.iconTheme)
                          KansoBadge(label: l10n.galleryBadgeIcons),
                        if (item.disabled)
                          KansoBadge(label: l10n.galleryBadgeWithdrawn),
                      ],
                    ),
                  ),
                const SizedBox(height: KansoSpacing.xxs),
                Text(
                  facts,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: typography.meta,
                ),
                if (source != null)
                  Text(
                    gallerySourceLabel(source),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.meta.copyWith(color: colors.textMuted),
                  ),
                const SizedBox(height: KansoSpacing.sm),
                Row(
                  children: <Widget>[
                    if (!item.disabled)
                      Expanded(
                        child: KansoButton(
                          label: primaryLabel,
                          variant: primaryEnabled
                              ? KansoButtonVariant.primary
                              : KansoButtonVariant.secondary,
                          icon: item.installed && !primaryEnabled && !working
                              ? KansoIcons.check
                              : null,
                          compact: true,
                          onPressed: primaryEnabled ? onInstall : null,
                        ),
                      ),
                    if (item.installed) ...<Widget>[
                      if (!item.disabled)
                        const SizedBox(width: KansoSpacing.xs),
                      KansoButton(
                        label: l10n.galleryRemove,
                        variant: KansoButtonVariant.quiet,
                        expanded: false,
                        compact: true,
                        onPressed: working ? null : onRemove,
                      ),
                    ],
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
