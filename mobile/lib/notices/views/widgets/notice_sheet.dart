// Kanso — notice_sheet.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/notices_types.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/sheet_header.dart';

/// The full notice in a sheet: what changed, the numbered steps that fix
/// it, and the buttons. The first button does what the notice says (opens
/// the page of Settings it names); Mark as fixed and Dismiss put the card
/// away.
class NoticeSheet extends StatelessWidget {
  const NoticeSheet({
    required this.notice,
    required this.onAction,
    required this.onMarkFixed,
    required this.onDismiss,
    super.key,
  });

  final PhoneNotice notice;

  /// Opens what the notice's button names; null without a button.
  final VoidCallback? onAction;
  final VoidCallback onMarkFixed;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final action = onAction;
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.md),
      children: <Widget>[
        SheetHeader(
          title: l10n.noticeSheetTitle,
          actionLabel: l10n.noticeSheetDone,
        ),
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            0,
          ),
          child: Text(notice.title, style: typography.heading2),
        ),
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.md,
          ),
          child: Text(notice.whatChanged, style: typography.secondary),
        ),
        for (var i = 0; i < notice.steps.length; i++)
          Padding(
            padding: const EdgeInsetsDirectional.fromSTEB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.sm,
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Container(
                  width: KansoSizes.iconSmall + 8,
                  height: KansoSizes.iconSmall + 8,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: colors.text.withValues(alpha: 0.08),
                    shape: BoxShape.circle,
                  ),
                  child: Text(
                    '${i + 1}',
                    style: typography.secondary.copyWith(
                      color: colors.textSecondary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Expanded(
                  child: Padding(
                    padding: const EdgeInsets.only(top: 1),
                    child: Text(notice.steps[i], style: typography.body),
                  ),
                ),
              ],
            ),
          ),
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            0,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              if (action != null) ...<Widget>[
                KansoButton(label: notice.actionLabel, onPressed: action),
                const SizedBox(height: KansoSpacing.xs),
              ],
              KansoButton(
                label: l10n.noticeMarkFixed,
                variant: action == null
                    ? KansoButtonVariant.primary
                    : KansoButtonVariant.secondary,
                onPressed: onMarkFixed,
              ),
              const SizedBox(height: KansoSpacing.xs),
              KansoButton(
                label: l10n.noticeDismiss,
                variant: KansoButtonVariant.quiet,
                onPressed: onDismiss,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
