// Kanso — canvas_surface.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../models/data/canvas_tool.dart';
import '../../utils/canvas_viewport.dart';
import '../../utils/ink_colors.dart';
import '../../utils/pointer_role.dart';
import 'canvas_painter.dart';

/// Holds where the view is looking and fits it on request; the page's Fit
/// button and the surface share one.
class CanvasViewController extends ValueNotifier<CanvasViewport> {
  CanvasViewController() : super(const CanvasViewport());

  Size _size = Size.zero;

  /// Shows [bounds] whole (the origin when null).
  void fit(Rect? bounds) => value = CanvasViewport.fit(bounds, _size);
}

/// Turns a stroke's samples into the outline to paint while it is drawn.
typedef InkPreview = List<double> Function(InkStyle style, List<PenSample> s);

/// The drawing surface: pen, finger, mouse and trackpad in; strokes, erases,
/// selections and card edits out. The document is not here: the surface
/// paints the [CanvasView] it is given and reports what the person did.
class CanvasSurface extends StatefulWidget {
  const CanvasSurface({
    required this.view,
    required this.tool,
    required this.color,
    required this.width,
    required this.fingerDraws,
    required this.stylusSeen,
    required this.selection,
    required this.edit,
    required this.theme,
    required this.preview,
    required this.controller,
    required this.onStylusSeen,
    required this.onStroke,
    required this.onErase,
    required this.onLasso,
    required this.onSelectionCleared,
    required this.onSelectionMoved,
    required this.onCardMoved,
    required this.onCardTapped,
    required this.onPlace,
    required this.onArrow,
    super.key,
  });

  final CanvasView view;
  final CanvasTool tool;
  final String color;
  final double width;
  final bool fingerDraws;
  final bool stylusSeen;
  final Set<String> selection;

  /// Changes whenever the library took an edit: a lifted stroke is painted
  /// until then.
  final int edit;
  final CanvasTheme theme;
  final InkPreview preview;
  final CanvasViewController controller;

  final VoidCallback onStylusSeen;
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
  final void Function(String id, CanvasRect rect) onCardMoved;
  final void Function(CanvasCard card) onCardTapped;

  /// A tap with the text tool, at a canvas point.
  final void Function(Offset at) onPlace;

  /// Two cards were tapped in turn with the arrow tool.
  final void Function(String from, String to) onArrow;

  /// How far a pointer may move and still be a tap, in screen points.
  static const double tapSlop = 10;

  /// The eraser's radius on the screen.
  static const double eraserScreenRadius = 14;

  @override
  State<CanvasSurface> createState() => _CanvasSurfaceState();
}

class _Touch {
  _Touch(this.kind, this.role, this.start) : last = start;

  final PointerDeviceKind kind;
  final PointerRole role;
  final Offset start;
  Offset last;
  bool moved = false;
}

class _CanvasSurfaceState extends State<CanvasSurface> {
  final StrokePaths _paths = StrokePaths();
  final Map<int, _Touch> _pointers = <int, _Touch>{};

  /// The pointer that is drawing, erasing or dragging, if one is.
  int? _actor;
  List<PenSample> _samples = <PenSample>[];
  List<Offset> _trail = <Offset>[];
  List<Offset> _lasso = <Offset>[];
  Offset? _eraserAt;
  Offset _shift = Offset.zero;
  String? _draggedCard;
  bool _draggingSelection = false;
  String? _arrowFrom;
  LiveInk? _live;
  final List<(LiveInk, int)> _settling = <(LiveInk, int)>[];
  bool _fitted = false;

  // Two-finger gesture state.
  Offset? _focal;
  double? _spread;

  CanvasViewController get _controller => widget.controller;

  /// Whether a pen is making a mark now: touches are palms until it lifts.
  bool get _penDrawing {
    final actor = _actor;
    if (actor == null) {
      return false;
    }
    final kind = _pointers[actor]?.kind;
    return kind == PointerDeviceKind.stylus ||
        kind == PointerDeviceKind.invertedStylus;
  }

  CanvasViewport get _viewport => _controller.value;

  @override
  void initState() {
    super.initState();
    _controller.addListener(_changed);
  }

  @override
  void didUpdateWidget(CanvasSurface old) {
    super.didUpdateWidget(old);
    if (old.controller != widget.controller) {
      old.controller.removeListener(_changed);
      widget.controller.addListener(_changed);
    }
    if (old.edit != widget.edit) {
      _settling.removeWhere((s) => s.$2 < widget.edit);
    }
    if (old.tool != widget.tool) {
      _arrowFrom = null;
    }
  }

