// Kanso — notice_card.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/notices_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The card at the top of the notes list when an update changed something
/// this phone may need to fix: what it is, how many are waiting, and two
/// quiet buttons, How to fix and Dismiss.
///
/// It sits on the list's own colour with a hairline and a warning wash on
/// its glyph, so it reads as a notice and not as an alarm.
class NoticeCard extends StatelessWidget {
  const NoticeCard({
    required this.notice,
    required this.countLabel,
    required this.onHowToFix,
    required this.onDismiss,
    super.key,
  });

  final PhoneNotice notice;

  /// "1 of 2", or empty when it is the only one.
  final String countLabel;
  final VoidCallback onHowToFix;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    // One container, read in order: the heading, the count, the title and
    // what changed, then the two buttons.
    return Semantics(
      container: true,
      child: Padding(
        padding: const EdgeInsetsDirectional.fromSTEB(
          KansoSpacing.md,
          KansoSpacing.xs,
          KansoSpacing.md,
          KansoSpacing.xs,
        ),
        child: Container(
          padding: const EdgeInsets.all(KansoSpacing.md),
          decoration: BoxDecoration(
            color: colors.elevated,
            borderRadius: KansoRadii.largeBorder,
            border: Border.all(color: colors.divider),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Row(
                children: <Widget>[
                  Container(
                    width: KansoSizes.chip,
                    height: KansoSizes.chip,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: colors.warning.withValues(alpha: 0.16),
                      shape: BoxShape.circle,
                    ),
                    child: KansoIcon(
                      KansoIcons.triangleAlert,
                      size: KansoSizes.iconSmall,
                      color: colors.warning,
                    ),
                  ),
                  const SizedBox(width: KansoSpacing.sm),
                  Expanded(
                    child: Text(
                      l10n.noticeNeedsAttention,
                      style: typography.secondary.copyWith(
                        color: colors.textSecondary,
                      ),
                    ),
                  ),
                  if (countLabel.isNotEmpty)
                    Text(
                      countLabel,
                      style: typography.secondary.copyWith(
                        color: colors.textMuted,
                      ),
                    ),
                ],
              ),
              const SizedBox(height: KansoSpacing.sm),
              Semantics(
                header: true,
                child: Text(notice.title, style: typography.heading2),
              ),
              const SizedBox(height: KansoSpacing.xxs),
              Text(
                notice.whatChanged,
                maxLines: 3,
                overflow: TextOverflow.ellipsis,
                style: typography.secondary,
              ),
              const SizedBox(height: KansoSpacing.xs),
              Row(
                children: <Widget>[
                  _CardButton(
                    label: l10n.noticeHowToFix,
                    color: colors.accent,
                    onTap: onHowToFix,
                  ),
                  const SizedBox(width: KansoSpacing.xs),
                  _CardButton(
                    label: l10n.noticeDismiss,
                    color: colors.textSecondary,
                    onTap: onDismiss,
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A text button that is a full touch target tall, set against the card's
/// leading edge.
class _CardButton extends StatelessWidget {
  const _CardButton({
    required this.label,
    required this.color,
    required this.onTap,
  });

  final String label;
  final Color color;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return KansoPressable(
      onTap: onTap,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: KansoSizes.touchTarget),
        child: Padding(
          padding: const EdgeInsetsDirectional.only(end: KansoSpacing.sm),
          child: Align(
            alignment: AlignmentDirectional.centerStart,
            widthFactor: 1,
            child: Text(
              label,
              style: context.typography.body.copyWith(
                color: color,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
