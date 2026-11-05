// Kanso — fake_canvas.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:dartz/dartz.dart';
import 'package:kanso/canvas/models/repos/abs/canvas_repository.dart';
import 'package:kanso/core/bridge/api/canvas_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';

/// A square outline `size` big at `(x, y)`, as the library would make one.
Float64List squareOutline(double x, double y, [double size = 20]) =>
    Float64List.fromList(<double>[
      x,
      y,
      x + size,
      y,
      x + size,
      y + size,
      x,
      y + size,
    ]);

/// A stroke at `(x, y)`.
CanvasStroke fakeStroke(
  String id, {
  double x = 0,
  double y = 0,
  bool known = false,
  String color = '#1e1e1e',
  String tool = 'pen',
}) => CanvasStroke(
  id: id,
  nodeId: 'ink-1',
  tool: tool,
  color: color,
  opacity: 1,
  width: 3,
  bounds: CanvasRect(x: x, y: y, width: 20, height: 20),
  outline: known ? Float64List(0) : squareOutline(x, y),
  known: known,
);

/// A canvas view with what the test gives it.
CanvasView fakeView({
  List<CanvasStroke> strokes = const <CanvasStroke>[],
  List<CanvasCard> cards = const <CanvasCard>[],
  CanvasBudget budget = CanvasBudget.ok,
  bool dirty = false,
  bool canUndo = false,
  bool canRedo = false,
  String title = 'Sketch',
  String rev = '1-a',
}) => CanvasView(
  noteId: 'canvas-1',
  rev: rev,
  title: title,
  strokes: strokes,
  cards: cards,
  groups: const <CanvasGroup>[],
  arrows: const <CanvasArrow>[],
  bounds: strokes.isEmpty && cards.isEmpty
      ? null
      : const CanvasRect(x: 0, y: 0, width: 400, height: 300),
  bytes: 1200,
  budget: budget,
  canUndo: canUndo,
  canRedo: canRedo,
  dirty: dirty,
);

/// A document that keeps a list of strokes and records what it was asked.
class FakeCanvasDocument implements CanvasDocument {
  FakeCanvasDocument({this.cards = const <CanvasCard>[]});

  final List<CanvasCard> cards;
  final List<CanvasStroke> strokes = <CanvasStroke>[];
  final List<String> calls = <String>[];

  /// The `known` lists the document was given, in order.
  final List<List<String>> knownSeen = <List<String>>[];
  bool dirty = false;
  bool canUndo = false;
  bool canRedo = false;
  CanvasBudget budget = CanvasBudget.ok;
  ApplicationError? saveError;
  ApplicationError? drawError;
  int saves = 0;
  int _next = 0;

  /// What the document is when it is an inline sketch.
  SketchInfo? sketch;

  /// The titles "Open as canvas" was asked with.
  final List<String> canvasTitles = <String>[];
  List<CanvasStroke>? _undone;

  /// The view as it is: strokes already known by the caller come back
  /// without an outline.
  CanvasView current(List<String> known) {
    knownSeen.add(known);
    return fakeView(
      strokes: <CanvasStroke>[
        for (final s in strokes)
          known.contains(s.id)
              ? fakeStroke(s.id, x: s.bounds.x, y: s.bounds.y, known: true)
              : s,
      ],
      cards: cards,
      budget: budget,
      dirty: dirty,
      canUndo: canUndo,
      canRedo: canRedo,
    );
  }

  CanvasResult<CanvasView> _ok(String call, List<String> known) {
    calls.add(call);
    return Right<ApplicationError, CanvasView>(current(known));
  }

  @override
  Future<CanvasResult<CanvasView>> view(List<String> known) async =>
      _ok('view', known);

  @override
  Future<CanvasResult<CanvasView>> draw(
    InkStyle style,
    List<PenSample> samples,
    List<String> known,
  ) async {
    final error = drawError;
    if (error != null) {
      return Left<ApplicationError, CanvasView>(error);
    }
    final first = samples.first;
    strokes.add(fakeStroke('s${_next++}', x: first.x, y: first.y));
    dirty = true;
    canUndo = true;
    return _ok('draw', known);
  }

  @override
  Future<CanvasResult<CanvasView>> eraseStrokesAlong(
    List<CanvasPoint> path,
    double radius,
    List<String> known,
  ) async {
    if (strokes.isNotEmpty) {
      strokes.removeLast();
      dirty = true;
    }
    return _ok('eraseStrokesAlong', known);
  }

