// Kanso — canvas_event.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'canvas_bloc.dart';

@freezed
sealed class CanvasEvent with _$CanvasEvent {
  /// Opens canvas [noteId].
  const factory CanvasEvent.started(String noteId) = CanvasStartedEvent;

  /// A pen touched the screen: fingers pan and zoom from now on.
  const factory CanvasEvent.stylusSeen() = CanvasStylusSeenEvent;

  const factory CanvasEvent.toolChanged(CanvasTool tool) =
      CanvasToolChangedEvent;
  const factory CanvasEvent.colorChanged(String color) =
      CanvasColorChangedEvent;
  const factory CanvasEvent.widthChanged(double width) =
      CanvasWidthChangedEvent;
  const factory CanvasEvent.fingerDrawsChanged({required bool on}) =
      CanvasFingerDrawsChangedEvent;

  /// A stroke ended: draw it, then save.
  const factory CanvasEvent.strokeCommitted(List<PenSample> samples) =
      CanvasStrokeCommittedEvent;

  /// An eraser was dragged along [path] (canvas points).
  const factory CanvasEvent.erased({
    required List<CanvasPoint> path,
    required double radius,
    required bool partial,
  }) = CanvasErasedEvent;

  const factory CanvasEvent.lassoClosed(List<CanvasPoint> polygon) =
      CanvasLassoClosedEvent;
  const factory CanvasEvent.selectionCleared() = CanvasSelectionClearedEvent;
  const factory CanvasEvent.selectionDeleted() = CanvasSelectionDeletedEvent;
  const factory CanvasEvent.selectionMoved(double dx, double dy) =
      CanvasSelectionMovedEvent;
  const factory CanvasEvent.selectionMovedToNewCanvas(String title) =
      CanvasSelectionMovedToNewCanvasEvent;

  const factory CanvasEvent.textCardAdded(double x, double y, String text) =
      CanvasTextCardAddedEvent;
  const factory CanvasEvent.noteCardAdded(String noteId, double x, double y) =
      CanvasNoteCardAddedEvent;
  const factory CanvasEvent.cardTextChanged(String id, String text) =
      CanvasCardTextChangedEvent;
  const factory CanvasEvent.cardMoved(String id, CanvasRect rect) =
      CanvasCardMovedEvent;
  const factory CanvasEvent.cardDeleted(String id) = CanvasCardDeletedEvent;
  const factory CanvasEvent.arrowAdded(String from, String to) =
      CanvasArrowAddedEvent;

  const factory CanvasEvent.undone() = CanvasUndoneEvent;
  const factory CanvasEvent.redone() = CanvasRedoneEvent;

  /// Two seconds without input: write what changed.
  const factory CanvasEvent.saveRequested() = CanvasSaveRequestedEvent;

  /// The person read the message about the last failure.
  const factory CanvasEvent.messageDismissed() = CanvasMessageDismissedEvent;

  // ---- a shared canvas ----------------------------------------------------

  /// A finger, pen or mouse went down on the surface ([active]), or the last
  /// of them lifted. The bloc counts the gestures the moment this is added,
  /// so what others drew waits for the person's hands to be free.
  const factory CanvasEvent.gestureChanged({required bool active}) =
      CanvasGestureChangedEvent;

  /// The room's tick: hand it what was drawn and take what was drawn there.
  const factory CanvasEvent.pulsed() = CanvasPulsedEvent;

  /// The stroke being drawn, with every sample so far, to stream to the
  /// others.
  const factory CanvasEvent.strokeStreamed(List<PenSample> samples) =
      CanvasStrokeStreamedEvent;

  /// The pen's stroke was dropped (a second finger, the system): the others
  /// drop what they got of it.
  const factory CanvasEvent.strokeCancelled() = CanvasStrokeCancelledEvent;

  /// The pointer is at a canvas point.
  const factory CanvasEvent.pointerMoved(double x, double y) =
      CanvasPointerMovedEvent;

  /// The pointer left the surface.
  const factory CanvasEvent.pointerLeft() = CanvasPointerLeftEvent;

  /// The app is back in front.
  const factory CanvasEvent.resumed() = CanvasResumedEvent;
}
