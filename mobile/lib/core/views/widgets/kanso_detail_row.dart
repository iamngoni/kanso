// Kanso — kanso_detail_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// A row of a grouped section with a [title] and a [subtitle] under it that
/// may take several lines, and what follows ([trailing]): a device with when
/// it was last seen, a passkey with when it was used, the storage used.
class KansoDetailRow extends StatelessWidget {
  const KansoDetailRow({
    required this.title,
    this.subtitle,
    this.icon,
    this.trailing,
    this.onTap,
    super.key,
  });

  final String title;
  final String? subtitle;
  final KansoIcons? icon;
  final Widget? trailing;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final glyph = icon;
    final note = subtitle;
    final end = trailing;
    return KansoPressable(
      onTap: onTap,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              if (glyph != null) ...<Widget>[
                KansoIcon(glyph, color: colors.textSecondary),
                const SizedBox(width: KansoSpacing.sm),
              ],
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(title, style: typography.body),
                    if (note != null) Text(note, style: typography.secondary),
                  ],
                ),
              ),
              if (end != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                end,
              ],
            ],
          ),
        ),
      ),
    );
  }
}