  @override
  void dispose() {
    _controller.removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() {});

  Size get _size => _controller._size;

  Offset _canvasPoint(Offset screen) => _viewport.toCanvas(screen, _size);

  PenSample _sample(PointerEvent e) => penSample(
    canvasPoint: _canvasPoint(e.localPosition),
    kind: e.kind,
    pressure: e.pressure,
    pressureMin: e.pressureMin,
    pressureMax: e.pressureMax,
    tilt: e.tilt,
    orientation: e.orientation,
  );

  InkStyle get _style => InkStyle(
    tool: widget.tool.draws ? widget.tool.ink : 'pen',
    color: widget.color,
    width: widget.width,
  );

  // ---- pointer events -----------------------------------------------------

  void _down(PointerDownEvent e) {
    if (e.kind == PointerDeviceKind.stylus ||
        e.kind == PointerDeviceKind.invertedStylus) {
      widget.onStylusSeen();
    }
    final stylusNow =
        widget.stylusSeen ||
        e.kind == PointerDeviceKind.stylus ||
        e.kind == PointerDeviceKind.invertedStylus;
    final role = roleFor(
      kind: e.kind,
      tool: widget.tool,
      stylusSeen: stylusNow,
      fingerDraws: widget.fingerDraws,
    );
    final touch = _Touch(e.kind, role, e.localPosition);
    _pointers[e.pointer] = touch;
    if (_penDrawing && e.kind == PointerDeviceKind.touch) {
      // A palm resting while the pen writes: it neither pans, zooms nor
      // ends the stroke.
      return;
    }
    final fingers = _pointers.values
        .where((t) => t.kind == PointerDeviceKind.touch)
        .length;
    if (fingers >= 2) {
      // A second finger: this is a pan and zoom, not a mark.
      _cancelAction();
      _beginGesture();
      return;
    }
    if (_actor != null) {
      return;
    }
    switch (role) {
      case PointerRole.pan:
        break;
      case PointerRole.ink:
        _actor = e.pointer;
        _samples = <PenSample>[_sample(e)];
        _updateLive();
      case PointerRole.erase:
      case PointerRole.partialErase:
        _actor = e.pointer;
        _trail = <Offset>[_canvasPoint(e.localPosition)];
        _eraserAt = e.localPosition;
      case PointerRole.lasso:
        _actor = e.pointer;
        _beginLasso(e.localPosition);
      case PointerRole.place:
        _actor = e.pointer;
    }
    setState(() {});
  }

  void _move(PointerMoveEvent e) {
    final touch = _pointers[e.pointer];
    if (touch == null) {
      return;
    }
    if (_penDrawing && touch.kind == PointerDeviceKind.touch) {
      return;
    }
    final fingers = _pointers.values
        .where((t) => t.kind == PointerDeviceKind.touch)
        .toList(growable: false);
    if (fingers.length >= 2 && touch.kind == PointerDeviceKind.touch) {
      touch.last = e.localPosition;
      _stepGesture();
      return;
    }
    if ((e.localPosition - touch.start).distance > CanvasSurface.tapSlop) {
      touch.moved = true;
    }
    final delta = e.localPosition - touch.last;
    touch.last = e.localPosition;
    if (e.pointer != _actor) {
      // Nothing pans under a mark being made: the samples already taken
      // would be in a different place from the ones still to come.
      if (touch.role == PointerRole.pan && touch.moved && _actor == null) {
        _controller.value = _viewport.panned(delta);
      }
      return;
    }
    switch (touch.role) {
      case PointerRole.ink:
        _samples.add(_sample(e));
        _updateLive();
      case PointerRole.erase:
      case PointerRole.partialErase:
        _trail.add(_canvasPoint(e.localPosition));
        _eraserAt = e.localPosition;
      case PointerRole.lasso:
        _moveLasso(e.localPosition, delta);
      case PointerRole.place:
      case PointerRole.pan:
        break;
    }
    setState(() {});
  }

