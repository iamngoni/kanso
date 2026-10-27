// Kanso — accessory_key.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// A key of the keyboard accessory bar: an [icon], or a text [glyph] when
/// Lucide has no icon for it. A key with no [onTap] is dimmed: there is
/// nothing for it to do now.
class AccessoryKey extends StatelessWidget {
  const AccessoryKey({
    required this.semanticLabel,
    this.icon,
    this.glyph,
    this.onTap,
    this.onLongPress,
    super.key,
  }) : assert(
         (icon == null) != (glyph == null),
         'Give an icon or a glyph.',
       );

  final String semanticLabel;
  final KansoIcons? icon;
  final String? glyph;
  final VoidCallback? onTap;

  /// What holding the key does.
  final VoidCallback? onLongPress;

  /// How much of its colour a key with nothing to do keeps.
  static const double dimmed = 0.4;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final mark = icon;
    final tint = onTap == null
        ? colors.textSecondary.withValues(alpha: dimmed)
        : colors.textSecondary;
    return KansoPressable(
      onTap: onTap,
      onLongPress: onLongPress,
      semanticLabel: semanticLabel,
      child: SizedBox.square(
        dimension: KansoSizes.touchTarget,
        child: Center(
          child: mark != null
              ? KansoIcon(mark, color: tint)
              // The key's label says what it does; its glyph is not a word.
              : ExcludeSemantics(
                  child: Text(
                    glyph!,
                    style: context.typography.body.copyWith(
                      color: tint,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
        ),
      ),
    );
  }
}
