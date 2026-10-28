// Kanso — tasks_loading_pane.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// The shape of the tasks while they are read: a header and rows of a box
/// and two lines, with the real row's padding so nothing jumps.
class TasksLoadingPane extends StatelessWidget {
  const TasksLoadingPane({super.key});

  /// How wide each row's text is, as a share of the row; it repeats.
  static const List<double> fractions = <double>[0.7, 0.5, 0.82, 0.6];

  /// How many rows are drawn: enough to fill a screen.
  static const int rows = 8;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.tasksLoading,
      container: true,
      child: ExcludeSemantics(
        child: ListView(
          physics: const NeverScrollableScrollPhysics(),
          padding: const EdgeInsets.only(top: KansoSpacing.sm),
          children: <Widget>[
            for (var i = 0; i < rows; i++) ...<Widget>[
              if (i > 0) const KansoDivider(indent: KansoSpacing.md),
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.sm,
                ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    const KansoSkeleton(width: 20, height: 20),
                    const SizedBox(width: KansoSpacing.sm),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: <Widget>[
                          Align(
                            alignment: AlignmentDirectional.centerStart,
                            child: FractionallySizedBox(
                              widthFactor: fractions[i % fractions.length],
                              child: const KansoSkeleton(height: 14),
                            ),
                          ),
                          const SizedBox(height: KansoSpacing.xs),
                          const Align(
                            alignment: AlignmentDirectional.centerStart,
                            child: KansoSkeleton(width: 120, height: 10),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
