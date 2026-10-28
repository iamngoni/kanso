// Kanso — kanso_settings_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// A row of settings: an optional icon, a label, the current [value] and
/// what follows it ([trailing], a [chevron] or an [external] link mark).
///
/// A [danger] row is drawn in the danger colour; [centered] puts its label
/// in the middle, for an action such as signing out.
class KansoSettingsRow extends StatelessWidget {
  const KansoSettingsRow({
    required this.label,
    this.icon,
    this.value,
    this.trailing,
    this.chevron = false,
    this.external = false,
    this.danger = false,
    this.centered = false,
    this.onTap,
    super.key,
  });

  final String label;
  final KansoIcons? icon;
  final String? value;
  final Widget? trailing;
  final bool chevron;
  final bool external;
  final bool danger;
  final bool centered;
  final VoidCallback? onTap;

  /// The widest a value may grow before it is cut short.
  static const double valueMaxWidth = 220;

  /// Where the label of a row with an icon starts, for the hairlines
  /// between such rows.
  static const double iconInset = 48;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final glyph = icon;
    final note = value;
    final end = trailing;
    final ink = danger ? colors.danger : colors.text;
    final labelText = Text(
      label,
      textAlign: centered ? TextAlign.center : TextAlign.start,
      style: typography.body.copyWith(
        color: ink,
        fontWeight: danger ? FontWeight.w500 : null,
      ),
    );
    return KansoPressable(
      onTap: onTap,
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              if (glyph != null) ...<Widget>[
                KansoIcon(
                  glyph,
                  color: danger ? colors.danger : colors.textSecondary,
                ),
                const SizedBox(width: KansoSpacing.sm),
              ],
              if (centered)
                Expanded(child: labelText)
              else
                Expanded(child: labelText),
              if (note != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: valueMaxWidth),
                  child: Text(
                    note,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.secondary,
                  ),
                ),
              ],
              if (end != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                end,
              ],
              if (chevron) ...<Widget>[
                const SizedBox(width: KansoSpacing.xxs),
                KansoIcon(
                  KansoIcons.chevronRight,
                  size: KansoSizes.iconSmall,
                  color: colors.textMuted,
                ),
              ],
              if (external) ...<Widget>[
                const SizedBox(width: KansoSpacing.xxs),
                KansoIcon(
                  KansoIcons.externalLink,
                  size: KansoSizes.iconSmall,
                  color: colors.textMuted,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
