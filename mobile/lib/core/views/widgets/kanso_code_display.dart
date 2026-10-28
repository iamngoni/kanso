// Kanso — kanso_code_display.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// A code to read off or write down, large, in the code font, in a quiet
/// panel: the code GitHub asks for, the six digits two devices compare, a
/// recovery key.
///
/// The text is shown as it is given, grouped by whoever made it. Give
/// [semanticLabel] when the screen reader should say it another way (digit
/// by digit).
class KansoCodeDisplay extends StatelessWidget {
  const KansoCodeDisplay({
    required this.text,
    this.caption,
    this.semanticLabel,
    this.fontSize = 32,
    super.key,
  });

  final String text;

  /// A quiet line under the code.
  final String? caption;
  final String? semanticLabel;

  /// The code's size in points.
  final double fontSize;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final note = caption;
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(
        vertical: KansoSpacing.lg,
        horizontal: KansoSpacing.md,
      ),
      decoration: BoxDecoration(
        color: colors.list,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: colors.divider),
      ),
      child: Column(
        children: <Widget>[
          Semantics(
            label: semanticLabel,
            child: ExcludeSemantics(
              excluding: semanticLabel != null,
              child: Text(
                text,
                textAlign: TextAlign.center,
                style: typography.mono.copyWith(
                  fontSize: fontSize,
                  height: 1.3,
                  letterSpacing: 1,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
          ),
          if (note != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            Text(note, textAlign: TextAlign.center, style: typography.meta),
          ],
        ],
      ),
    );
  }
}
