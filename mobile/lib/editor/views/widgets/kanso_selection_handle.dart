// Kanso — kanso_selection_handle.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';

/// A handle at an end of the selection in the editor: a stem the height of
/// the line in the accent colour with a round knob at one end, the same on
/// iOS and Android.
///
/// The start of a selection has its knob above the line; the end and the
/// caret have theirs below.
class KansoSelectionHandle extends StatelessWidget {
  const KansoSelectionHandle({
    required this.type,
    required this.lineHeight,
    super.key,
  });

  final TextSelectionHandleType type;

  /// The height of the line the handle is at.
  final double lineHeight;

  /// The width of the handle, the touch target of its knob.
  static const double width = 24;

  /// How big the knob is across.
  static const double knob = 10;

  /// How wide the stem is.
  static const double stem = 2;

  /// The size of a handle for a line [lineHeight] high.
  static Size sizeFor(double lineHeight) => Size(width, lineHeight + knob);

  /// Where, in the handle, the end of the line it marks is: the bottom of
  /// the stem. The framework puts this point on the end of the selection.
  static Offset anchorFor(TextSelectionHandleType type, double lineHeight) =>
      type == TextSelectionHandleType.left
      ? Offset(width / 2, lineHeight + knob)
      : Offset(width / 2, lineHeight);

  @override
  Widget build(BuildContext context) {
    final color = context.colors.accent;
    final above = type == TextSelectionHandleType.left;
    final stemTop = above ? knob : 0.0;
    return SizedBox.fromSize(
      size: sizeFor(lineHeight),
      child: Stack(
        children: <Widget>[
          Positioned(
            left: (width - stem) / 2,
            top: stemTop,
            width: stem,
            height: lineHeight,
            child: ColoredBox(color: color),
          ),
          Positioned(
            left: (width - knob) / 2,
            top: above ? 0 : lineHeight,
            width: knob,
            height: knob,
            child: DecoratedBox(
              decoration: BoxDecoration(color: color, shape: BoxShape.circle),
            ),
          ),
        ],
      ),
    );
  }
}
