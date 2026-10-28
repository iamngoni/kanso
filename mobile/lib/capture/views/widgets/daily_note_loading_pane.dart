// Kanso — daily_note_loading_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// What shows while today's note is found or made: still bars where the
/// note's title and first lines will be, and a line that says what is
/// happening.
class DailyNoteLoadingPane extends StatelessWidget {
  const DailyNoteLoadingPane({super.key});

  /// The widest the bars grow, so a tablet's are not stretched across it.
  static const double maxWidth = 360;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.md),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: maxWidth),
          child: Semantics(
            container: true,
            liveRegion: true,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                const FractionallySizedBox(
                  alignment: AlignmentDirectional.centerStart,
                  widthFactor: 0.6,
                  child: KansoSkeleton(height: 24),
                ),
                const SizedBox(height: KansoSpacing.md),
                const KansoSkeleton(),
                const SizedBox(height: KansoSpacing.sm),
                const FractionallySizedBox(
                  alignment: AlignmentDirectional.centerStart,
                  widthFactor: 0.85,
                  child: KansoSkeleton(),
                ),
                const SizedBox(height: KansoSpacing.sm),
                const FractionallySizedBox(
                  alignment: AlignmentDirectional.centerStart,
                  widthFactor: 0.5,
                  child: KansoSkeleton(),
                ),
                const SizedBox(height: KansoSpacing.lg),
                Text(
                  context.l10n.dailyNoteLoading,
                  textAlign: TextAlign.center,
                  style: context.typography.secondary,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
