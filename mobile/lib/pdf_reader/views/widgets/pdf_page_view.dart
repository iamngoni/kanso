// Kanso — pdf_page_view.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';

import '../../../core/bridge/api/pdf_types.dart';
import '../../../core/configs/kanso_colors.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../note/views/widgets/pdf_highlights_sheet.dart'
    show highlightColor;
import '../../models/repos/abs/pdf_reader_repository.dart';
import 'pdf_page_overlay.dart';

/// One page of the open PDF: its picture, the note's highlights on it, the
/// selection and the search's hit, with the two handles that change the
/// selection.
///
/// The picture is drawn by the Rust reader at the screen's density; a page
/// is redrawn when the width it is shown at or the dark rendering changes.
/// The page keeps the last picture while a new one is drawn, so a resize
/// does not flash.
class PdfPageView extends StatefulWidget {
  const PdfPageView({
    required this.session,
    required this.page,
    required this.size,
    required this.width,
    required this.invert,
    required this.marks,
    required this.selection,
    required this.search,
    required this.focus,
    required this.onWordSelected,
    required this.onLineSelected,
    required this.onSelectionExtended,
    required this.onSelectionCleared,
    this.overlays = const <PdfPageOverlay>[],
    super.key,
  });

  /// The open document.
  final PdfDocumentSession session;

  /// The page, 1-based.
  final int page;

  /// The page's own size, in points.
  final PdfPageSize size;

  /// The page's width on screen, in logical pixels.
  final double width;

  /// Whether the page is drawn for a dark screen.
  final bool invert;

  /// The note's highlights on this page.
  final List<PdfMark> marks;

  /// The selection, when it reaches this page.
  final PdfSelection? selection;

  /// The search hit on this page, when the shown hit is here.
  final PdfPageMarks? search;

  /// The highlight the link named, drawn stronger while it is in view.
  final String? focus;

  /// A word was held at a point of the page: fractions from its top left.
  final void Function(int page, double x, double y) onWordSelected;

  /// A line was tapped twice at a point of the page.
  final void Function(int page, double x, double y) onLineSelected;

  /// A selection handle was dragged to a point of the page; `fromStart` is
  /// the end that stays fixed.
  final void Function(int page, double x, double y, {required bool fromStart})
  onSelectionExtended;

  /// The page, outside a word, was tapped: the selection goes.
  final VoidCallback onSelectionCleared;

  /// Layers drawn over the page's picture and highlights.
  final List<PdfPageOverlay> overlays;

  /// The page's height on screen for [width]: its own proportions.
  static double heightFor(PdfPageSize size, double width) {
    if (size.width <= 0) {
      return width * 1.414;
    }
    return width * size.height / size.width;
  }

  @override
  State<PdfPageView> createState() => _PdfPageViewState();
}

class _PdfPageViewState extends State<PdfPageView> {
  ui.Image? _image;
  bool _failed = false;
  int _request = 0;

