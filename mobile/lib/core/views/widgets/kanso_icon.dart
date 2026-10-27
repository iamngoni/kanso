// Kanso — kanso_icon.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import 'kanso_icon_set_scope.dart';
import 'kanso_themed_icon.dart';

/// One of Kanso's [KansoIcons], drawn in the icon theme in use: the theme's
/// SVG for it (see [KansoIconSetScope]), or the bundled Lucide SVG where the
/// theme leaves it out.
///
/// The colour defaults to the surrounding [IconTheme], then to the theme's
/// text colour. Pass [semanticLabel] when the icon carries meaning of its
/// own; otherwise it is hidden from screen readers.
class KansoIcon extends StatelessWidget {
  const KansoIcon(
    this.icon, {
    this.size = 20,
    this.color,
    this.semanticLabel,
    super.key,
  });

  final KansoIcons icon;
  final double size;
  final Color? color;
  final String? semanticLabel;

  @override
  Widget build(BuildContext context) {
    return KansoThemedIcon(
      lucideName: icon.lucideName,
      svg: KansoIconSetScope.of(context).svgFor(icon.lucideName),
      size: size,
      color: color,
      semanticLabel: semanticLabel,
    );
  }
}
