// Kanso — home_map_card.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../core/bridge/api/notebook_home_types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The map of a notebook's notes as a 16:10 card: a dot for each note, a line
/// for each link, and a few short stubs where a note links beyond the map.
/// A tap opens the graph narrowed to the notebook. While the map is being
/// worked out the card is an empty tile of the same size, so nothing moves
/// when it arrives.
class HomeMapCard extends StatelessWidget {
  const HomeMapCard({
    required this.map,
    required this.semanticLabel,
    required this.onTap,
    super.key,
  });

  /// The map, or null while it is being computed.
  final HomeMap? map;
  final String semanticLabel;
  final VoidCallback onTap;

  /// The card's shape.
  static const double aspect = 16 / 10;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final read = map;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: semanticLabel,
      child: AspectRatio(
        aspectRatio: aspect,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: colors.fill,
            borderRadius: KansoRadii.mediumBorder,
            border: Border.all(color: colors.divider),
          ),
          child: ClipRRect(
            borderRadius: KansoRadii.mediumBorder,
            child: read == null
                ? const SizedBox.expand()
                : CustomPaint(
                    key: const ValueKey<String>('home-map-canvas'),
                    painter: HomeMapPainter(
                      map: read,
                      line: colors.textMuted.withValues(alpha: 0.45),
                      dot: colors.accent,
                      stub: colors.textMuted.withValues(alpha: 0.35),
                    ),
                    size: Size.infinite,
                  ),
          ),
        ),
      ),
    );
  }
}

/// Draws a [HomeMap] scaled to fit, with a margin.
class HomeMapPainter extends CustomPainter {
  HomeMapPainter({
    required this.map,
    required this.line,
    required this.dot,
    required this.stub,
  });

  final HomeMap map;
  final Color line;
  final Color dot;
  final Color stub;

  /// The room kept free around the drawing.
  static const double margin = 14;

  /// The most stubs drawn from one dot.
  static const int maxStubs = 6;

  @override
  void paint(Canvas canvas, Size size) {
    if (map.nodes.isEmpty || map.width <= 0 || map.height <= 0) {
      return;
    }
    final scale = math.min(
      (size.width - 2 * margin) / map.width,
      (size.height - 2 * margin) / map.height,
    );
    final dx = (size.width - map.width * scale) / 2;
    final dy = (size.height - map.height * scale) / 2;
    Offset at(HomeMapNode n) => Offset(dx + n.x * scale, dy + n.y * scale);
    final edgePaint = Paint()
      ..color = line
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;
    for (final e in map.edges) {
      if (e.a < 0 ||
          e.b < 0 ||
          e.a >= map.nodes.length ||
          e.b >= map.nodes.length) {
        continue;
      }
      canvas.drawLine(at(map.nodes[e.a]), at(map.nodes[e.b]), edgePaint);
    }
    final stubPaint = Paint()
      ..color = stub
      ..strokeWidth = 1
      ..strokeCap = StrokeCap.round;
    final dotPaint = Paint()..color = dot;
    for (final n in map.nodes) {
      final centre = at(n);
      final radius = math.max(2.5, n.radius * scale);
      final stubs = math.min(n.stubs, maxStubs);
      for (var i = 0; i < stubs; i++) {
        final angle = 2 * math.pi * i / stubs + math.pi / 6;
        final from = centre + Offset.fromDirection(angle, radius);
        final to = centre + Offset.fromDirection(angle, radius + 7);
        canvas.drawLine(from, to, stubPaint);
      }
      canvas.drawCircle(centre, radius, dotPaint);
    }
  }

  @override
  bool shouldRepaint(HomeMapPainter old) =>
      old.map != map ||
      old.line != line ||
      old.dot != dot ||
      old.stub != stub;
}