  void _up(PointerUpEvent e) {
    final touch = _pointers.remove(e.pointer);
    if (touch == null) {
      return;
    }
    if (_pointers.length < 2) {
      _focal = null;
      _spread = null;
    }
    if (e.pointer != _actor) {
      if (touch.role == PointerRole.pan &&
          !touch.moved &&
          _pointers.isEmpty &&
          _actor == null) {
        _tapped(e.localPosition);
      }
      return;
    }
    _actor = null;
    switch (touch.role) {
      case PointerRole.ink:
        _endStroke(e);
      case PointerRole.erase:
      case PointerRole.partialErase:
        _endErase(touch.role == PointerRole.partialErase);
      case PointerRole.lasso:
        _endLasso(touch, e.localPosition);
      case PointerRole.place:
        if (!touch.moved) {
          _tapped(e.localPosition);
        }
      case PointerRole.pan:
        break;
    }
    setState(() {});
  }

  void _cancel(PointerCancelEvent e) {
    final touch = _pointers.remove(e.pointer);
    if (touch == null) {
      return;
    }
    if (e.pointer == _actor) {
      _cancelAction();
    }
    if (_pointers.length < 2) {
      _focal = null;
      _spread = null;
    }
    setState(() {});
  }

  void _signal(PointerSignalEvent e) {
    if (e is! PointerScrollEvent) {
      return;
    }
    GestureBinding.instance.pointerSignalResolver.register(e, (event) {
      final scroll = event as PointerScrollEvent;
      final keys = HardwareKeyboard.instance;
      if (keys.isControlPressed || keys.isMetaPressed) {
        _controller.value = _viewport.zoomedAt(
          math.exp(-scroll.scrollDelta.dy / 300),
          scroll.localPosition,
          _size,
        );
      } else {
        _controller.value = _viewport.panned(-scroll.scrollDelta);
      }
    });
  }

  double _trackpadScale = 1;

  void _panZoomStart(PointerPanZoomStartEvent e) => _trackpadScale = 1;

  void _panZoomUpdate(PointerPanZoomUpdateEvent e) {
    final factor = e.scale / _trackpadScale;
    _trackpadScale = e.scale;
    _controller.value = _viewport
        .panned(e.localPanDelta)
        .zoomedAt(factor, e.localPosition, _size);
  }

  // ---- gestures -----------------------------------------------------------

  void _beginGesture() {
    final fingers = _fingers();
    if (fingers.length < 2) {
      return;
    }
    _focal = _centerOf(fingers);
    _spread = _spreadOf(fingers);
  }

  void _stepGesture() {
    final fingers = _fingers();
    if (fingers.length < 2 || _focal == null || _spread == null) {
      return;
    }
    final focal = _centerOf(fingers);
    final spread = _spreadOf(fingers);
    _controller.value = _viewport.gestured(
      oldFocal: _focal!,
      newFocal: focal,
      scale: spread / math.max(_spread!, 1),
      size: _size,
    );
    _focal = focal;
    _spread = spread;
  }

  List<Offset> _fingers() => _pointers.values
      .where((t) => t.kind == PointerDeviceKind.touch)
      .map((t) => t.last)
      .toList(growable: false);

  Offset _centerOf(List<Offset> points) {
    var sum = Offset.zero;
    for (final p in points.take(2)) {
      sum += p;
    }
    return sum / 2;
  }

  double _spreadOf(List<Offset> points) => (points[0] - points[1]).distance;

  /// Drops the mark being made: a second finger arrived, or the system
  /// took the pointer.
  void _cancelAction() {
    _actor = null;
    _samples = <PenSample>[];
    _live = null;
    _trail = <Offset>[];
    _eraserAt = null;
    _lasso = <Offset>[];
    _shift = Offset.zero;
    _draggedCard = null;
    _draggingSelection = false;
  }

  // ---- ink ----------------------------------------------------------------

  void _updateLive() {
    final style = _style;
    _live = LiveInk(
      outline: widget.preview(style, List<PenSample>.of(_samples)),
      color: style.color,
      opacity: _opacityOf(style.tool),
    );
  }

  double _opacityOf(String tool) => switch (tool) {
    'highlighter' => 0.35,
    'pencil' => 0.85,
    _ => 1,
  };

  void _endStroke(PointerUpEvent e) {
    final samples = _samples;
    final live = _live;
    _samples = <PenSample>[];
    _live = null;
    if (samples.isEmpty) {
      return;
    }
    if (live != null) {
      // Painted until the library has taken the stroke.
      _settling.add((live, widget.edit));
    }
    widget.onStroke(samples);
  }

  // ---- erasers ------------------------------------------------------------

  void _endErase(bool partial) {
    final trail = _trail;
    _trail = <Offset>[];
    _eraserAt = null;
    if (trail.isEmpty) {
      return;
    }
    widget.onErase(
      <CanvasPoint>[for (final p in trail) CanvasPoint(x: p.dx, y: p.dy)],
      CanvasSurface.eraserScreenRadius / _viewport.zoom,
      partial: partial,
    );
  }

