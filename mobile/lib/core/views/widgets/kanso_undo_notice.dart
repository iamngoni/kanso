// Kanso — kanso_undo_notice.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_pressable.dart';

/// A short line afloat at the foot of the screen that says what a change
/// did and offers to take it back, like the desktop's notice.
///
/// It is a live region, so a screen reader reads it when it appears; Undo
/// is a button of its own. While the change is being taken back ([busy])
/// Undo does nothing.
class KansoUndoNotice extends StatelessWidget {
  const KansoUndoNotice({
    required this.message,
    required this.onUndo,
    this.busy = false,
    super.key,
  });

  final String message;
  final VoidCallback onUndo;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
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
        boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
      ),
      child: Row(
        children: <Widget>[
          Expanded(
            child: Semantics(
              liveRegion: true,
              child: Text(
                message,
                style: context.typography.secondary.copyWith(
                  color: colors.text,
                ),
              ),
            ),
          ),
          KansoPressable(
            onTap: busy ? null : onUndo,
            child: Padding(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              child: Text(
                l10n.undoNoticeUndo,
                style: context.typography.body.copyWith(
                  color: busy ? colors.textMuted : colors.accent,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
