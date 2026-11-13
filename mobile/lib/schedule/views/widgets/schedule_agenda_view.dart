// Kanso — schedule_agenda_view.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../core/bridge/api/schedule_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../state/schedule/schedule_bloc.dart';
import '../../utils/schedule_days.dart';
import 'schedule_row_tile.dart';

/// The agenda view: every day of the month that has something on it, one
/// after another, today's marked, each with its items.
class ScheduleAgendaView extends StatelessWidget {
  const ScheduleAgendaView({required this.state, super.key});

  final ScheduleState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final locale = l10n.localeName;
    final inMonth = <ScheduleRow>[
      for (final row in state.rows)
        if (row.date.year == state.focused.year &&
            row.date.month == state.focused.month)
          row,
    ];
    if (inMonth.isEmpty) {
      return KansoEmptyState(
        title: l10n.scheduleEmptyTitle,
        message: l10n.scheduleEmptyMessage,
      );
    }
    final days = rowsByDay(inMonth);
    final order = days.keys.toList()
      ..sort((a, b) => dateOf(a).compareTo(dateOf(b)));
    return KansoListScrollbar(
      child: ListView(
        padding: EdgeInsets.only(
          bottom: MediaQuery.paddingOf(context).bottom + 80,
        ),
        children: <Widget>[
          for (final day in order) ...<Widget>[
            Padding(
              padding: const EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.md,
                KansoSpacing.md,
                KansoSpacing.xxs,
              ),
              child: Semantics(
                header: true,
                child: Row(
                  children: <Widget>[
                    Text(
                      DateFormat.MMMEd(locale).format(dateOf(day)),
                      style: context.typography.sectionLabel.copyWith(
                        color: day == state.today
                            ? colors.accent
                            : colors.textSecondary,
                      ),
                    ),
                    if (day == state.today) ...<Widget>[
                      const SizedBox(width: KansoSpacing.xs),
                      Text(
                        l10n.scheduleToday,
                        style: context.typography.meta.copyWith(
                          color: colors.accent,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
            for (final row in days[day]!)
              ScheduleRowTile(
                key: ValueKey<String>(
                  '${row.noteId}:${row.line}:${row.property}:${row.date.day}',
                ),
                row: row,
              ),
          ],
        ],
      ),
    );
  }
}
