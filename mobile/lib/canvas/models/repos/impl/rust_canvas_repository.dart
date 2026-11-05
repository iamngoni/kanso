// Kanso — rust_canvas_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/canvas.dart' as bridge;
import '../../../../core/bridge/api/canvas_types.dart';
import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/canvas_repository.dart';

/// Canvases over the local Rust library.
@LazySingleton(as: CanvasRepository)
class RustCanvasRepository implements CanvasRepository {
  RustCanvasRepository(KansoLibrary library) : _library = library;

  final KansoLibrary _library;

  @override
  bool isCanvas(String noteId) => _library.isCanvas(id: noteId);

  @override
  Future<CanvasResult<CanvasDocument>> open(String noteId) => attempt(
    () async => _RustCanvasDocument(await _library.canvas(id: noteId)),
  );

  @override
  Future<CanvasResult<Saved>> create({String? notebookId, String title = ''}) =>
      attempt(
        () => _library.createCanvas(notebookId: notebookId, title: title),
      );

  @override
  Future<CanvasResult<Saved>> importExcalidraw({
    required String source,
    String? notebookId,
    String title = '',
  }) => attempt(
    () => _library.importExcalidraw(
      source: source,
      notebookId: notebookId,
      title: title,
    ),
  );

  @override
  List<double> preview(InkStyle style, List<PenSample> samples) =>
      bridge.canvasInkPreview(style: style, samples: samples);
}

class _RustCanvasDocument implements CanvasDocument {
  _RustCanvasDocument(this._canvas);

  final bridge.KansoCanvas _canvas;

  @override
  Future<CanvasResult<CanvasView>> view(List<String> known) =>
      attempt(() => _canvas.view(known: known));

  @override
  Future<CanvasResult<CanvasView>> draw(
    InkStyle style,
    List<PenSample> samples,
    List<String> known,
  ) => attempt(
    () => _canvas.draw(style: style, samples: samples, known: known),
  );

  @override
  Future<CanvasResult<CanvasView>> eraseStrokesAlong(
    List<CanvasPoint> path,
    double radius,
    List<String> known,
  ) => attempt(
    () => _canvas.eraseStrokesAlong(path: path, radius: radius, known: known),
  );

  @override
  Future<CanvasResult<CanvasView>> eraseAlong(
    List<CanvasPoint> path,
    double radius,
    List<String> known,
  ) => attempt(
    () => _canvas.eraseAlong(path: path, radius: radius, known: known),
  );

  @override
  Future<CanvasResult<List<StrokeKey>>> select(List<CanvasPoint> polygon) =>
      attempt(() => _canvas.select(polygon: polygon));

  @override
  Future<CanvasResult<CanvasView>> deleteStrokes(
    List<StrokeKey> keys,
    List<String> known,
  ) => attempt(() => _canvas.deleteStrokes(keys: keys, known: known));

  @override
  Future<CanvasResult<CanvasView>> moveStrokes(
    List<StrokeKey> keys,
    double dx,
    double dy,
    List<String> known,
  ) => attempt(
    () => _canvas.moveStrokes(keys: keys, dx: dx, dy: dy, known: known),
  );

  @override
  Future<CanvasResult<CanvasView>> addTextCard(
    double x,
    double y,
    String text,
    List<String> known,
  ) => attempt(
    () => _canvas.addTextCard(x: x, y: y, text: text, known: known),
  );

  @override
  Future<CanvasResult<CanvasView>> addNoteCard(
    String noteId,
    double x,
    double y,
    List<String> known,
  ) => attempt(
    () => _canvas.addNoteCard(noteId: noteId, x: x, y: y, known: known),
  );

  @override
  Future<CanvasResult<CanvasView>> setCardText(
    String id,
    String text,
    List<String> known,
  ) => attempt(() => _canvas.setCardText(id: id, text: text, known: known));

  @override
  Future<CanvasResult<CanvasView>> setCardRect(
    String id,
    CanvasRect rect,
    List<String> known,
  ) => attempt(() => _canvas.setCardRect(id: id, rect: rect, known: known));

  @override
  Future<CanvasResult<CanvasView>> deleteCard(String id, List<String> known) =>
      attempt(() => _canvas.deleteCard(id: id, known: known));

  @override
  Future<CanvasResult<CanvasView>> addArrow(
    String from,
    String to,
    List<String> known,
  ) => attempt(() => _canvas.addArrow(from: from, to: to, known: known));

  @override
  Future<CanvasResult<CanvasView>> undo(List<String> known) =>
      attempt(() => _canvas.undo(known: known));

  @override
  Future<CanvasResult<CanvasView>> redo(List<String> known) =>
      attempt(() => _canvas.redo(known: known));

  @override
  Future<CanvasResult<CanvasSaved>> save() => attempt(_canvas.save);

  @override
  Future<CanvasResult<(Saved, CanvasView)>> moveToNewCanvas(
    List<StrokeKey> keys,
    String title,
    List<String> known,
  ) => attempt(
    () => _canvas.moveToNewCanvas(keys: keys, title: title, known: known),
  );

  @override
  Future<CanvasResult<String>> exportSvg({required bool dark}) =>
      attempt(() => _canvas.exportSvg(dark: dark));

  @override
  Future<CanvasResult<String>> exportExcalidraw() =>
      attempt(_canvas.exportExcalidraw);

  @override
  Future<CanvasResult<CanvasPulse>> pulse(List<String> known) =>
      attempt(() => _canvas.pulse(known: known));

  @override
  Future<CanvasResult<List<CanvasPointerRow>>> pointers() =>
      attempt(_canvas.pointers);

  @override
  Future<CanvasResult<bool>> streamStroke(
    InkStyle style,
    List<PenSample> samples,
  ) => attempt(() => _canvas.streamStroke(style: style, samples: samples));

  @override
  Future<CanvasResult<Unit>> cancelStroke() => attempt(() async {
    await _canvas.cancelStroke();
    return unit;
  });

  @override
  Future<CanvasResult<Unit>> setPointer(double x, double y, String tool) =>
      attempt(() async {
        await _canvas.setPointer(x: x, y: y, tool: tool);
        return unit;
      });

  @override
  Future<CanvasResult<Unit>> clearPointer() => attempt(() async {
    await _canvas.clearPointer();
    return unit;
  });

  @override
  Future<CanvasResult<Unit>> reconnect() => attempt(() async {
    await _canvas.reconnect();
    return unit;
  });

  @override
  Future<CanvasResult<Unit>> close() => attempt(() async {
    await _canvas.close();
    return unit;
  });
}
