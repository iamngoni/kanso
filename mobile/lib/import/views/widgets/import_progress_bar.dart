// Kanso: import_progress_bar.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/utils/context_extensions.dart';

/// How far something has got, as a thin bar: [fraction] of the way from 0
/// to 1. It never animates by itself, so a picture of it is steady.
class ImportProgressBar extends StatelessWidget {
  const ImportProgressBar({required this.fraction, super.key});

  final double fraction;

  /// How tall the bar is.
  static const double height = 6;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      value: '${(fraction.clamp(0, 1) * 100).round()}%',
      child: ClipRRect(
        borderRadius: KansoRadii.smallBorder,
        child: SizedBox(
          height: height,
          child: Stack(
            children: <Widget>[
              Positioned.fill(child: ColoredBox(color: colors.divider)),
              Positioned.fill(
                child: FractionallySizedBox(
                  alignment: Alignment.centerLeft,
                  widthFactor: fraction.clamp(0, 1).toDouble(),
                  child: ColoredBox(color: colors.accent),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
