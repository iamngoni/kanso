// Kanso — tasks_notice_line.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../models/data/tasks_notice.dart';

/// The line over the list that says a tick did not happen, until it is
/// dismissed.
class TasksNoticeLine extends StatelessWidget {
  const TasksNoticeLine({
    required this.notice,
    required this.onDismiss,
    super.key,
  });

  final TasksNotice notice;
  final VoidCallback onDismiss;

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
          boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
        ),
        child: Row(
          children: <Widget>[
            Expanded(
              child: Text(
                switch (notice) {
                  TasksNotice.moved => l10n.tasksMoved,
                  TasksNotice.locked => l10n.noteListChangeLocked,
                  TasksNotice.failed => l10n.tasksChangeFailed,
                },
                style: context.typography.secondary.copyWith(
                  color: colors.text,
                ),
              ),
            ),
            KansoPressable(
              onTap: onDismiss,
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