  // ---- lasso, selection and cards -----------------------------------------

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

  CanvasCard? _cardAt(Offset canvasPoint) {
    for (final card in widget.view.cards.reversed) {
      final r = card.rect;
      if (Rect.fromLTWH(r.x, r.y, r.width, r.height).contains(canvasPoint)) {
        return card;
      }
    }
    return null;
  }

  void _beginLasso(Offset screen) {
    final at = _canvasPoint(screen);
    final held = _selectionBounds;
    _shift = Offset.zero;
    if (held != null && held.inflate(8 / _viewport.zoom).contains(at)) {
      _draggingSelection = true;
      return;
    }
    final card = _cardAt(at);
    if (card != null) {
      _draggedCard = card.id;
      return;
    }
    widget.onSelectionCleared();
    _lasso = <Offset>[at];
  }

  void _moveLasso(Offset screen, Offset delta) {
    if (_draggingSelection || _draggedCard != null) {
      _shift += delta / _viewport.zoom;
    } else {
      _lasso = <Offset>[..._lasso, _canvasPoint(screen)];
    }
  }

  void _endLasso(_Touch touch, Offset screen) {
    final shift = _shift;
    final card = _draggedCard;
    final lasso = _lasso;
    final dragging = _draggingSelection;
    _shift = Offset.zero;
    _draggedCard = null;
    _draggingSelection = false;
    _lasso = <Offset>[];
    if (!touch.moved) {
      if (card != null) {
        _tapped(screen);
      }
      return;
    }
    if (dragging) {
      widget.onSelectionMoved(shift);
    } else if (card != null) {
      final c = widget.view.cards.firstWhere((c) => c.id == card);
      widget.onCardMoved(
        card,
        CanvasRect(
          x: c.rect.x + shift.dx,
          y: c.rect.y + shift.dy,
          width: c.rect.width,
          height: c.rect.height,
        ),
      );
    } else if (lasso.length >= 3) {
      widget.onLasso(<CanvasPoint>[
        for (final p in lasso) CanvasPoint(x: p.dx, y: p.dy),
      ]);
    }
  }

  void _tapped(Offset screen) {
    final at = _canvasPoint(screen);
    final card = _cardAt(at);
    if (widget.tool == CanvasTool.arrow) {
      if (card == null) {
        _arrowFrom = null;
      } else if (_arrowFrom == null) {
        _arrowFrom = card.id;
      } else if (_arrowFrom != card.id) {
        widget.onArrow(_arrowFrom!, card.id);
        _arrowFrom = null;
      }
      return;
    }
    if (card != null) {
      widget.onCardTapped(card);
    } else if (widget.tool == CanvasTool.text) {
      widget.onPlace(at);
    }
  }

  // ---- build --------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = constraints.biggest;
        _controller._size = size;
        if (!_fitted && !size.isEmpty) {
          _fitted = true;
          final b = widget.view.bounds;
          if (b != null) {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (mounted) {
                _controller.fit(Rect.fromLTWH(b.x, b.y, b.width, b.height));
              }
            });
          }
        }
        final shiftedCard = _draggedCard == null
            ? null
            : (_draggedCard!, _shift);
        return Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: _down,
          onPointerMove: _move,
          onPointerUp: _up,
          onPointerCancel: _cancel,
          onPointerSignal: _signal,
          onPointerPanZoomStart: _panZoomStart,
          onPointerPanZoomUpdate: _panZoomUpdate,
          child: ClipRect(
            child: CustomPaint(
              size: size,
              painter: CanvasPainter(
                view: widget.view,
                viewport: _viewport,
                theme: widget.theme,
                paths: _paths,
                live: _live,
                settling: <LiveInk>[for (final s in _settling) s.$1],
                lasso: _lasso,
                eraserAt: _eraserAt,
                eraserRadius: CanvasSurface.eraserScreenRadius,
                selected: widget.selection,
                selectionShift: _draggingSelection ? _shift : Offset.zero,
                selectionBounds: _selectionBounds,
                cardShift: shiftedCard,
                pendingArrowFrom: _arrowFrom,
              ),
            ),
          ),
        );
      },
    );
  }
}

/// The colour of the pen the toolbar shows for [hex] on this theme.
Color inkSwatch(String hex, CanvasTheme theme) =>
    inkColor(hex, dark: theme.dark, text: theme.text);
