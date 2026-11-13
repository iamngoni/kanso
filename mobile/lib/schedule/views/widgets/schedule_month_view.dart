// Kanso — schedule_month_view.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:intl/intl.dart';

import '../../../core/bridge/api/schedule_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_colors.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../state/schedule/schedule_bloc.dart';
import '../../utils/schedule_days.dart';
import 'schedule_row_tile.dart';

/// The month view: the month's weeks with a dot for each task and note on a
/// day, and under them the chosen day's items.
class ScheduleMonthView extends StatelessWidget {
  const ScheduleMonthView({required this.state, super.key});

  final ScheduleState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final locale = l10n.localeName;
    final weeks = monthWeeks(state.focused, state.weekStart);
    final byDay = state.byDay;
    final items = byDay[state.selected] ?? const <ScheduleRow>[];
    final labels = <String>[
      for (final day in weeks.first)
        DateFormat.E(locale).format(dateOf(day)).characters.first.toUpperCase(),
    ];
    return Column(
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xs),
          child: Column(
            children: <Widget>[
              Row(
                children: <Widget>[
                  for (final label in labels)
                    Expanded(
                      child: Center(
                        child: Text(
                          label,
                          style: context.typography.meta.copyWith(
                            color: colors.textMuted,
                          ),
                        ),
                      ),
                    ),
                ],
              ),
              const SizedBox(height: KansoSpacing.xxs),
              for (final week in weeks)
                Row(
                  children: <Widget>[
                    for (final day in week)
                      Expanded(
                        child: _DayCell(
                          day: day,
                          inMonth: day.month == state.focused.month,
                          isToday: day == state.today,
                          isSelected: day == state.selected,
                          rows: byDay[day] ?? const <ScheduleRow>[],
                        ),
                      ),
                  ],
                ),
              const SizedBox(height: KansoSpacing.xs),
            ],
          ),
        ),
        const KansoDivider(),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xxs,
          ),
          child: Row(
            children: <Widget>[
              Expanded(
                child: Semantics(
                  header: true,
                  child: Text(
                    DateFormat.MMMMEEEEd(locale).format(dateOf(state.selected)),
                    style: context.typography.barTitle,
                  ),
                ),
              ),
              if (items.isNotEmpty)
                Text(
                  l10n.scheduleDayItems(items.length),
                  style: context.typography.meta.copyWith(
                    color: colors.textMuted,
                  ),
                ),
            ],
          ),
        ),
        Expanded(
          child: items.isEmpty
              ? Padding(
                  padding: const EdgeInsets.all(KansoSpacing.lg),
                  child: Align(
                    alignment: AlignmentDirectional.topStart,
                    child: Text(
                      l10n.scheduleDayEmpty,
                      style: context.typography.body.copyWith(
                        color: colors.textMuted,
                      ),
                    ),
                  ),
                )
              : KansoListScrollbar(
                  child: ListView(
                    padding: EdgeInsets.only(
                      bottom: MediaQuery.paddingOf(context).bottom + 80,
                    ),
                    children: <Widget>[
                      for (final row in items)
                        ScheduleRowTile(
                          key: ValueKey<String>(_keyOf(row)),
                          row: row,
                        ),
                    ],
                  ),
                ),
        ),
      ],
    );
  }
}

/// A row's identity on its day: the same task on another day is another row.
String _keyOf(ScheduleRow row) =>
    '${row.noteId}:${row.line}:${row.property}:${row.projected}';

/// One day of the grid: its number, and a dot for what is on it.
class _DayCell extends StatelessWidget {
  const _DayCell({
    required this.day,
    required this.inMonth,
    required this.isToday,
    required this.isSelected,
    required this.rows,
  });

  final DayDate day;
  final bool inMonth;
  final bool isToday;
  final bool isSelected;
  final List<ScheduleRow> rows;

  /// How many dots a day shows.
  static const int maxDots = 3;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final ink = isSelected
        ? colors.buttonText
        : isToday
        ? colors.accent
        : inMonth
        ? colors.text
        : colors.textMuted;
    final label = DateFormat.yMMMMEEEEd(l10n.localeName).format(dateOf(day));
    return KansoPressable(
      onTap: () => context.read<ScheduleBloc>().add(
        ScheduleEvent.daySelected(day),
      ),
      semanticLabel: rows.isEmpty
          ? label
          : '$label, ${l10n.scheduleDayItems(rows.length)}',
      child: ExcludeSemantics(
        child: SizedBox(
          height: 54,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              Container(
                width: 34,
                height: 34,
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: isSelected ? colors.button : null,
                  borderRadius: KansoRadii.mediumBorder,
                  border: isToday && !isSelected
                      ? Border.all(color: colors.accent, width: 1.5)
                      : null,
                ),
                child: Text(
                  '${day.day}',
                  style: context.typography.body.copyWith(
                    color: ink,
                    fontWeight: isSelected || isToday ? FontWeight.w600 : null,
                  ),
                ),
              ),
              const SizedBox(height: 3),
              SizedBox(
                height: 6,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: <Widget>[
                    for (final row in rows.take(maxDots))
                      Container(
                        width: 6,
                        height: 6,
                        margin: const EdgeInsets.symmetric(horizontal: 1.5),
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: _dot(colors, row).withValues(
                            alpha: row.projected || !inMonth ? 0.4 : 1,
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  static Color _dot(KansoColors colors, ScheduleRow row) {
    if (row.done) {
      return colors.textMuted;
    }
    return row.kind == ScheduleRowKind.task ? colors.accent : colors.link;
  }
}
