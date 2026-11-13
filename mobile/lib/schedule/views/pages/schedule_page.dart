// Kanso — schedule_page.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/journal_time.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../state/schedule/schedule_bloc.dart';
import '../../utils/schedule_days.dart';
import '../widgets/schedule_pane.dart';

/// The Schedule, at `/schedule`: the tasks and dated notes of a month as a
/// month grid or an agenda. Read when it is opened and given up when it is
/// left. Weeks start on the first day of the week of the device's region.
class SchedulePage extends StatelessWidget {
  const SchedulePage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<ScheduleBloc>(
      create: (_) => getIt<ScheduleBloc>()
        ..add(
          ScheduleEvent.started(
            today: dayOf(clock.now()),
            weekStart: weekStartOf(
              firstDayOfWeekIndexFor(
                WidgetsBinding.instance.platformDispatcher.locale.countryCode,
              ),
            ),
          ),
        ),
      child: KansoPage(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.list,
        body: const SchedulePane(),
      ),
    );
  }
}
