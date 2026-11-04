// Kanso — canvas_viewport.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter/foundation.dart';

/// How the endless canvas maps onto the screen: the canvas point in the
/// middle of the view and how many screen points one canvas point takes.
/// The desktop's canvas pans and zooms through the same transform.
@immutable
class CanvasViewport {
  const CanvasViewport({this.center = Offset.zero, this.zoom = 1});

  /// The view that shows [bounds] whole in a view [size] big with [margin]
  /// around it, no closer in than [maxFit]. An empty canvas shows its
  /// origin at the middle.
  factory CanvasViewport.fit(
    Rect? bounds,
    Size size, {
    double margin = 32,
    double maxFit = 1,
  }) {
    if (bounds == null || size.isEmpty) {
      return const CanvasViewport();
    }
    final roomX = math.max(size.width - 2 * margin, 1);
    final roomY = math.max(size.height - 2 * margin, 1);
    final zoom = math
        .min(
          roomX / math.max(bounds.width, 1),
          roomY / math.max(bounds.height, 1),
        )
        .clamp(minZoom, math.min(maxFit, maxZoom))
        .toDouble();
    return CanvasViewport(center: bounds.center, zoom: zoom);
  }

  /// The farthest the view zooms out.
  static const double minZoom = 0.02;

  /// The closest the view zooms in.
  static const double maxZoom = 8;

  final Offset center;
  final double zoom;

  /// Where canvas point [p] is in a view [size] big, from its top left.
  Offset toScreen(Offset p, Size size) => Offset(
    (p.dx - center.dx) * zoom + size.width / 2,
    (p.dy - center.dy) * zoom + size.height / 2,
  );

  /// The canvas point at screen point [s] in a view [size] big.
  Offset toCanvas(Offset s, Size size) => Offset(
    (s.dx - size.width / 2) / zoom + center.dx,
    (s.dy - size.height / 2) / zoom + center.dy,
  );

  /// The canvas rectangle a view [size] big shows.
  Rect visible(Size size) => Rect.fromPoints(
    toCanvas(Offset.zero, size),
    toCanvas(Offset(size.width, size.height), size),
  );

  /// The view moved by [delta] screen points (a drag of the content).
  CanvasViewport panned(Offset delta) =>
      CanvasViewport(center: center - delta / zoom, zoom: zoom);

  /// Zoomed by [factor] (kept within the limits), keeping the canvas point
  /// under screen point [at] where it is.
  CanvasViewport zoomedAt(double factor, Offset at, Size size) {
    final anchor = toCanvas(at, size);
    final next = CanvasViewport(
      center: center,
      zoom: (zoom * factor).clamp(minZoom, maxZoom),
    );
    final moved = next.toCanvas(at, size);
    return CanvasViewport(center: center + (anchor - moved), zoom: next.zoom);
  }

  /// One step of a two-finger gesture: the content under the fingers' old
  /// focal point follows the new one, scaled by [scale].
  CanvasViewport gestured({
    required Offset oldFocal,
    required Offset newFocal,
    required double scale,
    required Size size,
  }) {
    final anchor = toCanvas(oldFocal, size);
    final next = CanvasViewport(
      center: center,
      zoom: (zoom * scale).clamp(minZoom, maxZoom),
    );
    final under = next.toCanvas(newFocal, size);
    return CanvasViewport(center: center + (anchor - under), zoom: next.zoom);
  }

  @override
  bool operator ==(Object other) =>
      other is CanvasViewport && other.center == center && other.zoom == zoom;

  @override
  int get hashCode => Object.hash(center, zoom);
}
