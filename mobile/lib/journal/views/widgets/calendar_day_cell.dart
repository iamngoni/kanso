// Kanso — calendar_day_cell.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/journal_time.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// One day of the calendar: its number, a dot under it when it has a note,
/// and a wash of the accent colour when it is today. A day of the months
/// either side is drawn muted; it can be opened all the same.
class CalendarDayCell extends StatelessWidget {
  const CalendarDayCell({required this.day, required this.onTap, super.key});

  final CalendarDay day;
  final VoidCallback onTap;

  /// The size of a day's target.
  static const double size = KansoSizes.touchTarget;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final date = DateFormat.yMMMMEEEEd(
      l10n.localeName,
    ).format(dateTimeOf(day.date));
    final label = switch ((day.isToday, day.hasNote)) {
      (true, true) => l10n.calendarDayTodayWithNote(date),
      (true, false) => l10n.calendarDayToday(date),
      (false, true) => l10n.calendarDayWithNote(date),
      (false, false) => l10n.calendarDayLabel(date),
    };
    final ink = day.isToday
        ? colors.buttonText
        : day.inMonth
        ? colors.text
        : colors.textMuted;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: label,
      child: ExcludeSemantics(
        child: SizedBox.square(
          dimension: size,
          child: Center(
            child: Container(
              width: size - 4,
              height: size - 4,
              decoration: BoxDecoration(
                color: day.isToday ? colors.button : null,
                borderRadius: KansoRadii.mediumBorder,
              ),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: <Widget>[
                  Text(
                    '${day.date.day}',
                    style: context.typography.body.copyWith(
                      color: ink,
                      fontWeight: day.isToday ? FontWeight.w600 : null,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Container(
                    width: 5,
                    height: 5,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: day.hasNote
                          ? (day.isToday ? colors.buttonText : colors.accent)
                          : null,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
