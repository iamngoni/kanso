// Kanso — journal_strip.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../state/journal/journal_bloc.dart';
import 'day_stepper.dart';

/// The line under the Journal notebook's bar: today's note, and the day
/// before, the day after and the calendar. With no note open the days are
/// counted from today.
class JournalStrip extends StatelessWidget {
  const JournalStrip({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.only(start: KansoSpacing.xxs),
          child: Row(
            children: <Widget>[
              KansoPressable(
                onTap: () => context.read<JournalBloc>().add(
                  const JournalEvent.todayRequested(),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.journalTodaysNote,
                    style: context.typography.body.copyWith(
                      color: context.colors.accent,
                    ),
                  ),
                ),
              ),
              const Spacer(),
              const DayStepper(),
            ],
          ),
        ),
        const KansoDivider(),
      ],
    );
  }
}
