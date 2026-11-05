// Kanso — interrupted_lock_bar.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../state/interrupted_lock/interrupted_lock_bloc.dart';
import '../../utils/seal_progress_rules.dart';

/// The bar above a note whose lock stopped between its pictures and files:
/// what is sealed so far, and Finish locking or Undo locking. While one of
/// them runs it counts the files. It reads the `InterruptedLockBloc` above
/// it and draws nothing when no lock stopped.
class InterruptedLockBar extends StatelessWidget {
  const InterruptedLockBar({required this.gutter, super.key});

  /// The space either side, the page's.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<InterruptedLockBloc, InterruptedLockState>(
      buildWhen: (before, now) =>
          before.stage != now.stage ||
          before.files != now.files ||
          before.sealed != now.sealed ||
          before.progress != now.progress,
      builder: (context, state) {
        final shown = switch (state.stage) {
          InterruptedStage.checking || InterruptedStage.none => false,
          _ => true,
        };
        if (!shown) {
          return const SizedBox.shrink();
        }
        final l10n = context.l10n;
        final colors = context.colors;
        final typography = context.typography;
        final working = state.stage == InterruptedStage.working;
        final bloc = context.read<InterruptedLockBloc>();
        final progress = state.progress;
        return Padding(
          padding: EdgeInsets.fromLTRB(gutter, 0, gutter, KansoSpacing.xxs),
          child: Semantics(
            liveRegion: true,
            container: true,
            child: Container(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              decoration: BoxDecoration(
                color: colors.warning.withValues(alpha: 0.14),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  Text(
                    l10n.lockInterruptedMessage(state.sealed, state.files),
                    style: typography.secondary,
                  ),
                  if (state.stage == InterruptedStage.failed) ...<Widget>[
                    const SizedBox(height: KansoSpacing.xxs),
                    Text(
                      l10n.lockInterruptedFailed,
                      style: typography.secondary.copyWith(
                        color: colors.danger,
                      ),
                    ),
                  ],
                  if (working && progress != null) ...<Widget>[
                    const SizedBox(height: KansoSpacing.xs),
                    LinearProgressIndicator(
                      value: sealFraction(progress),
                      minHeight: 4,
                      color: colors.accent,
                      backgroundColor: colors.fill,
                    ),
                  ],
                  const SizedBox(height: KansoSpacing.xs),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: <Widget>[
                      KansoButton(
                        label: working
                            ? l10n.lockInterruptedWorking
                            : l10n.lockInterruptedUndo,
                        variant: KansoButtonVariant.quiet,
                        expanded: false,
                        compact: true,
                        onPressed: working
                            ? null
                            : () => bloc.add(
                                const InterruptedLockEvent.undone(),
                              ),
                      ),
                      const SizedBox(width: KansoSpacing.xs),
                      KansoButton(
                        label: l10n.lockInterruptedFinish,
                        expanded: false,
                        compact: true,
                        onPressed: working
                            ? null
                            : () => bloc.add(
                                const InterruptedLockEvent.finished(),
                              ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}
