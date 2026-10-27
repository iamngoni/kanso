// Kanso — library_tasks_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';

/// The library row that opens the Tasks place. [overdue] open tasks past
/// their date are counted in a warm red pill, and nothing is shown when
/// there are none, as the desktop's sidebar does.
class LibraryTasksRow extends StatelessWidget {
  const LibraryTasksRow({
    required this.overdue,
    this.selected = false,
    super.key,
  });

  final int overdue;
  final bool selected;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Semantics(
      value: overdue > 0 ? l10n.libraryTasksOverdue(overdue) : null,
      child: KansoSidebarRow(
        icon: KansoIcons.circleCheck,
        label: l10n.libraryTasks,
        selected: selected,
        badge: overdue > 0
            ? ExcludeSemantics(
                child: Container(
                  height: 17,
                  padding: const EdgeInsets.symmetric(horizontal: 6),
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: colors.danger.withValues(alpha: 0.16),
                    borderRadius: KansoRadii.pillBorder,
                  ),
                  child: Text(
                    '$overdue',
                    style: context.typography.meta.copyWith(
                      fontWeight: FontWeight.w600,
                      color: colors.danger,
                    ),
                  ),
                ),
              )
            : null,
        onTap: () {
          KansoDrawerScope.maybeOf(context)?.close();
          context.go(const BrowseLocation.tasks().location);
        },
      ),
    );
  }
}
