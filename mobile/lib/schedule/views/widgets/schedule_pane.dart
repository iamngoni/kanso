// Kanso — schedule_pane.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../models/data/schedule_status.dart';
import '../../state/schedule/schedule_bloc.dart';
import 'schedule_agenda_view.dart';
import 'schedule_bar.dart';
import 'schedule_month_view.dart';

/// The Schedule: a bar, and under it the month grid with the chosen day, or
/// the agenda; while it is read the shape of it, or why it did not load.
class SchedulePane extends StatelessWidget {
  const SchedulePane({super.key});

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: context.colors.list,
      child: BlocBuilder<ScheduleBloc, ScheduleState>(
        builder: (context, state) {
          final notice = state.notice;
          return Stack(
            children: <Widget>[
              Column(
                children: <Widget>[
                  ScheduleBar(state: state),
                  Expanded(child: _Body(state: state)),
                ],
              ),
              if (notice != null)
                PositionedDirectional(
                  start: KansoSpacing.md,
                  end: KansoSpacing.md,
                  bottom:
                      MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
                  child: _NoticeLine(notice: notice),
                ),
            ],
          );
        },
      ),
    );
  }
}

class _Body extends StatelessWidget {
  const _Body({required this.state});

  final ScheduleState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return switch (state.status) {
      ScheduleStatus.loading => Semantics(
        label: l10n.scheduleLoading,
        container: true,
        child: ExcludeSemantics(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                for (var i = 0; i < 6; i++) ...<Widget>[
                  KansoSkeleton(height: i < 3 ? 40 : 16),
                  const SizedBox(height: KansoSpacing.sm),
                ],
              ],
            ),
          ),
        ),
      ),
      ScheduleStatus.failed => Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: KansoMessageCard(
            icon: KansoIcons.calendarDays,
            title: l10n.scheduleFailedTitle,
            message: l10n.scheduleFailedMessage,
            actions: <Widget>[
              KansoButton(
                label: l10n.retry,
                onPressed: () => context.read<ScheduleBloc>().add(
                  const ScheduleEvent.retried(),
                ),
              ),
            ],
          ),
        ),
      ),
      ScheduleStatus.ready => switch (state.mode) {
        ScheduleMode.month => ScheduleMonthView(state: state),
        ScheduleMode.agenda => ScheduleAgendaView(state: state),
      },
    };
  }
}

class _NoticeLine extends StatelessWidget {
  const _NoticeLine({required this.notice});

  final ScheduleNotice notice;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Semantics(
      liveRegion: true,
      child: Container(
        padding: const EdgeInsetsDirectional.only(
          start: KansoSpacing.md,
          top: KansoSpacing.xxs,
          bottom: KansoSpacing.xxs,
        ),
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: KansoRadii.mediumBorder,
          border: Border.all(color: colors.divider),
          boxShadow: KansoShadows.floating(context.brightness),
        ),
        child: Row(
          children: <Widget>[
            Expanded(
              child: Text(
                switch (notice) {
                  ScheduleNotice.moved => l10n.scheduleMoved,
                  ScheduleNotice.locked => l10n.scheduleLocked,
                  ScheduleNotice.failed => l10n.scheduleChangeFailed,
                },
                style: context.typography.secondary.copyWith(
                  color: colors.text,
                ),
              ),
            ),
            KansoPressable(
              onTap: () => context.read<ScheduleBloc>().add(
                const ScheduleEvent.noticeDismissed(),
              ),
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: Text(
                  l10n.noteListDismiss,
                  style: context.typography.body.copyWith(color: colors.accent),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
