// Kanso — discovery_status_row.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';

/// The live local model status, including progress while it is being prepared.
class DiscoveryStatusRow extends StatelessWidget {
  const DiscoveryStatusRow({
    required this.status,
    required this.onRetry,
    super.key,
  });

  final RelatedState? status;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final current = status;
    if (current == null) return const SizedBox.shrink();
    final statusState = current.status;
    final progress = switch (statusState) {
      RelatedStatus_Downloading(:final done, :final total) ||
      RelatedStatus_Indexing(
        :final done,
        :final total,
      ) => total <= 0 ? null : done / total,
      _ => null,
    };
    final failed = statusState is RelatedStatus_Failed;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.sm,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(current.title ?? '', style: context.typography.label),
          if (current.detail case final detail?) ...<Widget>[
            const SizedBox(height: KansoSpacing.xxs),
            Text(
              detail,
              style: context.typography.secondary.copyWith(
                color: context.colors.textSecondary,
              ),
            ),
          ],
          if (progress != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            LinearProgressIndicator(
              value: progress.clamp(0, 1),
              minHeight: 2,
              color: context.colors.accent,
              backgroundColor: context.colors.selected,
            ),
          ],
          if (failed)
            TextButton(
              onPressed: onRetry,
              child: Text(context.l10n.discoveryTryAgain),
            ),
        ],
      ),
    );
  }
}
