// Kanso — canvas_state.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'canvas_bloc.dart';

/// How far the canvas has been read.
enum CanvasStatus { loading, ready, locked, failed }

/// What went wrong with the last call, for the notice.
enum CanvasProblem {
  /// The canvas is at its size limit and refused new ink.
  full,

  /// A call failed.
  failed,

  /// The canvas could not be written.
  notSaved,
}

@freezed
abstract class CanvasState with _$CanvasState {
  const factory CanvasState({
    @Default(CanvasStatus.loading) CanvasStatus status,
    CanvasView? view,
    @Default(CanvasTool.pen) CanvasTool tool,

    /// The pen colour, `#rrggbb`.
    @Default('#1e1e1e') String color,

    /// The pen width, in canvas points.
    @Default(3.0) double width,

    /// Whether a finger draws (until a pen is seen); else it pans.
    @Default(true) bool fingerDraws,

    /// Whether a pen has touched this canvas.
    @Default(false) bool stylusSeen,

    /// The strokes a lasso holds.
    @Default(<StrokeKey>[]) List<StrokeKey> selection,

    /// Counts every change of the view, so a stroke being settled can tell
    /// that the library has taken it.
    @Default(0) int edit,
    CanvasProblem? problem,
  }) = _CanvasState;

  const CanvasState._();

  bool get ready => status == CanvasStatus.ready && view != null;
}
