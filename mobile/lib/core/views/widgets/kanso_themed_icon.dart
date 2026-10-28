// Kanso — kanso_themed_icon.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../configs/kanso_icons.dart';
import '../../utils/context_extensions.dart';

/// An icon drawn from an SVG a theme gives, or from the bundled Lucide icon
/// named [lucideName] where it gives none ([svg] is `null`).
///
/// The colour defaults to the surrounding [IconTheme], then to the theme's
/// text colour. A theme's SVG that cannot be drawn shows the Lucide icon.
/// `KansoIcon` is this for the icon theme in use; the Icons page draws each
/// theme's own with it.
class KansoThemedIcon extends StatelessWidget {
  const KansoThemedIcon({
    required this.lucideName,
    this.svg,
    this.size = 20,
    this.color,
    this.semanticLabel,
    super.key,
  });

  /// The icon's name in the Lucide set.
  final String lucideName;

  /// The theme's SVG, if it draws this icon.
  final String? svg;
  final double size;
  final Color? color;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    final resolved =
        color ?? IconTheme.of(context).color ?? context.colors.text;
    final filter = ColorFilter.mode(resolved, BlendMode.srcIn);
    final lucide = SvgPicture.asset(
      KansoIcons.assetPathOf(lucideName),
      width: size,
      height: size,
      colorFilter: filter,
      semanticsLabel: semanticLabel,
      excludeFromSemantics: semanticLabel == null,
    );
    final themed = svg;
    if (themed == null) {
      return lucide;
    }
    return SvgPicture.string(
      themed,
      width: size,
      height: size,
      colorFilter: filter,
      semanticsLabel: semanticLabel,
      excludeFromSemantics: semanticLabel == null,
      errorBuilder: (context, error, stackTrace) => lucide,
    );
  }
}
