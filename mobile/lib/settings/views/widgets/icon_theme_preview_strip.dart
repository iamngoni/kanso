// Kanso — icon_theme_preview_strip.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart' show IconThemeRow;
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_themed_icon.dart';

/// A few icons of [theme], each drawn in the theme itself (not in the icon
/// theme in use), side by side.
class IconThemePreviewStrip extends StatelessWidget {
  const IconThemePreviewStrip({required this.theme, super.key});

  final IconThemeRow theme;

  /// How large each icon is drawn.
  static const double iconSize = 22;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      label: context.l10n.iconsPreview(theme.name),
      excludeSemantics: true,
      child: Row(
        children: <Widget>[
          for (final icon in theme.preview) ...<Widget>[
            KansoThemedIcon(
              lucideName: icon.name,
              svg: icon.svg,
              size: iconSize,
              color: colors.textSecondary,
            ),
            const SizedBox(width: KansoSpacing.md),
          ],
        ],
      ),
    );
  }
}
