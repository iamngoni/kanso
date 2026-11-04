// Kanso — canvas_tool.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/configs/kanso_icons.dart';

/// What the pen, a finger or the eraser end does on a canvas.
enum CanvasTool {
  pen('pen', KansoIcons.penLine),
  pencil('pencil', KansoIcons.pencil),
  marker('marker', KansoIcons.penTool),
  highlighter('highlighter', KansoIcons.highlighter),

  /// Removes whole strokes.
  eraser('', KansoIcons.eraser),

  /// Rubs out only the part it touches and leaves the rest of the stroke
  /// as new strokes.
  partialEraser('', KansoIcons.eraser),
  lasso('', KansoIcons.lasso),

  /// Taps place a text card.
  text('', KansoIcons.typeface),

  /// Tap one card then another to draw an arrow between them.
  arrow('', KansoIcons.arrowRight),

  /// Drags pan the view.
  hand('', KansoIcons.hand);

  const CanvasTool(this.ink, this.icon);

  /// The pen name the library draws with, for the four pens; empty for
  /// the rest.
  final String ink;
  final KansoIcons icon;

  /// Whether the tool draws a stroke.
  bool get draws => ink.isNotEmpty;

  /// Whether the tool rubs strokes out.
  bool get erases => this == eraser || this == partialEraser;
}
