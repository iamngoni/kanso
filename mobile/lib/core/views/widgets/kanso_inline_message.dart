// Kanso — kanso_inline_message.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';

/// How a [KansoInlineMessage] weighs on the person.
enum KansoMessageTone {
  /// Something went wrong: the danger colour.
  error,

  /// Something went through: the success colour.
  success,

  /// Something to know: quiet.
  info,
}

/// A line or two on a quiet wash of its colour, inside a form: why a call
/// failed, what a change did.
class KansoInlineMessage extends StatelessWidget {
  const KansoInlineMessage({
    required this.message,
    this.tone = KansoMessageTone.error,
    super.key,
  });

  final String message;
  final KansoMessageTone tone;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final (ink, glyph) = switch (tone) {
      KansoMessageTone.error => (colors.danger, KansoIcons.circleX),
      KansoMessageTone.success => (colors.success, KansoIcons.circleCheck),
      KansoMessageTone.info => (colors.textSecondary, KansoIcons.info),
    };
    return Semantics(
      liveRegion: true,
      child: Container(
        padding: const EdgeInsets.all(KansoSpacing.sm),
        decoration: BoxDecoration(
          color: ink.withValues(alpha: KansoEffects.quietWash),
          borderRadius: KansoRadii.mediumBorder,
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: KansoIcon(glyph, size: KansoSizes.iconSmall, color: ink),
            ),
            const SizedBox(width: KansoSpacing.xs),
            Expanded(
              child: Text(
                message,
                style: context.typography.secondary.copyWith(color: ink),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
