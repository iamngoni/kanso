// Kanso — preview_figure.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_icons.dart';
import '../../../../core/utils/context_extensions.dart';
import '../../../../core/views/widgets/kanso_icon.dart';
import 'preview_style.dart';

/// Math, a diagram or a drawing set on its own: its SVG at its size,
/// scaled with the system's text, made smaller to fit the note's width.
/// Display math is centred.
class PreviewFigure extends StatelessWidget {
  const PreviewFigure({required this.kind, required this.svg, super.key});

  final PreviewFigureKind kind;
  final PreviewSvg svg;

  @override
  Widget build(BuildContext context) {
    // Drawn at the note's text size, so scaled with the system's text as the
    // text around it is.
    final size = PreviewScope.of(context).style.size;
    final scale = size > 0
        ? MediaQuery.textScalerOf(context).scale(size) / size
        : 1.0;
    return Align(
      alignment: kind == PreviewFigureKind.math
          ? Alignment.center
          : Alignment.centerLeft,
      child: FittedBox(
        fit: BoxFit.scaleDown,
        child: SizedBox(
          width: svg.width * scale,
          height: svg.height * scale,
          child: SvgPicture.string(
            svg.markup,
            width: svg.width * scale,
            height: svg.height * scale,
          ),
        ),
      ),
    );
  }
}

/// A drawing with nothing in it yet: a quiet placeholder that says so.
class PreviewEmptyDrawing extends StatelessWidget {
  const PreviewEmptyDrawing({super.key});

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 380),
        height: 120,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: style.surface,
          border: Border.all(color: style.border),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Text(
          context.l10n.previewEmptyDrawing,
          style: style.sized(0.8667).copyWith(color: style.muted),
        ),
      ),
    );
  }
}

/// A canvas a note embeds: its picture at its size, made smaller to fit the
/// note's width, with the notice the embed carries under it; tapping it
/// opens the canvas. A locked, missing or undrawable canvas is a quiet
/// placeholder, and a locked one carries nothing of the canvas.
class PreviewCanvas extends StatelessWidget {
  const PreviewCanvas({required this.note, required this.shown, super.key});

  final String note;
  final PreviewCanvasShown shown;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final l10n = context.l10n;
    final body = switch (shown) {
      PreviewCanvasShown_Drawing(:final svg, :final notices) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          PreviewFigure(kind: PreviewFigureKind.drawing, svg: svg),
          for (final notice in notices)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                switch (notice) {
                  PreviewCanvasNotice.wholeCanvas =>
                    l10n.previewCanvasWholeCanvas,
                  PreviewCanvasNotice.groupMissing =>
                    l10n.previewCanvasGroupMissing,
                },
                style: style.sized(0.8667).copyWith(color: style.muted),
              ),
            ),
        ],
      ),
      PreviewCanvasShown_Locked() => _placeholder(
        context,
        l10n.previewCanvasLocked,
        lock: true,
      ),
      PreviewCanvasShown_Missing() => _placeholder(
        context,
        l10n.previewCanvasMissing,
      ),
      PreviewCanvasShown_NotCanvas() => _placeholder(
        context,
        l10n.previewCanvasNotCanvas,
      ),
      PreviewCanvasShown_Invalid() => _placeholder(
        context,
        l10n.previewCanvasInvalid,
      ),
    };
    return Semantics(
      button: true,
      label: l10n.previewCanvasOpen,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        // A locked canvas opens like any locked note: it asks for the lock.
        onTap: () => scope.onLink('inkdrop://note/$note'),
        child: body,
      ),
    );
  }

  Widget _placeholder(BuildContext context, String words, {bool lock = false}) {
    final style = PreviewScope.of(context).style;
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        constraints: const BoxConstraints(maxWidth: 380),
        height: 96,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: style.surface,
          border: Border.all(color: style.border),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            if (lock) ...<Widget>[
              KansoIcon(KansoIcons.lock, size: style.size, color: style.muted),
              const SizedBox(width: 6),
            ],
            Text(
              words,
              style: style.sized(0.8667).copyWith(color: style.muted),
            ),
          ],
        ),
      ),
    );
  }
}
