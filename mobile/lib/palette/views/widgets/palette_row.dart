// Kanso — palette_row.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/linking_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_themed_icon.dart';
import '../../models/data/palette_icon_names.dart';

/// One thing the palette found: what it is, what it does or where it is
/// ([PaletteEntry.hint]), and whether it is already on ([PaletteEntry.checked])
/// or done ([PaletteEntry.status]).
///
/// The entry's depth steps the label in, for the places that nest.
class PaletteRow extends StatelessWidget {
  const PaletteRow({required this.entry, required this.onTap, super.key});

  final PaletteEntry entry;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final hint = entry.hint;
    final status = entry.status;
    final indent = KansoSpacing.md + entry.depth * KansoSpacing.sm;
    return KansoPressable(
      onTap: onTap,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
        child: Padding(
          padding: EdgeInsetsDirectional.only(
            start: indent,
            end: KansoSpacing.md,
            top: KansoSpacing.xs,
            bottom: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              KansoThemedIcon(
                lucideName: paletteIconName(entry.icon),
                fallback: paletteFallbackIcon,
                size: 18,
                color: colors.textSecondary,
              ),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: <Widget>[
                    Text(
                      entry.label,
                      style: typography.body.copyWith(color: colors.text),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    if (hint != null)
                      Text(
                        hint,
                        style: typography.meta.copyWith(
                          color: colors.textMuted,
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                  ],
                ),
              ),
              if (status != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.sm),
                Text(
                  status,
                  style: typography.meta.copyWith(color: colors.textMuted),
                  maxLines: 1,
                ),
              ],
              if (entry.checked) ...<Widget>[
                const SizedBox(width: KansoSpacing.sm),
                KansoIcon(KansoIcons.check, size: 16, color: colors.accent),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
