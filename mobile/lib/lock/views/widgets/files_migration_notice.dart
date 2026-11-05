// Kanso — files_migration_notice.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// A short line afloat at the foot of the screen about sealing the
/// attachments of locked notes: a quiet reminder with "Seal now", or how a
/// pass went. It is a live region, and [onClose] puts it away.
class FilesMigrationNotice extends StatelessWidget {
  const FilesMigrationNotice({
    required this.message,
    required this.closeLabel,
    required this.onClose,
    this.actionLabel,
    this.onAction,
    super.key,
  });

  final String message;
  final String closeLabel;
  final VoidCallback onClose;
  final String? actionLabel;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final label = actionLabel;
    return Container(
      padding: const EdgeInsetsDirectional.only(
        start: KansoSpacing.md,
        top: KansoSpacing.xxs,
        bottom: KansoSpacing.xxs,
      ),
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.mediumBorder,
        border: Border.all(color: colors.divider),
        boxShadow: KansoShadows.floating(context.brightness),
      ),
      child: Row(
        children: <Widget>[
          Expanded(
            child: Semantics(
              liveRegion: true,
              child: Text(
                message,
                style: typography.secondary.copyWith(color: colors.text),
              ),
            ),
          ),
          if (label != null)
            KansoPressable(
              onTap: onAction,
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: Text(
                  label,
                  style: typography.body.copyWith(color: colors.accent),
                ),
              ),
            ),
          KansoPressable(
            onTap: onClose,
            child: Padding(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              child: Text(
                closeLabel,
                style: typography.body.copyWith(color: colors.textMuted),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