  double get _height => PdfPageView.heightFor(widget.size, widget.width);

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _load();
  }

  @override
  void didUpdateWidget(PdfPageView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.width != widget.width ||
        oldWidget.invert != widget.invert ||
        oldWidget.page != widget.page ||
        !identical(oldWidget.session, widget.session)) {
      _load();
    }
  }

  @override
  void dispose() {
    _request++;
    _image?.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final request = ++_request;
    final density = MediaQuery.devicePixelRatioOf(context).clamp(1.0, 3.0);
    final widthPx = (widget.width * density).round().clamp(200, 4096);
    // Called while the widget is being built; the picture's placeholder is
    // drawn by the build that follows.
    _failed = false;
    final result = await widget.session.render(
      widget.page - 1,
      widthPx,
      invert: widget.invert,
    );
    if (!mounted || request != _request) {
      return;
    }
    final image = await result.fold<Future<ui.Image?>>(
      (error) async => null,
      _decode,
    );
    if (!mounted || request != _request) {
      image?.dispose();
      return;
    }
    final old = _image;
    setState(() {
      _image = image;
      _failed = image == null;
    });
    old?.dispose();
  }

  /// The reader's BGRA pixels as a picture. They are not copied: the
  /// decoder reads the page's buffer, which the bridge handed over.
  static Future<ui.Image?> _decode(PdfPageImage page) async {
    if (page.width <= 0 || page.height <= 0) {
      return null;
    }
    final completer = Completer<ui.Image?>();
    ui.decodeImageFromPixels(
      page.bgra,
      page.width,
      page.height,
      ui.PixelFormat.bgra8888,
      completer.complete,
    );
    return completer.future;
  }

  /// The selection's rectangles on this page, top to bottom, left to right.
  List<PdfRect> get _selectionRects {
    final selection = widget.selection;
    if (selection == null) {
      return const <PdfRect>[];
    }
    final rects =
        <PdfRect>[
          for (final page in selection.marks)
            if (page.page == widget.page) ...page.rects,
        ]..sort((a, b) {
          final byTop = a.y0.compareTo(b.y0);
          return byTop != 0 ? byTop : a.x0.compareTo(b.x0);
        });
    return rects;
  }

  void _point(Offset position, void Function(double x, double y) at) {
    final width = widget.width;
    final height = _height;
    if (width <= 0 || height <= 0) {
      return;
    }
    at(
      (position.dx / width).clamp(0.0, 1.0),
      (position.dy / height).clamp(0.0, 1.0),
    );
  }

  @override
  Widget build(BuildContext context) {
    final width = widget.width;
    final height = _height;
    final rects = _selectionRects;
    final handleStart = rects.isEmpty ? null : rects.first;
    final handleEnd = rects.isEmpty ? null : rects.last;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onLongPressStart: (details) => _point(
        details.localPosition,
        (x, y) => widget.onWordSelected(widget.page, x, y),
      ),
      onDoubleTapDown: (details) {
        _point(
          details.localPosition,
          (x, y) => widget.onLineSelected(widget.page, x, y),
        );
      },
      onTap: widget.selection == null ? null : widget.onSelectionCleared,
      child: SizedBox(
        width: width,
        height: height,
        child: Stack(
          clipBehavior: Clip.none,
          children: <Widget>[
            Positioned.fill(
              child: ColoredBox(
                color: Colors.white,
                child: _image == null
                    ? _placeholder(context)
                    : RawImage(image: _image, width: width, height: height),
              ),
            ),
            Positioned.fill(
              child: CustomPaint(
                painter: _PdfPagePainter(
                  marks: widget.marks,
                  focus: widget.focus,
                  selection: rects,
                  search: widget.search,
                  overlays: widget.overlays,
                  colors: context.colors,
                ),
              ),
            ),
            if (handleStart != null)
              _SelectionHandle(
                key: ValueKey<String>('pdf-handle-start-${widget.page}'),
                left: handleStart.x0 * width,
                top: handleStart.y1 * height,
                alignBottom: true,
                onDrag: (position) => _point(
                  position,
                  (x, y) => widget.onSelectionExtended(
                    widget.page,
                    x,
                    y,
                    fromStart: true,
                  ),
                ),
              ),
            if (handleEnd != null)
              _SelectionHandle(
                key: ValueKey<String>('pdf-handle-end-${widget.page}'),
                left: handleEnd.x1 * width,
                top: handleEnd.y0 * height,
                alignBottom: false,
                onDrag: (position) => _point(
                  position,
                  (x, y) => widget.onSelectionExtended(
                    widget.page,
                    x,
                    y,
                    fromStart: false,
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _placeholder(BuildContext context) {
    if (_failed) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Text(
            context.l10n.pdfReaderPageFailed,
            textAlign: TextAlign.center,
            style: context.typography.secondary,
          ),
        ),
      );
    }
    return const Center(child: KansoSpinner());
  }
}

/// A drag handle at the end of the selection: dragging it moves that end.
class _SelectionHandle extends StatefulWidget {
  const _SelectionHandle({
    required this.left,
    required this.top,
    required this.alignBottom,
    required this.onDrag,
    super.key,
  });

  /// The handle's centre, in the page.
  final double left;
  final double top;

  /// Whether the handle hangs below its point (the selection's start) or
  /// stands above it (its end).
  final bool alignBottom;

  /// Where the drag has reached, in the page's coordinates.
  final void Function(Offset position) onDrag;

  @override
  State<_SelectionHandle> createState() => _SelectionHandleState();
}

class _SelectionHandleState extends State<_SelectionHandle> {
  static const double _radius = 8;

  Offset? _origin;
  Offset _dragged = Offset.zero;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Positioned(
      left: widget.left - _radius,
      top: widget.alignBottom ? widget.top : widget.top - 2 * _radius,
      width: 2 * _radius,
      height: 2 * _radius,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanStart: (_) {
          _origin = Offset(widget.left, widget.top);
          _dragged = Offset.zero;
        },
        onPanUpdate: (details) {
          final origin = _origin;
          if (origin == null) {
            return;
          }
          _dragged += details.delta;
          widget.onDrag(origin + _dragged);
        },
        onPanEnd: (_) {
          _origin = null;
          _dragged = Offset.zero;
        },
        child: Center(
          child: Container(
            width: 2 * _radius,
            height: 2 * _radius,
            decoration: BoxDecoration(
              color: colors.accent,
              shape: BoxShape.circle,
              border: Border.all(color: colors.editor, width: 2),
            ),
          ),
        ),
      ),
    );
  }
}

/// Draws what lies over a page's picture: its highlights, the selection, the
/// search's hit, and any [PdfPageOverlay].
class _PdfPagePainter extends CustomPainter {
  const _PdfPagePainter({
    required this.marks,
    required this.focus,
    required this.selection,
    required this.search,
    required this.overlays,
    required this.colors,
  });

  final List<PdfMark> marks;
  final String? focus;
  final List<PdfRect> selection;
  final PdfPageMarks? search;
  final List<PdfPageOverlay> overlays;
  final KansoColors colors;

  static Rect _rect(PdfRect rect, Size size) => Rect.fromLTRB(
    rect.x0 * size.width,
    rect.y0 * size.height,
    rect.x1 * size.width,
    rect.y1 * size.height,
  );

  @override
  void paint(Canvas canvas, Size size) {
    final fill = Paint()..style = PaintingStyle.fill;
    for (final mark in marks) {
      final color = highlightColor(mark.color);
      final strong = focus != null && mark.id == focus;
      fill.color = color.withValues(alpha: strong ? 0.55 : 0.35);
      for (final page in mark.marks) {
        for (final rect in page.rects) {
          canvas.drawRect(_rect(rect, size), fill);
        }
      }
      if (strong) {
        final outline = Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = color;
        for (final page in mark.marks) {
          for (final rect in page.rects) {
            canvas.drawRRect(
              RRect.fromRectAndRadius(
                _rect(rect, size).inflate(1),
                const Radius.circular(2),
              ),
              outline,
            );
          }
        }
      }
    }
    final hit = search;
    if (hit != null) {
      fill.color = colors.warning.withValues(alpha: 0.45);
      for (final rect in hit.rects) {
        canvas.drawRect(_rect(rect, size), fill);
      }
    }
    if (selection.isNotEmpty) {
      fill.color = colors.accent.withValues(alpha: 0.28);
      for (final rect in selection) {
        canvas.drawRect(_rect(rect, size), fill);
      }
    }
    for (final overlay in overlays) {
      overlay.paint(canvas, size);
    }
  }

  @override
  bool shouldRepaint(_PdfPagePainter old) =>
      old.marks != marks ||
      old.focus != focus ||
      old.selection != selection ||
      old.search != search ||
      old.colors != colors ||
      old.overlays != overlays;
}
