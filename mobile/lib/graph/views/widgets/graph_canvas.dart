// Kanso — graph_canvas.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';

/// A pannable, zoomable graph whose links are drawn behind its note nodes.
class GraphCanvas extends StatelessWidget {
  const GraphCanvas({
    required this.layout,
    required this.onNodeSelected,
    super.key,
  });

  final GraphLayout layout;
  final ValueChanged<GraphNodeRow> onNodeSelected;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final dark = Theme.of(context).brightness == Brightness.dark;
    final painter = _GraphPainter(
      layout: layout,
      textColor: colors.text,
      textStyle: context.typography.secondary,
      edgeColor: colors.divider,
      dark: dark,
    );
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        return InteractiveViewer(
          minScale: 0.25,
          maxScale: 4,
          constrained: false,
          boundaryMargin: const EdgeInsets.all(300),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapUp: (details) {
              final index = painter.nodeAt(details.localPosition, size);
              if (index != null) onNodeSelected(layout.nodes[index]);
            },
            child: CustomPaint(size: size, painter: painter),
          ),
        );
      },
    );
  }
}

class _GraphPainter extends CustomPainter {
  const _GraphPainter({
    required this.layout,
    required this.textColor,
    required this.textStyle,
    required this.edgeColor,
    required this.dark,
  });

  final GraphLayout layout;
  final Color textColor;
  final TextStyle textStyle;
  final Color edgeColor;
  final bool dark;

  List<Offset> positions(Size size) {
    if (layout.nodes.isEmpty) return const <Offset>[];
    final minX = layout.nodes.map((node) => node.x).reduce(math.min);
    final maxX = layout.nodes.map((node) => node.x).reduce(math.max);
    final minY = layout.nodes.map((node) => node.y).reduce(math.min);
    final maxY = layout.nodes.map((node) => node.y).reduce(math.max);
    final spanX = math.max(1, maxX - minX).toDouble();
    final spanY = math.max(1, maxY - minY).toDouble();
    const padding = 90.0;
    final scale = math.min(
      (size.width - padding * 2) / spanX,
      (size.height - padding * 2) / spanY,
    );
    final graphWidth = spanX * scale;
    final graphHeight = spanY * scale;
    final left = (size.width - graphWidth) / 2;
    final top = (size.height - graphHeight) / 2;
    return <Offset>[
      for (final node in layout.nodes)
        Offset(left + (node.x - minX) * scale, top + (node.y - minY) * scale),
    ];
  }

  int? nodeAt(Offset point, Size size) {
    final points = positions(size);
    for (var i = points.length - 1; i >= 0; i--) {
      if ((point - points[i]).distance <=
          math.max(24, layout.nodes[i].radius + 10)) {
        return i;
      }
    }
    return null;
  }

  @override
  void paint(Canvas canvas, Size size) {
    final points = positions(size);
    final edgePaint = Paint()
      ..color = edgeColor.withValues(alpha: 0.65)
      ..strokeWidth = 1.2;
    for (final edge in layout.edges) {
      if (edge.from >= 0 &&
          edge.from < points.length &&
          edge.to >= 0 &&
          edge.to < points.length) {
        canvas.drawLine(points[edge.from], points[edge.to], edgePaint);
      }
    }
    for (var i = 0; i < layout.nodes.length; i++) {
      final node = layout.nodes[i];
      final radius = math.max(8, node.radius).toDouble();
      final center = points[i];
      canvas.drawCircle(
        center,
        radius,
        Paint()..color = Color(dark ? node.colorDark : node.colorLight),
      );
      if (node.isCenter) {
        canvas.drawCircle(
          center,
          radius + 4,
          Paint()
            ..color = textColor
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2,
        );
      }
      final painter = TextPainter(
        text: TextSpan(
          text: node.title,
          style: textStyle.copyWith(color: textColor),
        ),
        textDirection: TextDirection.ltr,
        maxLines: 1,
        ellipsis: '…',
      )..layout(maxWidth: 180);
      painter.paint(
        canvas,
        Offset(center.dx - painter.width / 2, center.dy + radius + 5),
      );
    }
  }

  @override
  bool shouldRepaint(_GraphPainter oldDelegate) =>
      oldDelegate.layout != layout ||
      oldDelegate.textColor != textColor ||
      oldDelegate.textStyle != textStyle ||
      oldDelegate.edgeColor != edgeColor ||
      oldDelegate.dark != dark;
}
