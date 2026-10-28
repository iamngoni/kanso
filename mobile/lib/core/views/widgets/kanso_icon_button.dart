// Kanso — kanso_icon_button.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// An icon that does something: a 44 point target around a 20 point icon.
///
/// [selected] draws the icon in the accent colour on a soft wash, for a
/// control that is on. [semanticLabel] says what it does, for screen
/// readers; there is no visible label. [onLongPress] is what holding it
/// does, offered to screen readers as an action too.
class KansoIconButton extends StatelessWidget {
  const KansoIconButton({
    required this.icon,
    required this.semanticLabel,
    this.onPressed,
    this.onLongPress,
    this.color,
    this.selected = false,
    this.iconSize = KansoSizes.icon,
    super.key,
  });

  final KansoIcons icon;
  final String semanticLabel;
  final VoidCallback? onPressed;
  final VoidCallback? onLongPress;

  /// The icon's colour; the secondary text colour when null.
  final Color? color;
  final bool selected;
  final double iconSize;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final enabled = onPressed != null;
    final tint = selected ? colors.accent : color ?? colors.textSecondary;
    return KansoPressable(
      onTap: onPressed,
      onLongPress: onLongPress,
      semanticLabel: semanticLabel,
      scale: KansoEffects.pressScale,
      child: SizedBox.square(
        dimension: KansoSizes.touchTarget,
        child: Center(
          child: Container(
            width: KansoSizes.touchTarget - 8,
            height: KansoSizes.touchTarget - 8,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: selected
                  ? colors.accent.withValues(alpha: KansoEffects.wash)
                  : null,
              borderRadius: KansoRadii.mediumBorder,
            ),
            child: Opacity(
              opacity: enabled ? 1 : KansoEffects.disabled,
              child: KansoIcon(icon, size: iconSize, color: tint),
            ),
          ),
        ),
      ),
    );
  }
}
