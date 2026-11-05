// Kanso — sketch_surface.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../models/data/canvas_tool.dart';
import '../../utils/canvas_viewport.dart';
import '../../utils/pointer_role.dart';
import 'canvas_painter.dart';
import 'canvas_surface.dart' show InkPreview;

/// The drawing box of an inline sketch: pen, mouse and (when asked) finger
/// in; strokes, erases and lasso loops out. It paints with the canvas's own
/// [CanvasPainter] over a viewport that never moves, so a sketch looks as a
/// canvas does and nothing pans or zooms in the note.
///
/// A finger scrolls the note unless [fingerDraws]; the pen, the mouse and
/// the eraser end always draw. The pointers that draw claim the touch, so
/// the note doesn't scroll under a stroke.
class SketchSurface extends StatefulWidget {
  const SketchSurface({
    required this.view,
    required this.box,
    required this.tool,
    required this.color,
    required this.width,
    required this.fingerDraws,
    required this.selection,
    required this.edit,
    required this.theme,
    required this.preview,
    required this.onStroke,
    required this.onErase,
    required this.onLasso,
    required this.onSelectionCleared,
    required this.onSelectionMoved,
    required this.onTouched,
    super.key,
  });

  final CanvasView view;

  /// The sketch's box in points: the surface shows all of it.
  final Size box;
  final CanvasTool tool;
  final String color;
  final double width;
  final bool fingerDraws;
  final Set<String> selection;

  /// Changes whenever the library took an edit: a lifted stroke is painted
  /// until then.
  final int edit;
  final CanvasTheme theme;
  final InkPreview preview;

  final void Function(List<PenSample> samples) onStroke;
  final void Function(
    List<CanvasPoint> path,
    double radius, {
    required bool partial,
  })
  onErase;
  final void Function(List<CanvasPoint> polygon) onLasso;
  final VoidCallback onSelectionCleared;
  final void Function(Offset delta) onSelectionMoved;

  /// A pointer that draws went down: the toolbar shows.
  final VoidCallback onTouched;

  /// The eraser's radius on the screen.
  static const double eraserScreenRadius = 14;

  @override
  State<SketchSurface> createState() => _SketchSurfaceState();
}

class _SketchSurfaceState extends State<SketchSurface> {
  final StrokePaths _paths = StrokePaths();
  int? _actor;
  PointerRole? _role;
  List<PenSample> _samples = <PenSample>[];
  List<Offset> _trail = <Offset>[];
  List<Offset> _lasso = <Offset>[];
  Offset? _eraserAt;
  Offset _shift = Offset.zero;
  bool _draggingSelection = false;
  LiveInk? _live;
  final List<(LiveInk, int)> _settling = <(LiveInk, int)>[];

  /// The viewport that shows the whole box in [size].
  CanvasViewport _viewport(Size size) => CanvasViewport(
    center: Offset(widget.box.width / 2, widget.box.height / 2),
    zoom: size.width / widget.box.width,
  );

  Set<PointerDeviceKind> get _drawers => <PointerDeviceKind>{
    PointerDeviceKind.stylus,
    PointerDeviceKind.invertedStylus,
    PointerDeviceKind.mouse,
    PointerDeviceKind.trackpad,
    if (widget.fingerDraws) PointerDeviceKind.touch,
  };

  @override
  void didUpdateWidget(SketchSurface old) {
    super.didUpdateWidget(old);
    if (old.edit != widget.edit) {
      _settling.removeWhere((s) => s.$2 < widget.edit);
    }
  }

  InkStyle get _style => InkStyle(
    tool: widget.tool.draws ? widget.tool.ink : 'pen',
    color: widget.color,
    width: widget.width,
  );

  PenSample _sample(PointerEvent e, Size size) => penSample(
    canvasPoint: _viewport(size).toCanvas(e.localPosition, size),
    kind: e.kind,
    pressure: e.pressure,
    pressureMin: e.pressureMin,
    pressureMax: e.pressureMax,
    tilt: e.tilt,
    orientation: e.orientation,
  );

  void _down(PointerDownEvent e, Size size) {
    if (_actor != null || !_drawers.contains(e.kind)) {
      return;
    }
    final role = roleFor(
      kind: e.kind,
      tool: widget.tool,
      stylusSeen: false,
      fingerDraws: widget.fingerDraws,
    );
    widget.onTouched();
    final at = _viewport(size).toCanvas(e.localPosition, size);
    switch (role) {
      case PointerRole.ink:
        _actor = e.pointer;
        _role = role;
        _samples = <PenSample>[_sample(e, size)];
        _updateLive();
      case PointerRole.erase:
      case PointerRole.partialErase:
        _actor = e.pointer;
        _role = role;
        _trail = <Offset>[at];
        _eraserAt = e.localPosition;
      case PointerRole.lasso:
        _actor = e.pointer;
        _role = role;
        _beginLasso(at);
      case PointerRole.place:
      case PointerRole.pan:
        return;
    }
    setState(() {});
  }

