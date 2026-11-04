// Kanso — pointer_role.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

import '../../core/bridge/api/canvas_types.dart';
import '../models/data/canvas_tool.dart';

/// What one pointer does on the canvas.
enum PointerRole {
  /// Draws a stroke.
  ink,

  /// Removes whole strokes.
  erase,

  /// Rubs out part of strokes.
  partialErase,

  /// Draws a lasso, or drags what it holds, or a card.
  lasso,

  /// A tap places or picks something (a text card, an arrow's ends).
  place,

  /// Pans the view; two such pointers also zoom it.
  pan,
}

/// The role of a pointer of [kind] under [tool].
///
/// A pen draws with the chosen tool, and its flipped end is the partial
/// eraser whatever the tool is. A finger or a mouse does what the tool says
/// too, unless a pen has been seen on this canvas (then fingers only pan
/// and zoom, as palms do) or the person turned "Finger draws" off. The hand
/// tool pans with anything.
PointerRole roleFor({
  required PointerDeviceKind kind,
  required CanvasTool tool,
  required bool stylusSeen,
  required bool fingerDraws,
}) {
  if (kind == PointerDeviceKind.invertedStylus) {
    return PointerRole.partialErase;
  }
  if (tool == CanvasTool.hand) {
    return PointerRole.pan;
  }
  if (kind == PointerDeviceKind.touch && (stylusSeen || !fingerDraws)) {
    return PointerRole.pan;
  }
  if (tool.draws) {
    return PointerRole.ink;
  }
  return switch (tool) {
    CanvasTool.eraser => PointerRole.erase,
    CanvasTool.partialEraser => PointerRole.partialErase,
    CanvasTool.lasso => PointerRole.lasso,
    _ => PointerRole.place,
  };
}

/// A pen sample from a pointer event's numbers. A pen's pressure is spread
/// over the range the device reports; a finger, a mouse and a pen with no
/// pressure draw at 0.5, which the library widens and narrows by speed. Tilt
/// comes as the library wants it: azimuth, and altitude (the angle above the
/// surface, a right angle when the pen stands upright).
PenSample penSample({
  required Offset canvasPoint,
  required PointerDeviceKind kind,
  required double pressure,
  required double pressureMin,
  required double pressureMax,
  required double tilt,
  required double orientation,
}) {
  final isPen =
      kind == PointerDeviceKind.stylus ||
      kind == PointerDeviceKind.invertedStylus;
  final range = pressureMax - pressureMin;
  final real = isPen && range > 0 && pressure > 0;
  final normal = real
      ? ((pressure - pressureMin) / range).clamp(0.0, 1.0)
      : 0.5;
  final tilted = isPen && tilt > 0;
  return PenSample(
    x: canvasPoint.dx,
    y: canvasPoint.dy,
    pressure: normal,
    azimuth: tilted ? orientation : null,
    altitude: tilted ? math.pi / 2 - tilt.clamp(0.0, math.pi / 2) : null,
  );
}
