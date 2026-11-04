// Kanso — feedback_entry_sheet.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/feedback_entry.dart';
import '../../utils/feedback_labels.dart';

/// A report in full: its first line, where it stands as a timeline, Open on
/// GitHub once it is filed, and Delete.
class FeedbackEntrySheet extends StatelessWidget {
  const FeedbackEntrySheet({
    required this.entry,
    required this.onDelete,
    super.key,
  });

  final FeedbackEntry entry;

  /// Asks to delete the report (the page confirms first).
  final VoidCallback onDelete;

  /// Shows the sheet over the app.
  static Future<void> show(
    BuildContext context,
    FeedbackEntry entry, {
    required VoidCallback onDelete,
  }) => showKansoBottomSheet<void>(
    context,
    builder: (_) => FeedbackEntrySheet(entry: entry, onDelete: onDelete),
  );

  /// The steps a report goes through, up to the one it is at. Reports that
  /// were not filed, matched another or were closed have no steps.
  static int? stage(FeedbackChip chip) => switch (chip) {
    FeedbackChip.waitingToSend => 0,
    FeedbackChip.received => 1,
    FeedbackChip.filedAs => 2,
    FeedbackChip.building => 3,
    FeedbackChip.fixProposed => 4,
    FeedbackChip.merged => 5,
    FeedbackChip.releasedIn => 6,
    FeedbackChip.notFiled ||
    FeedbackChip.sameAs ||
    FeedbackChip.closed ||
    FeedbackChip.deleting => null,
  };

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final reached = stage(entry.chip);
    final steps = <String>[
      l10n.feedbackChipWaiting,
      l10n.feedbackChipReceived,
      l10n.feedbackStepFiled,
      l10n.feedbackChipBuilding,
      l10n.feedbackChipFixProposed,
      l10n.feedbackChipMerged,
      if (entry.release == null)
        l10n.feedbackChipReleased('')
      else
        l10n.feedbackChipReleased(entry.release!),
    ];
    final detail = l10n.feedbackChipDetail(entry);
    final url = entry.issueUrl;
    return Padding(
      padding: const EdgeInsetsDirectional.fromSTEB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(
            entry.preview.isEmpty ? l10n.feedbackMine : entry.preview,
            maxLines: 3,
            overflow: TextOverflow.ellipsis,
            style: typography.barTitle,
          ),
          const SizedBox(height: KansoSpacing.sm),
          Text(l10n.feedbackChipLabel(entry), style: typography.body),
          if (detail != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            Text(detail, style: typography.secondary),
          ],
          if (reached != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.md),
            Text(l10n.feedbackTimeline, style: typography.secondary),
            const SizedBox(height: KansoSpacing.xs),
            for (var i = 1; i < steps.length; i++)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
                child: Row(
                  children: <Widget>[
                    KansoIcon(
                      i <= reached
                          ? KansoIcons.circleCheck
                          : KansoIcons.circleDot,
                      color: i <= reached
                          ? colors.accent
                          : colors.textSecondary,
                    ),
                    const SizedBox(width: KansoSpacing.sm),
                    Expanded(
                      child: Text(
                        steps[i],
                        style: typography.body.copyWith(
                          color: i <= reached
                              ? colors.text
                              : colors.textSecondary,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
          ],
          const SizedBox(height: KansoSpacing.md),
          if (url != null) ...<Widget>[
            KansoButton(
              label: l10n.feedbackOpenOnGithub,
              icon: KansoIcons.externalLink,
              onPressed: () {
                launchUrl(Uri.parse(url), mode: LaunchMode.externalApplication);
              },
            ),
            const SizedBox(height: KansoSpacing.sm),
          ],
          KansoButton(
            label: l10n.feedbackDelete,
            variant: KansoButtonVariant.danger,
            onPressed: entry.deleting
                ? null
                : () {
                    Navigator.of(context).pop();
                    onDelete();
                  },
          ),
        ],
      ),
    );
  }
}
