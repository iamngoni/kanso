// Kanso — calendar_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/journal_time.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../injection.dart';
import '../../state/calendar/calendar_bloc.dart';
import 'calendar_day_cell.dart';

/// The journal's month calendar, as the desktop's: the month with back and
/// forward, a Today button that goes to today's month, the weekdays from
/// the first day of the week, and the days with a note marked. Choosing a
/// day closes the sheet with that day.
///
/// It reads the [CalendarBloc] above it ([CalendarSheet.show] makes one).
class CalendarSheet extends StatelessWidget {
  const CalendarSheet({super.key});

  /// Shows the calendar over the whole app, on the month of the daily note
  /// [openNote] (or today's), and completes with the day chosen, or null.
  /// Weeks start on the first day of the week of the device's language.
  static Future<DayDate?> show(BuildContext context, {String? openNote}) {
    final firstDay = MaterialLocalizations.of(context).firstDayOfWeekIndex;
    return showKansoBottomSheet<DayDate>(
      context,
      builder: (_) => BlocProvider<CalendarBloc>(
        create: (_) => getIt<CalendarBloc>()
          ..add(
            CalendarEvent.started(
              openNote: openNote,
              today: dayDateOf(clock.now()),
              // Flutter counts weeks from Sunday; the library from Monday.
              weekStart: weekStartOf(firstDay),
            ),
          ),
        child: const CalendarSheet(),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<CalendarBloc>();
    return BlocBuilder<CalendarBloc, CalendarState>(
      builder: (context, state) {
        final grid = state.grid;
        return Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.sm,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Row(
                children: <Widget>[
                  KansoIconButton(
                    icon: KansoIcons.chevronLeft,
                    semanticLabel: l10n.calendarPreviousMonth,
                    onPressed: () =>
                        bloc.add(const CalendarEvent.monthStepped(-1)),
                  ),
                  Expanded(
                    child: Semantics(
                      header: true,
                      child: Text(
                        grid?.title ?? '',
                        textAlign: TextAlign.center,
                        style: context.typography.barTitle,
                      ),
                    ),
                  ),
                  KansoPressable(
                    onTap: () => bloc.add(const CalendarEvent.todayShown()),
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.sm),
                      child: Text(
                        l10n.calendarToday,
                        style: context.typography.body.copyWith(
                          color: colors.accent,
                        ),
                      ),
                    ),
                  ),
                  KansoIconButton(
                    icon: KansoIcons.chevronRight,
                    semanticLabel: l10n.calendarNextMonth,
                    onPressed: () =>
                        bloc.add(const CalendarEvent.monthStepped(1)),
                  ),
                ],
              ),
              if (grid != null) ...<Widget>[
                Row(
                  children: <Widget>[
                    for (final label in grid.weekdayLabels)
                      Expanded(
                        child: Center(
                          child: Text(
                            label,
                            style: context.typography.meta,
                          ),
                        ),
                      ),
                  ],
                ),
                const SizedBox(height: KansoSpacing.xxs),
                for (final week in grid.weeks)
                  Row(
                    children: <Widget>[
                      for (final day in week)
                        Expanded(
                          child: Center(
                            child: CalendarDayCell(
                              day: day,
                              onTap: () => Navigator.of(context).pop(day.date),
                            ),
                          ),
                        ),
                    ],
                  ),
              ] else if (state.status == LoadStatus.failed) ...<Widget>[
                Padding(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  child: Text(
                    l10n.calendarFailed,
                    textAlign: TextAlign.center,
                    style: context.typography.secondary,
                  ),
                ),
                Center(
                  child: KansoButton(
                    label: l10n.retry,
                    expanded: false,
                    onPressed: () => bloc.add(const CalendarEvent.retried()),
                  ),
                ),
              ] else
                const SizedBox(height: 6 * CalendarDayCell.size),
              Padding(
                padding: const EdgeInsets.only(top: KansoSpacing.xs),
                child: Text(l10n.calendarHint, style: context.typography.meta),
              ),
            ],
          ),
        );
      },
    );
  }
}
