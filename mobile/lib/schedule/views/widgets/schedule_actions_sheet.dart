// Kanso — schedule_actions_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/schedule_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/journal_time.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_month_grid.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../utils/schedule_days.dart';

/// What the actions sheet of a scheduled item can ask for.
enum ScheduleAction {
  /// Tick the task, or clear it.
  tick,

  /// Open the note.
  open,

  /// Pick another day for it.
  move,
}

/// What can be done with one scheduled item: tick the task, open its note,
/// and Move to day…, which keeps its time and zone. A projected occurrence
/// has no date of its own, so it cannot be moved and says so.
class ScheduleActionsSheet extends StatelessWidget {
  const ScheduleActionsSheet({required this.row, super.key});

  final ScheduleRow row;

  /// Shows the sheet over the whole app and completes with the action
  /// chosen, or null.
  static Future<ScheduleAction?> show(BuildContext context, ScheduleRow row) {
    return showKansoBottomSheet<ScheduleAction>(
      context,
      builder: (_) => ScheduleActionsSheet(row: row),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final isTask = row.kind == ScheduleRowKind.task;
    void choose(ScheduleAction action) => Navigator.of(context).pop(action);
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            0,
          ),
          child: Text(
            row.title,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: context.typography.barTitle,
          ),
        ),
        KansoSheetGroup(
          dividerIndent: 48,
          children: <Widget>[
            if (isTask)
              KansoSettingsRow(
                icon: KansoIcons.circleCheck,
                label: row.done ? l10n.tasksTickBack : l10n.tasksTickOff,
                onTap: () => choose(ScheduleAction.tick),
              ),
            KansoSettingsRow(
              icon: KansoIcons.fileText,
              label: l10n.tasksOpenNote,
              onTap: () => choose(ScheduleAction.open),
            ),
            if (canMove(row))
              KansoSettingsRow(
                icon: KansoIcons.calendarDays,
                label: l10n.scheduleMoveToDay,
                onTap: () => choose(ScheduleAction.move),
              ),
          ],
        ),
        if (!canMove(row))
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.lg,
              KansoSpacing.xs,
              KansoSpacing.lg,
              0,
            ),
            child: Text(
              l10n.scheduleProjectedHint,
              style: context.typography.meta.copyWith(
                color: colors.textMuted,
              ),
            ),
          ),
        const SizedBox(height: KansoSpacing.md),
      ],
    );
  }
}

/// A month to pick the day to move a scheduled item to. It pops with the
/// [DayDate] tapped.
class ScheduleDaySheet extends StatelessWidget {
  const ScheduleDaySheet({
    required this.row,
    required this.firstWeekday,
    super.key,
  });

  final ScheduleRow row;

  /// The weekday weeks start on: [DateTime.monday] through
  /// [DateTime.sunday].
  final int firstWeekday;

  /// Shows the sheet over the whole app and completes with the day, or null.
  static Future<DayDate?> show(
    BuildContext context,
    ScheduleRow row, {
    required int weekStart,
  }) {
    return showKansoBottomSheet<DayDate>(
      context,
      builder: (_) => ScheduleDaySheet(row: row, firstWeekday: weekStart + 1),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.only(bottom: KansoSpacing.xxs),
            child: Text(
              context.l10n.scheduleMoveTitle,
              style: context.typography.barTitle,
            ),
          ),
          KansoMonthGrid(
            selected: dateOf(row.date),
            firstWeekday: firstWeekday,
            onChanged: (day) => Navigator.of(context).pop(dayDateOf(day)),
          ),
        ],
      ),
    );
  }
}
