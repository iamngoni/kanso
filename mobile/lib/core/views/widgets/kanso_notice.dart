// Kanso — kanso_notice.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_pressable.dart';

/// A short line afloat at the foot of a screen that says why something did
/// not happen, and goes when it has been read. Announced to a screen reader
/// as it appears.
class KansoNotice extends StatelessWidget {
  const KansoNotice({
    required this.message,
    required this.dismissLabel,
    required this.onDismiss,
    super.key,
  });

  final String message;

  /// The word on the button that closes it.
  final String dismissLabel;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      liveRegion: true,
      child: Container(
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
              child: Text(
                message,
                style: context.typography.secondary.copyWith(
                  color: colors.text,
                ),
              ),
            ),
            KansoPressable(
              onTap: onDismiss,
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: Text(
                  dismissLabel,
                  style: context.typography.body.copyWith(
                    color: colors.accent,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
