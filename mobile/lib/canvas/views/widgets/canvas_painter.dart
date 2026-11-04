// Kanso — canvas_painter.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/rendering.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../utils/canvas_viewport.dart';
import '../../utils/ink_colors.dart';
import '../../utils/stroke_path.dart';

/// The colours a canvas is painted in: the screen's theme.
@immutable
class CanvasTheme {
  const CanvasTheme({
    required this.dark,
    required this.text,
    required this.muted,
    required this.surface,
    required this.border,
    required this.accent,
  });

  final bool dark;
  final Color text;
  final Color muted;
  final Color surface;
  final Color border;
  final Color accent;

  @override
  bool operator ==(Object other) =>
      other is CanvasTheme &&
      other.dark == dark &&
      other.text == text &&
      other.muted == muted &&
      other.surface == surface &&
      other.border == border &&
      other.accent == accent;

  @override
  int get hashCode => Object.hash(dark, text, muted, surface, border, accent);
}

/// The paths of strokes already built, by stroke ID: a stroke's outline
/// never changes, so its path is built once.
class StrokePaths {
  final Map<String, Path> _paths = <String, Path>{};

  /// The most paths kept before the cache starts over.
  static const int limit = 6000;

  Path of(CanvasStroke stroke) {
    final held = _paths[stroke.id];
    if (held != null) {
      return held;
    }
    if (_paths.length >= limit) {
      _paths.clear();
    }
    return _paths[stroke.id] = outlinePath(stroke.outline);
  }
}

/// A stroke being drawn: its outline, as the library makes it, and its pen.
@immutable
class LiveInk {
  const LiveInk({
    required this.outline,
    required this.color,
    required this.opacity,
  });

  final List<double> outline;
  final String color;
  final double opacity;
}

/// Paints a canvas: groups, arrows, cards and ink, culled to what the view
/// shows, with the stroke in progress, the lasso, the eraser's ring and the
/// selection on top.
class CanvasPainter extends CustomPainter {
  CanvasPainter({
    required this.view,
    required this.viewport,
    required this.theme,
    required this.paths,
    this.live,
    this.settling = const <LiveInk>[],
    this.lasso = const <Offset>[],
    this.eraserAt,
    this.eraserRadius = 0,
    this.selected = const <String>{},
    this.selectionShift = Offset.zero,
    this.selectionBounds,
    this.cardShift,
    this.pendingArrowFrom,
    super.repaint,
  });

  final CanvasView view;
  final CanvasViewport viewport;
  final CanvasTheme theme;
  final StrokePaths paths;
  final LiveInk? live;

  /// Strokes just lifted that the library hasn't taken yet.
  final List<LiveInk> settling;
  final List<Offset> lasso;
  final Offset? eraserAt;
  final double eraserRadius;
  final Set<String> selected;
  final Offset selectionShift;
  final Rect? selectionBounds;

  /// A card being dragged, and by how much.
  final (String, Offset)? cardShift;
  final String? pendingArrowFrom;

  @override
  void paint(Canvas canvas, Size size) {
    canvas
      ..save()
      ..clipRect(Offset.zero & size)
      ..translate(size.width / 2, size.height / 2)
      ..scale(viewport.zoom)
      ..translate(-viewport.center.dx, -viewport.center.dy);
    final seen = viewport.visible(size).inflate(8 / viewport.zoom);
    _groups(canvas, seen);
    _arrows(canvas);
    _cards(canvas, seen);
    _ink(canvas, seen);
    for (final ink in settling) {
      _fill(canvas, outlinePath(ink.outline), ink.color, ink.opacity);
    }
    final lively = live;
    if (lively != null) {
      _fill(canvas, outlinePath(lively.outline), lively.color, lively.opacity);
    }
    _selection(canvas);
    _lasso(canvas);
    canvas.restore();
    _eraser(canvas);
  }

  Rect _rect(CanvasRect r) => Rect.fromLTWH(r.x, r.y, r.width, r.height);

