// Kanso — canvas_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/canvas_types.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// A result of a canvas call.
typedef CanvasResult<T> = Either<ApplicationError, T>;

/// Canvases in the library: telling one from a note, making one, and
/// opening one for drawing.
abstract class CanvasRepository {
  /// Whether note [noteId] is a canvas (so the note route shows the canvas
  /// view). Reads nothing of a locked one.
  bool isCanvas(String noteId);

  /// Opens canvas [noteId]. A locked, closed one is a [LockedError].
  Future<CanvasResult<CanvasDocument>> open(String noteId);

  /// Opens sketch [sketchId] of Markdown note [noteId] for drawing: the same
  /// document calls as a canvas, saved back into the note's sketch. A locked,
  /// closed note is a [LockedError].
  Future<CanvasResult<CanvasDocument>> openSketch(
    String noteId,
    String sketchId,
  );

  /// [body] with an empty [width] by [height] sketch put in at [at] (UTF-16
  /// units), or why not.
  Future<CanvasResult<SketchInserted>> insertSketch({
    required String body,
    required int at,
    required int width,
    required int height,
  });

  /// Makes an empty canvas in [notebookId] (the preferred notebook, then
  /// the first, when null).
  Future<CanvasResult<Saved>> create({String? notebookId, String title = ''});

  /// Makes a canvas from an Excalidraw scene.
  Future<CanvasResult<Saved>> importExcalidraw({
    required String source,
    String? notebookId,
    String title = '',
  });

  /// The outline of a stroke still being drawn, `x0, y0, x1, y1, ...`.
  List<double> preview(InkStyle style, List<PenSample> samples);
}

/// One canvas open for drawing. Every call that changes it returns the
/// canvas as it is now; strokes whose ID is in `known` come back without
/// their outline (the caller holds it).
abstract class CanvasDocument {
  Future<CanvasResult<CanvasView>> view(List<String> known);

  Future<CanvasResult<CanvasView>> draw(
    InkStyle style,
    List<PenSample> samples,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> eraseStrokesAlong(
    List<CanvasPoint> path,
    double radius,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> eraseAlong(
    List<CanvasPoint> path,
    double radius,
    List<String> known,
  );

  Future<CanvasResult<List<StrokeKey>>> select(List<CanvasPoint> polygon);

  Future<CanvasResult<CanvasView>> deleteStrokes(
    List<StrokeKey> keys,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> moveStrokes(
    List<StrokeKey> keys,
    double dx,
    double dy,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> addTextCard(
    double x,
    double y,
    String text,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> addNoteCard(
    String noteId,
    double x,
    double y,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> setCardText(
    String id,
    String text,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> setCardRect(
    String id,
    CanvasRect rect,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> deleteCard(String id, List<String> known);

  Future<CanvasResult<CanvasView>> addArrow(
    String from,
    String to,
    List<String> known,
  );

  Future<CanvasResult<CanvasView>> undo(List<String> known);

  Future<CanvasResult<CanvasView>> redo(List<String> known);

  /// Writes the canvas if it changed.
  Future<CanvasResult<CanvasSaved>> save();

  Future<CanvasResult<(Saved, CanvasView)>> moveToNewCanvas(
    List<StrokeKey> keys,
    String title,
    List<String> known,
  );

  Future<CanvasResult<String>> exportSvg({required bool dark});

  Future<CanvasResult<String>> exportExcalidraw();

  /// What this is when it is an inline sketch: its ID, box and whether it is
  /// read only. Null for a canvas.
  SketchInfo? get sketchInfo;

  /// "Open as canvas" (or "Copy as canvas" when read only) on a sketch: a
  /// new canvas note titled [title].
  Future<CanvasResult<Saved>> openAsCanvas(String title);
}
