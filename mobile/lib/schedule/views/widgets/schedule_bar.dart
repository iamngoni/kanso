// Kanso — schedule_bar.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:intl/intl.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/data/schedule_status.dart';
import '../../state/schedule/schedule_bloc.dart';
import '../../utils/schedule_days.dart';
import 'schedule_filter_sheet.dart';

/// The Schedule's bar: back, the title, the filter; and under it the month
/// with back and forward and Today, and the choice of month or agenda.
class ScheduleBar extends StatelessWidget {
  const ScheduleBar({required this.state, super.key});

  final ScheduleState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<ScheduleBloc>();
    final index = context.read<LibraryIndexBloc>();
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        KansoTopBar(
          bordered: false,
          leading: const ShellBackButton(),
          title: l10n.scheduleTitle,
          trailing: <Widget>[
            KansoIconButton(
              icon: KansoIcons.ellipsis,
              semanticLabel: l10n.scheduleFilter,
              selected: state.refined,
              onPressed: () => showKansoBottomSheet<void>(
                context,
                builder: (_) => MultiBlocProvider(
                  providers: <BlocProvider<dynamic>>[
                    BlocProvider<ScheduleBloc>.value(value: bloc),
                    BlocProvider<LibraryIndexBloc>.value(value: index),
                  ],
                  child: const ScheduleFilterSheet(),
                ),
              ),
            ),
          ],
        ),
        Padding(
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.xs,
            end: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              KansoIconButton(
                icon: KansoIcons.chevronLeft,
                semanticLabel: l10n.calendarPreviousMonth,
                onPressed: () => bloc.add(const ScheduleEvent.monthStepped(-1)),
              ),
              Expanded(
                child: Semantics(
                  header: true,
                  child: Text(
                    state.status == ScheduleStatus.loading
                        ? ''
                        : DateFormat.yMMMM(
                            l10n.localeName,
                          ).format(dateOf(state.focused)),
                    textAlign: TextAlign.center,
                    style: context.typography.heading2,
                  ),
                ),
              ),
              KansoPressable(
                onTap: () => bloc.add(
                  ScheduleEvent.todayShown(dayOf(clock.now())),
                ),
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
                onPressed: () => bloc.add(const ScheduleEvent.monthStepped(1)),
              ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xxs,
            KansoSpacing.md,
            KansoSpacing.sm,
          ),
          child: KansoSegmented<ScheduleMode>(
            expanded: true,
            semanticLabel: l10n.scheduleViewLabel,
            selected: state.mode,
            onChanged: (mode) => bloc.add(ScheduleEvent.modeChanged(mode)),
            segments: <KansoSegment<ScheduleMode>>[
              KansoSegment<ScheduleMode>(
                value: ScheduleMode.month,
                label: l10n.scheduleMonth,
                icon: KansoIcons.calendarDays,
              ),
              KansoSegment<ScheduleMode>(
                value: ScheduleMode.agenda,
                label: l10n.scheduleAgenda,
                icon: KansoIcons.list,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
