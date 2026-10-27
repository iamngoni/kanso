// Kanso — kanso_sidebar_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_lock_mark.dart';
import 'kanso_pressable.dart';

/// A row of the library: an icon (or any [leading] widget), a label, a
/// count and, for a notebook, a chevron into it.
///
/// 40 points tall; [selected] fills it with the sidebar's active colour.
/// [depth] indents a notebook inside another. A notebook with others inside
/// it shows [foldOpen] instead of the chevron: an arrow that opens or folds
/// them, tapped on its own.
class KansoSidebarRow extends StatelessWidget {
  const KansoSidebarRow({
    required this.label,
    this.icon,
    this.leading,
    this.count,
    this.badge,
    this.selected = false,
    this.chevron = false,
    this.depth = 0,
    this.locked = false,
    this.foldOpen,
    this.onFold,
    this.wash,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final String label;
  final KansoIcons? icon;

  /// Replaces [icon], for a coloured dot.
  final Widget? leading;
  final int? count;

  /// Shown at the end of the row in place of [count]: a pill.
  final Widget? badge;
  final bool selected;
  final bool chevron;
  final int depth;

  /// Whether the notebook is locked: a lock is drawn after its label.
  final bool locked;

  /// Whether the notebooks inside this one show, for the arrow that opens
  /// and folds them; null when there are none inside.
  final bool? foldOpen;

  /// What a tap on that arrow does.
  final VoidCallback? onFold;

  /// A notebook's colour, washing the row as the desktop does: light while
  /// resting, stronger while [selected].
  final Color? wash;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  /// How far each level of [depth] moves a row in.
  static const double depthIndent = KansoSpacing.md;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final glyph = icon;
    final head = leading;
    final total = count;
    final pill = badge;
    final fold = foldOpen;
    final tint = wash;
    return KansoPressable(
      onTap: onTap,
      onLongPress: onLongPress,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xs),
        child: Container(
          height: KansoSizes.sidebarRow,
          padding: EdgeInsetsDirectional.only(
            start: KansoSpacing.sm + depth * depthIndent,
            end: KansoSpacing.sm,
          ),
          decoration: BoxDecoration(
            color: tint == null
                ? (selected ? colors.sidebarActive : null)
                : tint.withValues(alpha: selected ? 0.34 : 0.16),
            borderRadius: KansoRadii.smallBorder,
          ),
          child: Row(
            children: <Widget>[
              SizedBox(
                width: KansoSizes.icon,
                child: Center(
                  child:
                      head ??
                      (glyph == null
                          ? null
                          : KansoIcon(
                              glyph,
                              color: selected
                                  ? colors.text
                                  : colors.textSecondary,
                            )),
                ),
              ),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Text(
                  label,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: typography.body.copyWith(
                    fontWeight: selected ? FontWeight.w600 : null,
                  ),
                ),
              ),
              if (locked) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                const KansoLockMark(),
              ],
              if (pill != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                pill,
              ] else if (total != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                Text(
                  '$total',
                  style: typography.secondary.copyWith(color: colors.textMuted),
                ),
              ],
              if (fold != null) ...<Widget>[
                const SizedBox(width: KansoSpacing.xxs),
                KansoPressable(
                  onTap: onFold,
                  semanticLabel: label,
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.xxs),
                    child: KansoIcon(
                      fold ? KansoIcons.chevronDown : KansoIcons.chevronRight,
                      size: KansoSizes.iconSmall,
                      color: colors.textMuted,
                    ),
                  ),
                ),
              ] else if (chevron) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                KansoIcon(
                  KansoIcons.chevronRight,
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
