// Kanso — task_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import 'task_due_chip.dart';

/// A task: its box, its text (struck through and muted once done), its due
/// date and, quietly under them, the note and notebook it is in.
///
/// One node for a screen reader, which reads the task, says whether it is
/// done and can open the note, tick the task off or clear it. Pressing the
/// box ticks; pressing the rest opens the note. [open] is the task whose
/// note is open beside the list, [focused] the one the keyboard is on.
class TaskRow extends StatelessWidget {
  const TaskRow({
    required this.task,
    required this.onTick,
    required this.onOpen,
    this.open = false,
    this.focused = false,
    super.key,
  });

  final TaskItem task;
  final VoidCallback onTick;
  final VoidCallback onOpen;
  final bool open;
  final bool focused;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final due = task.dueLabel;
    final tone = task.dueTone;
    final whereLine = l10n.tasksRowContext(task.noteTitle, task.notebookName);
    final spoken = <String>[
      task.text,
      if (due != null) l10n.tasksRowDue(due),
      whereLine,
    ].join(', ');
    return Semantics(
      container: true,
      label: spoken,
      checked: task.checked,
      focused: focused,
      onTap: onOpen,
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        CustomSemanticsAction(
          label: task.checked ? l10n.tasksTickBack : l10n.tasksTickOff,
        ): onTick,
        CustomSemanticsAction(label: l10n.tasksOpenNote): onOpen,
      },
      child: ExcludeSemantics(
        child: Container(
          color: open ? colors.rowActive : (focused ? colors.rowHover : null),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              // The box has a full touch target of its own.
              KansoPressable(
                onTap: onTick,
                child: Padding(
                  padding: const EdgeInsetsDirectional.only(
                    start: KansoSpacing.md,
                    end: KansoSpacing.sm,
                    top: KansoSpacing.sm,
                    bottom: KansoSpacing.sm,
                  ),
                  child: IgnorePointer(
                    child: KansoCheckbox(value: task.checked),
                  ),
                ),
              ),
              Expanded(
                child: KansoPressable(
                  onTap: onOpen,
                  child: Padding(
                    padding: const EdgeInsetsDirectional.only(
                      end: KansoSpacing.md,
                      top: KansoSpacing.sm,
                      bottom: KansoSpacing.sm,
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: <Widget>[
                        Text(
                          task.text,
                          maxLines: 3,
                          overflow: TextOverflow.ellipsis,
                          style: typography.body.copyWith(
                            color: task.checked
                                ? colors.textMuted
                                : colors.text,
                            decoration: task.checked
                                ? TextDecoration.lineThrough
                                : null,
                            decorationColor: colors.textMuted,
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.xxs),
                        Row(
                          children: <Widget>[
                            if (due != null && tone != null) ...<Widget>[
                              TaskDueChip(label: due, tone: tone),
                              const SizedBox(width: KansoSpacing.xs),
                            ],
                            Expanded(
                              child: Text(
                                whereLine,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: typography.meta.copyWith(
                                  color: colors.textMuted,
                                ),
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
