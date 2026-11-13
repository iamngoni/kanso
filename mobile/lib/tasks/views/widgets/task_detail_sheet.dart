// Kanso — task_detail_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/sheet_header.dart';
import 'task_due_chip.dart';
import 'task_repeat_chip.dart';

/// What a long press on a task shows: its whole text, which a row cuts at
/// three lines, with its due date, how it repeats (`Repeats every week`),
/// where it is, and a button to open the note.
class TaskDetailSheet extends StatelessWidget {
  const TaskDetailSheet({required this.task, required this.onOpen, super.key});

  /// Shows the sheet over [context]. [onOpen] opens the task's note, once
  /// the sheet has closed.
  static Future<void> show(
    BuildContext context, {
    required TaskItem task,
    required VoidCallback onOpen,
  }) => showKansoBottomSheet<void>(
    context,
    builder: (_) => TaskDetailSheet(task: task, onOpen: onOpen),
  );

  final TaskItem task;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final due = task.dueLabel;
    final tone = task.dueTone;
    final repeat = task.repeatLabel;
    return SafeArea(
      top: false,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SheetHeader(
            title: l10n.tasksDetailTitle,
            actionLabel: l10n.noteListDone,
          ),
          Flexible(
            child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.sm,
                KansoSpacing.md,
                KansoSpacing.sm,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(
                    task.text,
                    style: typography.body.copyWith(
                      color: task.checked ? colors.textMuted : colors.text,
                      decoration: task.checked
                          ? TextDecoration.lineThrough
                          : null,
                      decorationColor: colors.textMuted,
                    ),
                  ),
                  if ((due != null && tone != null) ||
                      repeat != null) ...<Widget>[
                    const SizedBox(height: KansoSpacing.sm),
                    Wrap(
                      spacing: KansoSpacing.xs,
                      runSpacing: KansoSpacing.xs,
                      children: <Widget>[
                        if (due != null && tone != null)
                          TaskDueChip(label: due, tone: tone),
                        if (repeat != null)
                          TaskRepeatChip(label: repeat, done: task.checked),
                      ],
                    ),
                  ],
                  if (repeat != null) ...<Widget>[
                    const SizedBox(height: KansoSpacing.xs),
                    Text(
                      l10n.tasksRepeatSemantics(repeat),
                      style: typography.meta.copyWith(
                        color: colors.textSecondary,
                      ),
                    ),
                  ],
                  const SizedBox(height: KansoSpacing.sm),
                  Text(
                    l10n.tasksRowContext(task.noteTitle, task.notebookName),
                    style: typography.meta.copyWith(color: colors.textMuted),
                  ),
                ],
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.md,
              KansoSpacing.md,
            ),
            child: KansoButton(
              label: l10n.tasksOpenNote,
              icon: KansoIcons.fileText,
              variant: KansoButtonVariant.secondary,
              onPressed: () {
                Navigator.of(context).pop();
                onOpen();
              },
            ),
          ),
        ],
      ),
    );
  }
}
