// Kanso — citation_chip.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// A citation or a link in an answer, in the line of the text it is in: a
/// note's title on a wash of the link colour, or a web address underlined.
class CitationChip extends StatelessWidget {
  const CitationChip({
    required this.label,
    required this.onTap,
    this.web = false,
    super.key,
  });

  final String label;
  final VoidCallback onTap;

  /// Whether it links to a web page and not to a note.
  final bool web;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final ink = web ? colors.link : colors.accent;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: label,
      child: Container(
        padding: EdgeInsets.symmetric(horizontal: web ? 0 : 6),
        decoration: web
            ? null
            : BoxDecoration(
                color: ink.withValues(alpha: KansoEffects.wash),
                borderRadius: KansoRadii.smallBorder,
              ),
        child: Text(
          label,
          style: context.typography.body.copyWith(
            color: ink,
            decoration: web ? TextDecoration.underline : null,
            decorationColor: ink,
          ),
        ),
      ),
    );
  }
}