  void _groups(Canvas canvas, Rect seen) {
    for (final group in view.groups) {
      final rect = _rect(group.rect);
      if (!rect.overlaps(seen)) {
        continue;
      }
      final color = cardColor(group.color) ?? theme.muted;
      final frame = RRect.fromRectAndRadius(rect, const Radius.circular(8));
      canvas
        ..drawRRect(frame, Paint()..color = color.withValues(alpha: 0.07))
        ..drawRRect(
          frame,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.5
            ..color = color.withValues(alpha: 0.7),
        );
      if (group.label.isNotEmpty) {
        _text(
          canvas,
          group.label,
          rect.topLeft + const Offset(10, -20),
          maxWidth: rect.width,
          size: 13,
          color: theme.muted,
          maxLines: 1,
        );
      }
    }
  }

  void _arrows(Canvas canvas) {
    for (final arrow in view.arrows) {
      final color = cardColor(arrow.color) ?? theme.muted;
      Offset p(CanvasPoint v) => Offset(v.x, v.y);
      final path = Path()
        ..moveTo(arrow.from.x, arrow.from.y)
        ..cubicTo(
          arrow.c1.x,
          arrow.c1.y,
          arrow.c2.x,
          arrow.c2.y,
          arrow.to.x,
          arrow.to.y,
        );
      canvas.drawPath(
        path,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..strokeCap = StrokeCap.round
          ..color = color,
      );
      if (arrow.toArrow) {
        _head(canvas, p(arrow.to), p(arrow.c2), color);
      }
      if (arrow.fromArrow) {
        _head(canvas, p(arrow.from), p(arrow.c1), color);
      }
      if (arrow.label.isNotEmpty) {
        final mid = Offset(
          (arrow.from.x + arrow.to.x) / 2,
          (arrow.from.y + arrow.to.y) / 2,
        );
        _text(
          canvas,
          arrow.label,
          mid + const Offset(-40, -20),
          maxWidth: 80,
          size: 13,
          color: theme.muted,
          maxLines: 1,
          align: TextAlign.center,
        );
      }
    }
  }

  /// A head at [tip], pointing away from [control].
  void _head(Canvas canvas, Offset tip, Offset control, Color color) {
    final away = control - tip;
    final length = away.distance;
    if (length == 0) {
      return;
    }
    final normal = away / length;
    final side = Offset(-normal.dy, normal.dx);
    final base = tip + normal * 12;
    canvas.drawPath(
      Path()
        ..moveTo(tip.dx, tip.dy)
        ..lineTo(base.dx + side.dx * 6, base.dy + side.dy * 6)
        ..lineTo(base.dx - side.dx * 6, base.dy - side.dy * 6)
        ..close(),
      Paint()..color = color,
    );
  }

  void _cards(Canvas canvas, Rect seen) {
    for (final card in view.cards) {
      var rect = _rect(card.rect);
      final dragged = cardShift;
      if (dragged != null && dragged.$1 == card.id) {
        rect = rect.shift(dragged.$2);
      }
      if (!rect.overlaps(seen)) {
        continue;
      }
      final tint = cardColor(card.color);
      final border = tint ?? theme.border;
      final fill = tint == null
          ? theme.surface
          : Color.alphaBlend(tint.withValues(alpha: 0.14), theme.surface);
      final frame = RRect.fromRectAndRadius(rect, const Radius.circular(8));
      canvas
        ..drawRRect(frame, Paint()..color = fill)
        ..drawRRect(
          frame,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = card.id == pendingArrowFrom ? 3 : 1.5
            ..color = card.id == pendingArrowFrom ? theme.accent : border,
        );
      final lines = math.max(
        1,
        ((rect.height - 16) / (_fontSize * 1.35)).floor(),
      );
      _text(
        canvas,
        card.text,
        rect.topLeft + const Offset(12, 10),
        maxWidth: rect.width - 24,
        size: _fontSize,
        color: card.missing ? theme.muted : theme.text,
        maxLines: lines,
        weight: card.kind == 'text' ? FontWeight.w400 : FontWeight.w600,
        italic: card.missing,
      );
    }
  }

  static const double _fontSize = 16;

