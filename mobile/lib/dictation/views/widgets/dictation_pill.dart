// Kanso — dictation_pill.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The pill that shows above the keyboard's accessory bar while the phone
/// listens: the words being spoken, which are not in the note until the
/// recognizer has them right, and a Stop. It also carries the notice that
/// dictation could not run, with the one thing to do about it.
class DictationPill extends StatelessWidget {
  const DictationPill({
    required this.text,
    required this.actionLabel,
    required this.onAction,
    this.quiet = false,
    this.icon = KansoIcons.mic,
    super.key,
  });

  /// What it says: the words being spoken, or the notice.
  final String text;

  /// Whether [text] is a placeholder, drawn quietly.
  final bool quiet;
  final KansoIcons icon;

  /// The label of the one action: Stop, Open Settings or OK.
  final String actionLabel;
  final VoidCallback onAction;

  /// How many characters of the end of a long phrase the pill shows.
  static const int shown = 120;

  /// The end of [text], with an ellipsis where it was cut.
  static String tail(String text) =>
      text.length <= shown ? text : '…${text.substring(text.length - shown)}';

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Container(
      constraints: const BoxConstraints(maxWidth: KansoSizes.sheetWidth),
      padding: const EdgeInsets.only(left: KansoSpacing.md),
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: colors.divider),
        boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
      ),
      child: Row(
        children: <Widget>[
          KansoIcon(icon, color: colors.accent),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: KansoSpacing.sm),
              child: Text(
                tail(text),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: typography.body.copyWith(
                  color: quiet ? colors.textMuted : colors.text,
                ),
              ),
            ),
          ),
          KansoPressable(
            onTap: onAction,
            child: Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: KansoSpacing.md,
                vertical: KansoSpacing.sm + KansoSpacing.xxs,
              ),
              child: Text(
                actionLabel,
                style: typography.listTitle.copyWith(color: colors.accent),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
