// Kanso — kanso_month_grid.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon_button.dart';
import 'kanso_pressable.dart';

/// A month of days to pick one from, in Kanso's own drawing: the month with
/// back and forward, a Today button, the weekdays from [firstWeekday], and
/// the day picked in the button colour. It knows nothing of the library;
/// [onChanged] is told the day (midnight, on the device's clock) that was
/// tapped.
class KansoMonthGrid extends StatefulWidget {
  const KansoMonthGrid({
    required this.onChanged,
    this.selected,
    this.firstWeekday = DateTime.monday,
    super.key,
  });

  /// The day picked, if any; the grid starts on its month.
  final DateTime? selected;

  /// The weekday the weeks start on: [DateTime.monday] through
  /// [DateTime.sunday].
  final int firstWeekday;

  final ValueChanged<DateTime> onChanged;

  @override
  State<KansoMonthGrid> createState() => _KansoMonthGridState();
}

class _KansoMonthGridState extends State<KansoMonthGrid> {
  late DateTime shown = _firstOf(widget.selected ?? clock.now());

  static DateTime _firstOf(DateTime day) => DateTime(day.year, day.month);

  @override
  void didUpdateWidget(KansoMonthGrid oldWidget) {
    super.didUpdateWidget(oldWidget);
    final now = widget.selected;
    if (now != null && now != oldWidget.selected) {
      shown = _firstOf(now);
    }
  }

  void step(int by) =>
      setState(() => shown = DateTime(shown.year, shown.month + by));

  /// The weeks of the shown month, each of seven days; the days of the
  /// months either side fill the first and last week.
  List<List<DateTime>> weeks() {
    final lead = (shown.weekday - widget.firstWeekday + 7) % 7;
    final start = DateTime(shown.year, shown.month, 1 - lead);
    final days = DateTime(shown.year, shown.month + 1, 0).day;
    final count = ((lead + days) / 7).ceil();
    return <List<DateTime>>[
      for (var week = 0; week < count; week++)
        <DateTime>[
          for (var day = 0; day < 7; day++)
            DateTime(start.year, start.month, start.day + week * 7 + day),
        ],
    ];
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final locale = l10n.localeName;
    final today = clock.now();
    final grid = weeks();
    final labels = <String>[
      for (final day in grid.first)
        DateFormat.E(locale).format(day).characters.first.toUpperCase(),
    ];
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Row(
          children: <Widget>[
            KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: l10n.calendarPreviousMonth,
              onPressed: () => step(-1),
            ),
            Expanded(
              child: Semantics(
                header: true,
                child: Text(
                  DateFormat.yMMMM(locale).format(shown),
                  textAlign: TextAlign.center,
                  style: context.typography.barTitle,
                ),
              ),
            ),
            KansoPressable(
              onTap: () => setState(() => shown = _firstOf(clock.now())),
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: Text(
                  l10n.calendarToday,
                  style: context.typography.body.copyWith(color: colors.accent),
                ),
              ),
            ),
            KansoIconButton(
              icon: KansoIcons.chevronRight,
              semanticLabel: l10n.calendarNextMonth,
              onPressed: () => step(1),
            ),
          ],
        ),
        Row(
          children: <Widget>[
            for (final label in labels)
              Expanded(
                child: Center(
                  child: Text(label, style: context.typography.meta),
                ),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.xxs),
        for (final week in grid)
          Row(
            children: <Widget>[
              for (final day in week)
                Expanded(
                  child: Center(
                    child: _Day(
                      day: day,
                      inMonth: day.month == shown.month,
                      isToday: _same(day, today),
                      isSelected:
                          widget.selected != null &&
                          _same(day, widget.selected!),
                      onTap: () => widget.onChanged(day),
                    ),
                  ),
                ),
            ],
          ),
      ],
    );
  }

  static bool _same(DateTime a, DateTime b) =>
      a.year == b.year && a.month == b.month && a.day == b.day;
}

class _Day extends StatelessWidget {
  const _Day({
    required this.day,
    required this.inMonth,
    required this.isToday,
    required this.isSelected,
    required this.onTap,
  });

  final DateTime day;
  final bool inMonth;
  final bool isToday;
  final bool isSelected;
  final VoidCallback onTap;

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
    return KansoPressable(
      onTap: onTap,
      semanticLabel: DateFormat.yMMMMEEEEd(l10n.localeName).format(day),
      child: ExcludeSemantics(
        child: SizedBox.square(
          dimension: KansoSizes.touchTarget,
          child: Center(
            child: Container(
              width: KansoSizes.touchTarget - 6,
              height: KansoSizes.touchTarget - 6,
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
          ),
        ),
      ),
    );
  }
}