  void _text(
    Canvas canvas,
    String text,
    Offset at, {
    required double maxWidth,
    required double size,
    required Color color,
    required int maxLines,
    FontWeight weight = FontWeight.w400,
    bool italic = false,
    TextAlign align = TextAlign.start,
  }) {
    if (text.isEmpty || maxWidth <= 0) {
      return;
    }
    TextPainter(
        text: TextSpan(
          text: text,
          style: TextStyle(
            fontSize: size,
            color: color,
            fontWeight: weight,
            fontStyle: italic ? FontStyle.italic : FontStyle.normal,
            height: 1.3,
          ),
        ),
        textDirection: TextDirection.ltr,
        textAlign: align,
        maxLines: maxLines,
        ellipsis: '…',
      )
      ..layout(maxWidth: maxWidth)
      ..paint(canvas, at)
      ..dispose();
  }

  void _ink(Canvas canvas, Rect seen) {
    for (final stroke in view.strokes) {
      if (!_rect(stroke.bounds).overlaps(seen) || stroke.outline.isEmpty) {
        continue;
      }
      final color = inkColor(
        stroke.color,
        dark: theme.dark,
        text: theme.text,
      );
      final moving = selected.contains(stroke.id);
      if (moving && selectionShift != Offset.zero) {
        canvas
          ..save()
          ..translate(selectionShift.dx, selectionShift.dy);
      }
      canvas.drawPath(
        paths.of(stroke),
        Paint()..color = color.withValues(alpha: stroke.opacity),
      );
      if (moving && selectionShift != Offset.zero) {
        canvas.restore();
      }
    }
  }

  void _fill(Canvas canvas, Path path, String color, double opacity) {
    canvas.drawPath(
      path,
      Paint()
        ..color = inkColor(
          color,
          dark: theme.dark,
          text: theme.text,
        ).withValues(alpha: opacity),
    );
  }

  void _selection(Canvas canvas) {
    final bounds = selectionBounds;
    if (bounds == null || selected.isEmpty) {
      return;
    }
    final rect = bounds.shift(selectionShift).inflate(4 / viewport.zoom);
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5 / viewport.zoom
      ..color = theme.accent;
    canvas
      ..drawRect(rect, Paint()..color = theme.accent.withValues(alpha: 0.08))
      ..drawPath(_dashed(rect), paint);
  }

  void _lasso(Canvas canvas) {
    if (lasso.length < 2) {
      return;
    }
    final path = Path()..moveTo(lasso.first.dx, lasso.first.dy);
    for (final p in lasso.skip(1)) {
      path.lineTo(p.dx, p.dy);
    }
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5 / viewport.zoom
        ..color = theme.accent,
    );
  }

  /// A rectangle as dashes.
  Path _dashed(Rect rect) {
    final dash = 6 / viewport.zoom;
    final path = Path();
    void edge(Offset a, Offset b) {
      final length = (b - a).distance;
      final step = (b - a) / length;
      for (var at = 0.0; at < length; at += dash * 2) {
        final end = math.min(at + dash, length);
        path
          ..moveTo(a.dx + step.dx * at, a.dy + step.dy * at)
          ..lineTo(a.dx + step.dx * end, a.dy + step.dy * end);
      }
    }

    edge(rect.topLeft, rect.topRight);
    edge(rect.topRight, rect.bottomRight);
    edge(rect.bottomRight, rect.bottomLeft);
    edge(rect.bottomLeft, rect.topLeft);
    return path;
  }

  void _eraser(Canvas canvas) {
    final at = eraserAt;
    if (at == null) {
      return;
    }
    canvas.drawCircle(
      at,
      eraserRadius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = theme.muted,
    );
  }

  @override
  bool shouldRepaint(CanvasPainter old) =>
      old.view != view ||
      old.viewport != viewport ||
      old.theme != theme ||
      old.live != live ||
      old.settling != settling ||
      old.lasso != lasso ||
      old.eraserAt != eraserAt ||
      old.selected != selected ||
      old.selectionShift != selectionShift ||
      old.cardShift != cardShift ||
      old.pendingArrowFrom != pendingArrowFrom;
}
