// Kanso — library_schedule_row.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';

/// The library row that opens the Schedule: the tasks and dated notes on a
/// calendar.
class LibraryScheduleRow extends StatelessWidget {
  const LibraryScheduleRow({super.key});

  @override
  Widget build(BuildContext context) {
    return KansoSidebarRow(
      icon: KansoIcons.calendarDays,
      label: context.l10n.scheduleTitle,
      onTap: () {
        KansoDrawerScope.maybeOf(context)?.close();
        unawaited(context.push<void>(KansoRoutes.schedule));
      },
    );
  }
}