  void _move(PointerMoveEvent e, Size size) {
    if (e.pointer != _actor) {
      return;
    }
    final viewport = _viewport(size);
    switch (_role) {
      case PointerRole.ink:
        _samples.add(_sample(e, size));
        _updateLive();
      case PointerRole.erase:
      case PointerRole.partialErase:
        _trail.add(viewport.toCanvas(e.localPosition, size));
        _eraserAt = e.localPosition;
      case PointerRole.lasso:
        if (_draggingSelection) {
          _shift += e.localDelta / viewport.zoom;
        } else {
          _lasso = <Offset>[
            ..._lasso,
            viewport.toCanvas(e.localPosition, size),
          ];
        }
      case PointerRole.place || PointerRole.pan || null:
        break;
    }
    setState(() {});
  }

  void _up(PointerEvent e, Size size) {
    if (e.pointer != _actor) {
      return;
    }
    final role = _role;
    _actor = null;
    _role = null;
    switch (role) {
      case PointerRole.ink:
        _endStroke(e is PointerUpEvent);
      case PointerRole.erase:
      case PointerRole.partialErase:
        _endErase(role == PointerRole.partialErase, size, e is PointerUpEvent);
      case PointerRole.lasso:
        _endLasso(e is PointerUpEvent);
      case PointerRole.place || PointerRole.pan || null:
        break;
    }
    setState(() {});
  }

  void _updateLive() {
    final style = _style;
    _live = LiveInk(
      outline: widget.preview(style, List<PenSample>.of(_samples)),
      color: style.color,
      opacity: switch (style.tool) {
        'highlighter' => 0.35,
        'pencil' => 0.85,
        _ => 1,
      },
    );
  }

  void _endStroke(bool committed) {
    final samples = _samples;
    final live = _live;
    _samples = <PenSample>[];
    _live = null;
    if (!committed || samples.isEmpty) {
      return;
    }
    if (live != null) {
      // Painted until the library has taken the stroke.
      _settling.add((live, widget.edit));
    }
    widget.onStroke(samples);
  }

  void _endErase(bool partial, Size size, bool committed) {
    final trail = _trail;
    _trail = <Offset>[];
    _eraserAt = null;
    if (!committed || trail.isEmpty) {
      return;
    }
    widget.onErase(
      <CanvasPoint>[for (final p in trail) CanvasPoint(x: p.dx, y: p.dy)],
      SketchSurface.eraserScreenRadius / _viewport(size).zoom,
      partial: partial,
    );
  }

  Rect? get _selectionBounds {
    if (widget.selection.isEmpty) {
      return null;
    }
    Rect? bounds;
    for (final stroke in widget.view.strokes) {
      if (!widget.selection.contains(stroke.id)) {
        continue;
      }
      final r = stroke.bounds;
      final rect = Rect.fromLTWH(r.x, r.y, r.width, r.height);
      bounds = bounds == null ? rect : bounds.expandToInclude(rect);
    }
    return bounds;
  }

  void _beginLasso(Offset at) {
    final held = _selectionBounds;
    _shift = Offset.zero;
    if (held != null && held.inflate(8).contains(at)) {
      _draggingSelection = true;
      return;
    }
    widget.onSelectionCleared();
    _lasso = <Offset>[at];
  }

  void _endLasso(bool committed) {
    final shift = _shift;
    final lasso = _lasso;
    final dragging = _draggingSelection;
    _shift = Offset.zero;
    _lasso = <Offset>[];
    _draggingSelection = false;
    if (!committed) {
      return;
    }
    if (dragging) {
      if (shift != Offset.zero) {
        widget.onSelectionMoved(shift);
      }
    } else if (lasso.length >= 3) {
      widget.onLasso(<CanvasPoint>[
        for (final p in lasso) CanvasPoint(x: p.dx, y: p.dy),
      ]);
    }
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(
          constraints.maxWidth,
          constraints.maxWidth * widget.box.height / widget.box.width,
        );
        return RawGestureDetector(
          behavior: HitTestBehavior.opaque,
          gestures: <Type, GestureRecognizerFactory>{
            EagerGestureRecognizer:
                GestureRecognizerFactoryWithHandlers<EagerGestureRecognizer>(
                  EagerGestureRecognizer.new,
                  (recognizer) => recognizer.supportedDevices = _drawers,
                ),
          },
          child: Listener(
            behavior: HitTestBehavior.opaque,
            onPointerDown: (e) => _down(e, size),
            onPointerMove: (e) => _move(e, size),
            onPointerUp: (e) => _up(e, size),
            onPointerCancel: (e) => _up(e, size),
            child: SizedBox.fromSize(
              size: size,
              child: ClipRect(
                child: CustomPaint(
                  size: size,
                  painter: CanvasPainter(
                    view: widget.view,
                    viewport: _viewport(size),
                    theme: widget.theme,
                    paths: _paths,
                    live: _live,
                    settling: <LiveInk>[for (final s in _settling) s.$1],
                    lasso: _lasso,
                    eraserAt: _eraserAt,
                    eraserRadius: SketchSurface.eraserScreenRadius,
                    selected: widget.selection,
                    selectionShift: _draggingSelection ? _shift : Offset.zero,
                    selectionBounds: _selectionBounds,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
