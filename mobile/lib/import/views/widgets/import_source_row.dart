// Kanso: import_source_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// One place notes can be imported from: its name, what to export from it
/// and a chevron. Without [onTap] it is drawn dimmed and does nothing.
class ImportSourceRow extends StatelessWidget {
  const ImportSourceRow({required this.source, this.onTap, super.key});

  final ImportSource source;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final enabled = onTap != null;
    return KansoPressable(
      onTap: onTap,
      child: Opacity(
        opacity: enabled ? 1 : 0.5,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.sm,
            ),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(source.label, style: typography.body),
                      if (source.hint.isNotEmpty) ...<Widget>[
                        const SizedBox(height: KansoSpacing.xxs),
                        Text(source.hint, style: typography.meta),
                      ],
                    ],
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
                KansoIcon(
                  KansoIcons.chevronRight,
                  size: KansoSizes.iconSmall,
                  color: colors.textMuted,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