  @override
  Future<CanvasResult<CanvasView>> eraseAlong(
    List<CanvasPoint> path,
    double radius,
    List<String> known,
  ) async {
    dirty = true;
    return _ok('eraseAlong', known);
  }

  @override
  Future<CanvasResult<List<StrokeKey>>> select(
    List<CanvasPoint> polygon,
  ) async {
    calls.add('select');
    return Right<ApplicationError, List<StrokeKey>>(<StrokeKey>[
      for (final s in strokes) StrokeKey(nodeId: s.nodeId, strokeId: s.id),
    ]);
  }

  @override
  Future<CanvasResult<CanvasView>> moveStrokes(
    List<StrokeKey> keys,
    double dx,
    double dy,
    List<String> known,
  ) async {
    final moved = <CanvasStroke>[];
    for (final key in keys) {
      final at = strokes.indexWhere((s) => s.id == key.strokeId);
      if (at >= 0) {
        final old = strokes.removeAt(at);
        moved.add(
          fakeStroke('s${_next++}', x: old.bounds.x + dx, y: old.bounds.y + dy),
        );
      }
    }
    strokes.addAll(moved);
    dirty = true;
    return _ok('moveStrokes', known);
  }

  @override
  Future<CanvasResult<CanvasView>> deleteStrokes(
    List<StrokeKey> keys,
    List<String> known,
  ) async {
    strokes.removeWhere((s) => keys.any((k) => k.strokeId == s.id));
    dirty = true;
    return _ok('deleteStrokes', known);
  }

  @override
  Future<CanvasResult<CanvasView>> undo(List<String> known) async {
    if (strokes.isNotEmpty) {
      _undone = <CanvasStroke>[strokes.removeLast()];
      canRedo = true;
      dirty = true;
    }
    return _ok('undo', known);
  }

  @override
  Future<CanvasResult<CanvasView>> redo(List<String> known) async {
    strokes.addAll(_undone ?? const <CanvasStroke>[]);
    _undone = null;
    dirty = true;
    return _ok('redo', known);
  }

  @override
  Future<CanvasResult<CanvasView>> addTextCard(
    double x,
    double y,
    String text,
    List<String> known,
  ) async {
    dirty = true;
    return _ok('addTextCard', known);
  }

  @override
  SketchInfo? get sketchInfo => sketch;

  @override
  Future<CanvasResult<Saved>> openAsCanvas(String title) async {
    canvasTitles.add(title);
    return const Right<ApplicationError, Saved>(
      Saved(id: 'canvas-new', rev: '1-a'),
    );
  }

  @override
  Future<CanvasResult<CanvasSaved>> save() async {
    final error = saveError;
    if (error != null) {
      return Left<ApplicationError, CanvasSaved>(error);
    }
    saves++;
    dirty = false;
    return Right<ApplicationError, CanvasSaved>(
      CanvasSaved(rev: '${saves + 1}-a', bytes: 1200, merged: false),
    );
  }

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnimplementedError('${invocation.memberName} is not faked');
}

/// A repository that opens one fake document, or fails as told.
class FakeCanvasRepository implements CanvasRepository {
  FakeCanvasRepository({FakeCanvasDocument? document, this.openError})
    : document = document ?? FakeCanvasDocument();

  final FakeCanvasDocument document;
  final ApplicationError? openError;
  final Set<String> canvases = <String>{'canvas-1'};

  @override
  bool isCanvas(String noteId) => canvases.contains(noteId);

  @override
  Future<CanvasResult<CanvasDocument>> open(String noteId) async {
    final error = openError;
    return error == null
        ? Right<ApplicationError, CanvasDocument>(document)
        : Left<ApplicationError, CanvasDocument>(error);
  }

  /// The sketches opened, as `note/sketch`.
  final List<String> sketchOpens = <String>[];

  @override
  Future<CanvasResult<CanvasDocument>> openSketch(
    String noteId,
    String sketchId,
  ) async {
    sketchOpens.add('$noteId/$sketchId');
    final error = openError;
    return error == null
        ? Right<ApplicationError, CanvasDocument>(document)
        : Left<ApplicationError, CanvasDocument>(error);
  }

  @override
  List<double> preview(InkStyle style, List<PenSample> samples) => <double>[
    for (final s in samples) ...<double>[s.x, s.y],
  ];

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnimplementedError('${invocation.memberName} is not faked');
}
