// Kanso — feedback_entry_row.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../models/data/feedback_entry.dart';
import '../../utils/feedback_labels.dart';

/// One report in My feedback: its first line, a status chip and the date.
/// A report that was fixed and not yet looked at is bold.
class FeedbackEntryRow extends StatelessWidget {
  const FeedbackEntryRow({
    required this.entry,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final FeedbackEntry entry;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final date = MaterialLocalizations.of(context).formatShortDate(
      DateTime.fromMillisecondsSinceEpoch(entry.createdMs, isUtc: true),
    );
    final detail = l10n.feedbackChipDetail(entry);
    return KansoPressable(
      onTap: onTap,
      onLongPress: onLongPress,
      semanticLabel:
          '${entry.preview}, ${l10n.feedbackChipLabel(entry)}, $date',
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              entry.preview.isEmpty ? '…' : entry.preview,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: typography.body.copyWith(
                fontWeight: entry.fresh ? FontWeight.w700 : null,
              ),
            ),
            const SizedBox(height: KansoSpacing.xs),
            Row(
              children: <Widget>[
                KansoBadge(
                  label: l10n.feedbackChipLabel(entry),
                  tone: entry.fresh
                      ? KansoBadgeTone.accent
                      : KansoBadgeTone.neutral,
                ),
                const SizedBox(width: KansoSpacing.sm),
                Text(
                  date,
                  style: typography.secondary.copyWith(
                    color: colors.textSecondary,
                  ),
                ),
              ],
            ),
            if (detail != null) ...<Widget>[
              const SizedBox(height: KansoSpacing.xs),
              Text(detail, style: typography.secondary),
            ],
          ],
        ),
      ),
    );
  }
}
