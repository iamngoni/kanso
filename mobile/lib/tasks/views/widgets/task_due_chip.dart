// Kanso — task_due_chip.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/utils/context_extensions.dart';
import '../../utils/due_chip_colors.dart';

/// A task's due date in a small chip, coloured by how near it is.
class TaskDueChip extends StatelessWidget {
  const TaskDueChip({required this.label, required this.tone, super.key});

  final String label;
  final DueTone tone;

  @override
  Widget build(BuildContext context) {
    final (text, fill) = dueChipColors(tone, context.colors);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: fill,
        borderRadius: KansoRadii.smallBorder,
      ),
      child: Text(
        label,
        maxLines: 1,
        style: context.typography.meta.copyWith(
          fontWeight: FontWeight.w500,
          color: text,
          decoration: tone == DueTone.done ? TextDecoration.lineThrough : null,
          decorationColor: text,
        ),
      ),
    );
  }
}
