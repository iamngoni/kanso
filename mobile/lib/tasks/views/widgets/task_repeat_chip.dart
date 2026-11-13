// Kanso — task_repeat_chip.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';

/// A repeating task's repeat in a small quiet chip, beside its due date:
/// a loop mark and the words as the desktop has them (`every week`).
class TaskRepeatChip extends StatelessWidget {
  const TaskRepeatChip({required this.label, this.done = false, super.key});

  final String label;

  /// Whether the task is done: the chip is drawn muted.
  final bool done;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final ink = done ? colors.textMuted : colors.textSecondary;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: colors.fill.withValues(alpha: 0.7),
        borderRadius: KansoRadii.smallBorder,
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          KansoIcon(KansoIcons.refreshCw, size: KansoSizes.glyph, color: ink),
          const SizedBox(width: 3),
          Flexible(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.meta.copyWith(
                fontWeight: FontWeight.w500,
                color: ink,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
