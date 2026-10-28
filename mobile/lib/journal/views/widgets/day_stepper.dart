// Kanso — day_stepper.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../state/journal/journal_bloc.dart';
import '../../utils/journal_flows.dart';

/// The controls of the journal: the day before, the day after and the
/// calendar. [openNote] is the daily note open, if any; the days are
/// counted from it (from today when it is none). It needs the
/// [JournalBloc] of a `JournalOpener` above it.
class DayStepper extends StatelessWidget {
  const DayStepper({this.openNote, super.key});

  final String? openNote;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<JournalBloc>();
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        KansoIconButton(
          icon: KansoIcons.chevronLeft,
          semanticLabel: l10n.journalPreviousDay,
          onPressed: () => bloc.add(
            JournalEvent.stepRequested(openNote: openNote, forward: false),
          ),
        ),
        KansoIconButton(
          icon: KansoIcons.chevronRight,
          semanticLabel: l10n.journalNextDay,
          onPressed: () => bloc.add(
            JournalEvent.stepRequested(openNote: openNote, forward: true),
          ),
        ),
        KansoIconButton(
          icon: KansoIcons.calendar,
          semanticLabel: l10n.journalCalendar,
          onPressed: () => showCalendar(context, openNote: openNote),
        ),
      ],
    );
  }
}
