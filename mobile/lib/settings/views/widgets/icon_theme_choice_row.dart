// Kanso — icon_theme_choice_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart' show IconThemeRow;
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import 'icon_theme_preview_strip.dart';

/// One icon theme to choose: its name, who made it and under which licence,
/// a row of its icons, and a check on the one in use.
class IconThemeChoiceRow extends StatelessWidget {
  const IconThemeChoiceRow({
    required this.theme,
    required this.selected,
    required this.onTap,
    super.key,
  });

  final IconThemeRow theme;
  final bool selected;
  final VoidCallback onTap;

  /// The least a row measures: its two lines and the icons.
  static const double minHeight = 88;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    final credit = switch ((theme.author.isEmpty, theme.license.isEmpty)) {
      (false, false) => l10n.iconsCredit(theme.author, theme.license),
      (false, true) => theme.author,
      (true, false) => theme.license,
      (true, true) => null,
    };
    return Semantics(
      selected: selected,
      inMutuallyExclusiveGroup: true,
      child: KansoPressable(
        onTap: onTap,
        semanticLabel: theme.name,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: minHeight),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.sm,
            ),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(
                        theme.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.listTitle,
                      ),
                      if (credit != null) ...<Widget>[
                        const SizedBox(height: KansoSpacing.xxs),
                        Text(
                          credit,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: typography.secondary,
                        ),
                      ],
                      const SizedBox(height: KansoSpacing.xs),
                      IconThemePreviewStrip(theme: theme),
                    ],
                  ),
                ),
                SizedBox(
                  width: KansoSizes.touchTarget - KansoSpacing.xs,
                  child: selected
                      ? Align(
                          alignment: AlignmentDirectional.centerEnd,
                          child: KansoIcon(
                            KansoIcons.check,
                            color: colors.accent,
                            semanticLabel: l10n.themesCurrent,
                          ),
                        )
                      : null,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
