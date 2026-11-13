// Kanso — schedule_item_tile.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../core/bridge/api/schedule_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../utils/schedule_days.dart';

/// One thing on a day: a task with its box, or a note with the mark of the
/// date property that places it (play for publish, pause for embargo, a
/// calendar for any other: the desktop's ▶ ⏸ ◆).
///
/// Its time and, quietly, the note and notebook it is in sit under the
/// title; a repeating task has ↻ (the loop icon) and how it repeats. A projected occurrence
/// (a repeat worked out and never written) is dimmed; ticking it ticks the
/// real task. Pressing the box ticks, pressing the rest opens the note, and
/// the more button (or holding the row) opens its actions. One node for a
/// screen reader.
class ScheduleItemTile extends StatelessWidget {
  const ScheduleItemTile({
    required this.row,
    required this.notebook,
    required this.onTick,
    required this.onOpen,
    required this.onMore,
    super.key,
  });

  final ScheduleRow row;

  /// The name of the notebook the item is in.
  final String notebook;
  final VoidCallback onTick;
  final VoidCallback onOpen;
  final VoidCallback onMore;

  /// How much of its colour a projected occurrence keeps.
  static const double projectedOpacity = 0.5;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final isTask = row.kind == ScheduleRowKind.task;
    final zone = row.zone;
    final time = row.time;
    final repeat = row.repeat;
    final when = <String>[
      ?time,
      if (zone != null) zone.split('/').last.replaceAll('_', ' '),
    ].join(' ');
    final context1 = l10n.scheduleRowContext(row.noteTitle, notebook);
    final spoken = <String>[
      if (when.isEmpty)
        row.title
      else
        l10n.scheduleRowSemantics(row.title, when),
      if (repeat != null) l10n.tasksRepeatSemantics(repeat),
      if (row.projected) l10n.scheduleProjectedSemantics,
      context1,
    ].join(', ');
    final ink = row.done ? colors.textMuted : colors.text;
    return Semantics(
      container: true,
      label: spoken,
      checked: isTask ? row.done : null,
      onTap: onOpen,
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        if (isTask)
          CustomSemanticsAction(
            label: row.done ? l10n.tasksTickBack : l10n.tasksTickOff,
          ): onTick,
        CustomSemanticsAction(label: l10n.tasksOpenNote): onOpen,
        CustomSemanticsAction(label: l10n.scheduleMoreActions): onMore,
      },
      child: ExcludeSemantics(
        child: Opacity(
          opacity: row.projected ? projectedOpacity : 1,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              KansoPressable(
                onTap: isTask ? onTick : onOpen,
                child: Padding(
                  padding: const EdgeInsetsDirectional.only(
                    start: KansoSpacing.md,
                    end: KansoSpacing.sm,
                    top: KansoSpacing.sm,
                    bottom: KansoSpacing.sm,
                  ),
                  child: IgnorePointer(
                    child: isTask
                        ? KansoCheckbox(value: row.done)
                        : _NoteMark(property: row.property),
                  ),
                ),
              ),
              Expanded(
                child: KansoPressable(
                  onTap: onOpen,
                  onLongPress: onMore,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      vertical: KansoSpacing.sm,
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: <Widget>[
                        Text(
                          row.title,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                          style: typography.body.copyWith(
                            color: ink,
                            decoration: row.done
                                ? TextDecoration.lineThrough
                                : null,
                            decorationColor: colors.textMuted,
                          ),
                        ),
                        const SizedBox(height: KansoSpacing.xxs),
                        Wrap(
                          spacing: KansoSpacing.xs,
                          crossAxisAlignment: WrapCrossAlignment.center,
                          children: <Widget>[
                            if (when.isNotEmpty)
                              Text(
                                when,
                                style: typography.meta.copyWith(
                                  color: colors.textSecondary,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            if (repeat != null) _RepeatMark(repeat: repeat),
                            Text(
                              context1,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: typography.meta.copyWith(
                                color: colors.textMuted,
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
              ),
              KansoIconButton(
                icon: KansoIcons.ellipsis,
                semanticLabel: l10n.scheduleMoreActions,
                iconSize: 18,
                onPressed: onMore,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The soft round mark in a note's place of a task's box.
class _NoteMark extends StatelessWidget {
  const _NoteMark({required this.property});

  final String? property;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Container(
      width: 20,
      height: 20,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: colors.accent.withValues(alpha: 0.16),
        borderRadius: KansoRadii.smallBorder,
      ),
      child: KansoIcon(noteMark(property), size: 11, color: colors.accent),
    );
  }
}

/// A repeating task's ↻ (drawn as the loop icon) and how it repeats.
class _RepeatMark extends StatelessWidget {
  const _RepeatMark({required this.repeat});

  final String repeat;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        KansoIcon(
          KansoIcons.rotateCw,
          size: 11,
          color: colors.textSecondary,
        ),
        const SizedBox(width: KansoSpacing.xxs),
        Text(
          repeat,
          style: context.typography.meta.copyWith(
            color: colors.textSecondary,
          ),
        ),
      ],
    );
  }
}
