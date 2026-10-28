// Kanso — properties_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The row of a note's properties above its text, with how many there are and
/// a chevron that opens them (and turns down while they are open).
class PropertiesRow extends StatelessWidget {
  const PropertiesRow({
    required this.count,
    this.open = false,
    this.onTap,
    super.key,
  });

  final int count;

  /// Whether the properties are shown.
  final bool open;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: onTap == null
          ? null
          : open
          ? context.l10n.propertiesHide
          : context.l10n.propertiesShow,
      child: SizedBox(
        height: KansoSizes.touchTarget,
        child: Row(
          children: <Widget>[
            Text(
              context.l10n.editProperties(count),
              style: context.typography.label,
            ),
            const SizedBox(width: KansoSpacing.xxs),
            KansoIcon(
              open ? KansoIcons.chevronDown : KansoIcons.chevronRight,
              size: KansoSizes.iconSmall,
              color: colors.textMuted,
            ),
          ],
        ),
      ),
    );
  }
}
