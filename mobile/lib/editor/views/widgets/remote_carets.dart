// Kanso — remote_carets.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../controllers/kanso_markdown_controller.dart';

/// Other people's carets in a shared note, each a thin bar in their colour
/// with their name on a tag above the line (below it on the first line), as
/// the desktop draws them. It lies over the editor and takes no touches;
/// carets outside the field's window are not drawn.
class RemoteCarets extends StatefulWidget {
  const RemoteCarets({
    required this.controller,
    required this.editableKey,
    super.key,
  });

  final KansoMarkdownController controller;

  /// The editor's editable text, to find where a caret is drawn.
  final GlobalKey<EditableTextState> editableKey;

  /// How tall a name tag is, and its text, as the desktop draws them.
  static const double tagHeight = 15;
  static const double tagText = 10.5;

  @override
  State<RemoteCarets> createState() => _RemoteCaretsState();
}

class _RemoteCaretsState extends State<RemoteCarets> {
  final GlobalKey paintKey = GlobalKey();

  @override
  Widget build(BuildContext context) {
    return IgnorePointer(
      child: CustomPaint(
        key: paintKey,
        painter: _CaretsPainter(
          controller: widget.controller,
          editableKey: widget.editableKey,
          paintKey: paintKey,
          label: context.colors.editor,
          style: context.typography.meta.copyWith(
            fontSize: RemoteCarets.tagText,
            fontWeight: FontWeight.w600,
            height: 1.2,
          ),
          direction: Directionality.of(context),
        ),
        size: Size.infinite,
      ),
    );
  }
}

class _CaretsPainter extends CustomPainter {
  _CaretsPainter({
    required this.controller,
    required this.editableKey,
    required this.paintKey,
    required this.label,
    required this.style,
    required this.direction,
  }) : super(repaint: controller);

  final KansoMarkdownController controller;
  final GlobalKey<EditableTextState> editableKey;
  final GlobalKey paintKey;
  final Color label;
  final TextStyle style;
  final TextDirection direction;

  @override
  void paint(Canvas canvas, Size size) {
    final cursors = controller.remoteCursors;
    final render = editableKey.currentState?.renderEditable;
    final box = paintKey.currentContext?.findRenderObject();
    if (cursors.isEmpty ||
        render == null ||
        !render.attached ||
        !render.hasSize ||
        box is! RenderBox) {
      return;
    }
    // Where the editable text is on this layer.
    final origin = render.localToGlobal(Offset.zero, ancestor: box);
    canvas
      ..save()
      ..clipRect(Offset.zero & size);
    for (final cursor in cursors) {
      final at = controller.fieldOffsetOf(cursor.caret);
      if (at == null) {
        continue;
      }
      final rect = render
          .getLocalRectForCaret(TextPosition(offset: at))
          .shift(origin);
      final color = controller.cursorColor(cursor);
      canvas.drawRect(
        Rect.fromLTWH(rect.left - 1, rect.top, 2, rect.height),
        Paint()..color = color,
      );
      final name = TextPainter(
        text: TextSpan(
          text: cursor.name,
          style: style.copyWith(color: label),
        ),
        textDirection: direction,
        maxLines: 1,
      )..layout();
      const height = RemoteCarets.tagHeight;
      // Above the line, or below it on the first line shown.
      var top = rect.top - height;
      if (top < 0) {
        top = rect.bottom;
      }
      final tag = RRect.fromRectAndRadius(
        Rect.fromLTWH(rect.left - 1, top, name.width + 8, height),
        const Radius.circular(3),
      );
      canvas.drawRRect(tag, Paint()..color = color);
      name
        ..paint(canvas, Offset(rect.left + 3, top + (height - name.height) / 2))
        ..dispose();
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(_CaretsPainter old) =>
      old.controller != controller ||
      old.label != label ||
      old.style != style ||
      old.direction != direction;
}
