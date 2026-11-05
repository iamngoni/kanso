// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'canvas_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CanvasEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent()';
}


}

/// @nodoc
class $CanvasEventCopyWith<$Res>  {
$CanvasEventCopyWith(CanvasEvent _, $Res Function(CanvasEvent) __);
}


/// Adds pattern-matching-related methods to [CanvasEvent].
extension CanvasEventPatterns on CanvasEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CanvasStartedEvent value)?  started,TResult Function( CanvasSketchStartedEvent value)?  sketchStarted,TResult Function( CanvasStylusSeenEvent value)?  stylusSeen,TResult Function( CanvasToolChangedEvent value)?  toolChanged,TResult Function( CanvasColorChangedEvent value)?  colorChanged,TResult Function( CanvasWidthChangedEvent value)?  widthChanged,TResult Function( CanvasFingerDrawsChangedEvent value)?  fingerDrawsChanged,TResult Function( CanvasStrokeCommittedEvent value)?  strokeCommitted,TResult Function( CanvasErasedEvent value)?  erased,TResult Function( CanvasLassoClosedEvent value)?  lassoClosed,TResult Function( CanvasSelectionClearedEvent value)?  selectionCleared,TResult Function( CanvasSelectionDeletedEvent value)?  selectionDeleted,TResult Function( CanvasSelectionMovedEvent value)?  selectionMoved,TResult Function( CanvasSelectionMovedToNewCanvasEvent value)?  selectionMovedToNewCanvas,TResult Function( CanvasTextCardAddedEvent value)?  textCardAdded,TResult Function( CanvasNoteCardAddedEvent value)?  noteCardAdded,TResult Function( CanvasCardTextChangedEvent value)?  cardTextChanged,TResult Function( CanvasCardMovedEvent value)?  cardMoved,TResult Function( CanvasCardDeletedEvent value)?  cardDeleted,TResult Function( CanvasArrowAddedEvent value)?  arrowAdded,TResult Function( CanvasUndoneEvent value)?  undone,TResult Function( CanvasRedoneEvent value)?  redone,TResult Function( CanvasSaveRequestedEvent value)?  saveRequested,TResult Function( CanvasMessageDismissedEvent value)?  messageDismissed,TResult Function( CanvasGestureChangedEvent value)?  gestureChanged,TResult Function( CanvasPulsedEvent value)?  pulsed,TResult Function( CanvasStrokeStreamedEvent value)?  strokeStreamed,TResult Function( CanvasStrokeCancelledEvent value)?  strokeCancelled,TResult Function( CanvasPointerMovedEvent value)?  pointerMoved,TResult Function( CanvasPointerLeftEvent value)?  pointerLeft,TResult Function( CanvasResumedEvent value)?  resumed,TResult Function( CanvasPausedEvent value)?  paused,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CanvasStartedEvent() when started != null:
return started(_that);case CanvasSketchStartedEvent() when sketchStarted != null:
return sketchStarted(_that);case CanvasStylusSeenEvent() when stylusSeen != null:
return stylusSeen(_that);case CanvasToolChangedEvent() when toolChanged != null:
return toolChanged(_that);case CanvasColorChangedEvent() when colorChanged != null:
return colorChanged(_that);case CanvasWidthChangedEvent() when widthChanged != null:
return widthChanged(_that);case CanvasFingerDrawsChangedEvent() when fingerDrawsChanged != null:
return fingerDrawsChanged(_that);case CanvasStrokeCommittedEvent() when strokeCommitted != null:
return strokeCommitted(_that);case CanvasErasedEvent() when erased != null:
return erased(_that);case CanvasLassoClosedEvent() when lassoClosed != null:
return lassoClosed(_that);case CanvasSelectionClearedEvent() when selectionCleared != null:
return selectionCleared(_that);case CanvasSelectionDeletedEvent() when selectionDeleted != null:
return selectionDeleted(_that);case CanvasSelectionMovedEvent() when selectionMoved != null:
return selectionMoved(_that);case CanvasSelectionMovedToNewCanvasEvent() when selectionMovedToNewCanvas != null:
return selectionMovedToNewCanvas(_that);case CanvasTextCardAddedEvent() when textCardAdded != null:
return textCardAdded(_that);case CanvasNoteCardAddedEvent() when noteCardAdded != null:
return noteCardAdded(_that);case CanvasCardTextChangedEvent() when cardTextChanged != null:
return cardTextChanged(_that);case CanvasCardMovedEvent() when cardMoved != null:
return cardMoved(_that);case CanvasCardDeletedEvent() when cardDeleted != null:
return cardDeleted(_that);case CanvasArrowAddedEvent() when arrowAdded != null:
return arrowAdded(_that);case CanvasUndoneEvent() when undone != null:
return undone(_that);case CanvasRedoneEvent() when redone != null:
return redone(_that);case CanvasSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that);case CanvasMessageDismissedEvent() when messageDismissed != null:
return messageDismissed(_that);case CanvasGestureChangedEvent() when gestureChanged != null:
return gestureChanged(_that);case CanvasPulsedEvent() when pulsed != null:
return pulsed(_that);case CanvasStrokeStreamedEvent() when strokeStreamed != null:
return strokeStreamed(_that);case CanvasStrokeCancelledEvent() when strokeCancelled != null:
return strokeCancelled(_that);case CanvasPointerMovedEvent() when pointerMoved != null:
return pointerMoved(_that);case CanvasPointerLeftEvent() when pointerLeft != null:
return pointerLeft(_that);case CanvasResumedEvent() when resumed != null:
return resumed(_that);case CanvasPausedEvent() when paused != null:
return paused(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CanvasStartedEvent value)  started,required TResult Function( CanvasSketchStartedEvent value)  sketchStarted,required TResult Function( CanvasStylusSeenEvent value)  stylusSeen,required TResult Function( CanvasToolChangedEvent value)  toolChanged,required TResult Function( CanvasColorChangedEvent value)  colorChanged,required TResult Function( CanvasWidthChangedEvent value)  widthChanged,required TResult Function( CanvasFingerDrawsChangedEvent value)  fingerDrawsChanged,required TResult Function( CanvasStrokeCommittedEvent value)  strokeCommitted,required TResult Function( CanvasErasedEvent value)  erased,required TResult Function( CanvasLassoClosedEvent value)  lassoClosed,required TResult Function( CanvasSelectionClearedEvent value)  selectionCleared,required TResult Function( CanvasSelectionDeletedEvent value)  selectionDeleted,required TResult Function( CanvasSelectionMovedEvent value)  selectionMoved,required TResult Function( CanvasSelectionMovedToNewCanvasEvent value)  selectionMovedToNewCanvas,required TResult Function( CanvasTextCardAddedEvent value)  textCardAdded,required TResult Function( CanvasNoteCardAddedEvent value)  noteCardAdded,required TResult Function( CanvasCardTextChangedEvent value)  cardTextChanged,required TResult Function( CanvasCardMovedEvent value)  cardMoved,required TResult Function( CanvasCardDeletedEvent value)  cardDeleted,required TResult Function( CanvasArrowAddedEvent value)  arrowAdded,required TResult Function( CanvasUndoneEvent value)  undone,required TResult Function( CanvasRedoneEvent value)  redone,required TResult Function( CanvasSaveRequestedEvent value)  saveRequested,required TResult Function( CanvasMessageDismissedEvent value)  messageDismissed,required TResult Function( CanvasGestureChangedEvent value)  gestureChanged,required TResult Function( CanvasPulsedEvent value)  pulsed,required TResult Function( CanvasStrokeStreamedEvent value)  strokeStreamed,required TResult Function( CanvasStrokeCancelledEvent value)  strokeCancelled,required TResult Function( CanvasPointerMovedEvent value)  pointerMoved,required TResult Function( CanvasPointerLeftEvent value)  pointerLeft,required TResult Function( CanvasResumedEvent value)  resumed,required TResult Function( CanvasPausedEvent value)  paused,}){
final _that = this;
switch (_that) {
case CanvasStartedEvent():
return started(_that);case CanvasSketchStartedEvent():
return sketchStarted(_that);case CanvasStylusSeenEvent():
return stylusSeen(_that);case CanvasToolChangedEvent():
return toolChanged(_that);case CanvasColorChangedEvent():
return colorChanged(_that);case CanvasWidthChangedEvent():
return widthChanged(_that);case CanvasFingerDrawsChangedEvent():
return fingerDrawsChanged(_that);case CanvasStrokeCommittedEvent():
return strokeCommitted(_that);case CanvasErasedEvent():
return erased(_that);case CanvasLassoClosedEvent():
return lassoClosed(_that);case CanvasSelectionClearedEvent():
return selectionCleared(_that);case CanvasSelectionDeletedEvent():
return selectionDeleted(_that);case CanvasSelectionMovedEvent():
return selectionMoved(_that);case CanvasSelectionMovedToNewCanvasEvent():
return selectionMovedToNewCanvas(_that);case CanvasTextCardAddedEvent():
return textCardAdded(_that);case CanvasNoteCardAddedEvent():
return noteCardAdded(_that);case CanvasCardTextChangedEvent():
return cardTextChanged(_that);case CanvasCardMovedEvent():
return cardMoved(_that);case CanvasCardDeletedEvent():
return cardDeleted(_that);case CanvasArrowAddedEvent():
return arrowAdded(_that);case CanvasUndoneEvent():
return undone(_that);case CanvasRedoneEvent():
return redone(_that);case CanvasSaveRequestedEvent():
return saveRequested(_that);case CanvasMessageDismissedEvent():
return messageDismissed(_that);case CanvasGestureChangedEvent():
return gestureChanged(_that);case CanvasPulsedEvent():
return pulsed(_that);case CanvasStrokeStreamedEvent():
return strokeStreamed(_that);case CanvasStrokeCancelledEvent():
return strokeCancelled(_that);case CanvasPointerMovedEvent():
return pointerMoved(_that);case CanvasPointerLeftEvent():
return pointerLeft(_that);case CanvasResumedEvent():
return resumed(_that);case CanvasPausedEvent():
return paused(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CanvasStartedEvent value)?  started,TResult? Function( CanvasSketchStartedEvent value)?  sketchStarted,TResult? Function( CanvasStylusSeenEvent value)?  stylusSeen,TResult? Function( CanvasToolChangedEvent value)?  toolChanged,TResult? Function( CanvasColorChangedEvent value)?  colorChanged,TResult? Function( CanvasWidthChangedEvent value)?  widthChanged,TResult? Function( CanvasFingerDrawsChangedEvent value)?  fingerDrawsChanged,TResult? Function( CanvasStrokeCommittedEvent value)?  strokeCommitted,TResult? Function( CanvasErasedEvent value)?  erased,TResult? Function( CanvasLassoClosedEvent value)?  lassoClosed,TResult? Function( CanvasSelectionClearedEvent value)?  selectionCleared,TResult? Function( CanvasSelectionDeletedEvent value)?  selectionDeleted,TResult? Function( CanvasSelectionMovedEvent value)?  selectionMoved,TResult? Function( CanvasSelectionMovedToNewCanvasEvent value)?  selectionMovedToNewCanvas,TResult? Function( CanvasTextCardAddedEvent value)?  textCardAdded,TResult? Function( CanvasNoteCardAddedEvent value)?  noteCardAdded,TResult? Function( CanvasCardTextChangedEvent value)?  cardTextChanged,TResult? Function( CanvasCardMovedEvent value)?  cardMoved,TResult? Function( CanvasCardDeletedEvent value)?  cardDeleted,TResult? Function( CanvasArrowAddedEvent value)?  arrowAdded,TResult? Function( CanvasUndoneEvent value)?  undone,TResult? Function( CanvasRedoneEvent value)?  redone,TResult? Function( CanvasSaveRequestedEvent value)?  saveRequested,TResult? Function( CanvasMessageDismissedEvent value)?  messageDismissed,TResult? Function( CanvasGestureChangedEvent value)?  gestureChanged,TResult? Function( CanvasPulsedEvent value)?  pulsed,TResult? Function( CanvasStrokeStreamedEvent value)?  strokeStreamed,TResult? Function( CanvasStrokeCancelledEvent value)?  strokeCancelled,TResult? Function( CanvasPointerMovedEvent value)?  pointerMoved,TResult? Function( CanvasPointerLeftEvent value)?  pointerLeft,TResult? Function( CanvasResumedEvent value)?  resumed,TResult? Function( CanvasPausedEvent value)?  paused,}){
final _that = this;
switch (_that) {
case CanvasStartedEvent() when started != null:
return started(_that);case CanvasSketchStartedEvent() when sketchStarted != null:
return sketchStarted(_that);case CanvasStylusSeenEvent() when stylusSeen != null:
return stylusSeen(_that);case CanvasToolChangedEvent() when toolChanged != null:
return toolChanged(_that);case CanvasColorChangedEvent() when colorChanged != null:
return colorChanged(_that);case CanvasWidthChangedEvent() when widthChanged != null:
return widthChanged(_that);case CanvasFingerDrawsChangedEvent() when fingerDrawsChanged != null:
return fingerDrawsChanged(_that);case CanvasStrokeCommittedEvent() when strokeCommitted != null:
return strokeCommitted(_that);case CanvasErasedEvent() when erased != null:
return erased(_that);case CanvasLassoClosedEvent() when lassoClosed != null:
return lassoClosed(_that);case CanvasSelectionClearedEvent() when selectionCleared != null:
return selectionCleared(_that);case CanvasSelectionDeletedEvent() when selectionDeleted != null:
return selectionDeleted(_that);case CanvasSelectionMovedEvent() when selectionMoved != null:
return selectionMoved(_that);case CanvasSelectionMovedToNewCanvasEvent() when selectionMovedToNewCanvas != null:
return selectionMovedToNewCanvas(_that);case CanvasTextCardAddedEvent() when textCardAdded != null:
return textCardAdded(_that);case CanvasNoteCardAddedEvent() when noteCardAdded != null:
return noteCardAdded(_that);case CanvasCardTextChangedEvent() when cardTextChanged != null:
return cardTextChanged(_that);case CanvasCardMovedEvent() when cardMoved != null:
return cardMoved(_that);case CanvasCardDeletedEvent() when cardDeleted != null:
return cardDeleted(_that);case CanvasArrowAddedEvent() when arrowAdded != null:
return arrowAdded(_that);case CanvasUndoneEvent() when undone != null:
return undone(_that);case CanvasRedoneEvent() when redone != null:
return redone(_that);case CanvasSaveRequestedEvent() when saveRequested != null:
return saveRequested(_that);case CanvasMessageDismissedEvent() when messageDismissed != null:
return messageDismissed(_that);case CanvasGestureChangedEvent() when gestureChanged != null:
return gestureChanged(_that);case CanvasPulsedEvent() when pulsed != null:
return pulsed(_that);case CanvasStrokeStreamedEvent() when strokeStreamed != null:
return strokeStreamed(_that);case CanvasStrokeCancelledEvent() when strokeCancelled != null:
return strokeCancelled(_that);case CanvasPointerMovedEvent() when pointerMoved != null:
return pointerMoved(_that);case CanvasPointerLeftEvent() when pointerLeft != null:
return pointerLeft(_that);case CanvasResumedEvent() when resumed != null:
return resumed(_that);case CanvasPausedEvent() when paused != null:
return paused(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function( String noteId,  String sketchId)?  sketchStarted,TResult Function()?  stylusSeen,TResult Function( CanvasTool tool)?  toolChanged,TResult Function( String color)?  colorChanged,TResult Function( double width)?  widthChanged,TResult Function( bool on)?  fingerDrawsChanged,TResult Function( List<PenSample> samples)?  strokeCommitted,TResult Function( List<CanvasPoint> path,  double radius,  bool partial)?  erased,TResult Function( List<CanvasPoint> polygon)?  lassoClosed,TResult Function()?  selectionCleared,TResult Function()?  selectionDeleted,TResult Function( double dx,  double dy)?  selectionMoved,TResult Function( String title)?  selectionMovedToNewCanvas,TResult Function( double x,  double y,  String text)?  textCardAdded,TResult Function( String noteId,  double x,  double y)?  noteCardAdded,TResult Function( String id,  String text)?  cardTextChanged,TResult Function( String id,  CanvasRect rect)?  cardMoved,TResult Function( String id)?  cardDeleted,TResult Function( String from,  String to)?  arrowAdded,TResult Function()?  undone,TResult Function()?  redone,TResult Function()?  saveRequested,TResult Function()?  messageDismissed,TResult Function( bool active)?  gestureChanged,TResult Function()?  pulsed,TResult Function( List<PenSample> samples)?  strokeStreamed,TResult Function()?  strokeCancelled,TResult Function( double x,  double y)?  pointerMoved,TResult Function()?  pointerLeft,TResult Function()?  resumed,TResult Function()?  paused,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CanvasStartedEvent() when started != null:
return started(_that.noteId);case CanvasSketchStartedEvent() when sketchStarted != null:
return sketchStarted(_that.noteId,_that.sketchId);case CanvasStylusSeenEvent() when stylusSeen != null:
return stylusSeen();case CanvasToolChangedEvent() when toolChanged != null:
return toolChanged(_that.tool);case CanvasColorChangedEvent() when colorChanged != null:
return colorChanged(_that.color);case CanvasWidthChangedEvent() when widthChanged != null:
return widthChanged(_that.width);case CanvasFingerDrawsChangedEvent() when fingerDrawsChanged != null:
return fingerDrawsChanged(_that.on);case CanvasStrokeCommittedEvent() when strokeCommitted != null:
return strokeCommitted(_that.samples);case CanvasErasedEvent() when erased != null:
return erased(_that.path,_that.radius,_that.partial);case CanvasLassoClosedEvent() when lassoClosed != null:
return lassoClosed(_that.polygon);case CanvasSelectionClearedEvent() when selectionCleared != null:
return selectionCleared();case CanvasSelectionDeletedEvent() when selectionDeleted != null:
return selectionDeleted();case CanvasSelectionMovedEvent() when selectionMoved != null:
return selectionMoved(_that.dx,_that.dy);case CanvasSelectionMovedToNewCanvasEvent() when selectionMovedToNewCanvas != null:
return selectionMovedToNewCanvas(_that.title);case CanvasTextCardAddedEvent() when textCardAdded != null:
return textCardAdded(_that.x,_that.y,_that.text);case CanvasNoteCardAddedEvent() when noteCardAdded != null:
return noteCardAdded(_that.noteId,_that.x,_that.y);case CanvasCardTextChangedEvent() when cardTextChanged != null:
return cardTextChanged(_that.id,_that.text);case CanvasCardMovedEvent() when cardMoved != null:
return cardMoved(_that.id,_that.rect);case CanvasCardDeletedEvent() when cardDeleted != null:
return cardDeleted(_that.id);case CanvasArrowAddedEvent() when arrowAdded != null:
return arrowAdded(_that.from,_that.to);case CanvasUndoneEvent() when undone != null:
return undone();case CanvasRedoneEvent() when redone != null:
return redone();case CanvasSaveRequestedEvent() when saveRequested != null:
return saveRequested();case CanvasMessageDismissedEvent() when messageDismissed != null:
return messageDismissed();case CanvasGestureChangedEvent() when gestureChanged != null:
return gestureChanged(_that.active);case CanvasPulsedEvent() when pulsed != null:
return pulsed();case CanvasStrokeStreamedEvent() when strokeStreamed != null:
return strokeStreamed(_that.samples);case CanvasStrokeCancelledEvent() when strokeCancelled != null:
return strokeCancelled();case CanvasPointerMovedEvent() when pointerMoved != null:
return pointerMoved(_that.x,_that.y);case CanvasPointerLeftEvent() when pointerLeft != null:
return pointerLeft();case CanvasResumedEvent() when resumed != null:
return resumed();case CanvasPausedEvent() when paused != null:
return paused();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function( String noteId,  String sketchId)  sketchStarted,required TResult Function()  stylusSeen,required TResult Function( CanvasTool tool)  toolChanged,required TResult Function( String color)  colorChanged,required TResult Function( double width)  widthChanged,required TResult Function( bool on)  fingerDrawsChanged,required TResult Function( List<PenSample> samples)  strokeCommitted,required TResult Function( List<CanvasPoint> path,  double radius,  bool partial)  erased,required TResult Function( List<CanvasPoint> polygon)  lassoClosed,required TResult Function()  selectionCleared,required TResult Function()  selectionDeleted,required TResult Function( double dx,  double dy)  selectionMoved,required TResult Function( String title)  selectionMovedToNewCanvas,required TResult Function( double x,  double y,  String text)  textCardAdded,required TResult Function( String noteId,  double x,  double y)  noteCardAdded,required TResult Function( String id,  String text)  cardTextChanged,required TResult Function( String id,  CanvasRect rect)  cardMoved,required TResult Function( String id)  cardDeleted,required TResult Function( String from,  String to)  arrowAdded,required TResult Function()  undone,required TResult Function()  redone,required TResult Function()  saveRequested,required TResult Function()  messageDismissed,required TResult Function( bool active)  gestureChanged,required TResult Function()  pulsed,required TResult Function( List<PenSample> samples)  strokeStreamed,required TResult Function()  strokeCancelled,required TResult Function( double x,  double y)  pointerMoved,required TResult Function()  pointerLeft,required TResult Function()  resumed,required TResult Function()  paused,}) {final _that = this;
switch (_that) {
case CanvasStartedEvent():
return started(_that.noteId);case CanvasSketchStartedEvent():
return sketchStarted(_that.noteId,_that.sketchId);case CanvasStylusSeenEvent():
return stylusSeen();case CanvasToolChangedEvent():
return toolChanged(_that.tool);case CanvasColorChangedEvent():
return colorChanged(_that.color);case CanvasWidthChangedEvent():
return widthChanged(_that.width);case CanvasFingerDrawsChangedEvent():
return fingerDrawsChanged(_that.on);case CanvasStrokeCommittedEvent():
return strokeCommitted(_that.samples);case CanvasErasedEvent():
return erased(_that.path,_that.radius,_that.partial);case CanvasLassoClosedEvent():
return lassoClosed(_that.polygon);case CanvasSelectionClearedEvent():
return selectionCleared();case CanvasSelectionDeletedEvent():
return selectionDeleted();case CanvasSelectionMovedEvent():
return selectionMoved(_that.dx,_that.dy);case CanvasSelectionMovedToNewCanvasEvent():
return selectionMovedToNewCanvas(_that.title);case CanvasTextCardAddedEvent():
return textCardAdded(_that.x,_that.y,_that.text);case CanvasNoteCardAddedEvent():
return noteCardAdded(_that.noteId,_that.x,_that.y);case CanvasCardTextChangedEvent():
return cardTextChanged(_that.id,_that.text);case CanvasCardMovedEvent():
return cardMoved(_that.id,_that.rect);case CanvasCardDeletedEvent():
return cardDeleted(_that.id);case CanvasArrowAddedEvent():
return arrowAdded(_that.from,_that.to);case CanvasUndoneEvent():
return undone();case CanvasRedoneEvent():
return redone();case CanvasSaveRequestedEvent():
return saveRequested();case CanvasMessageDismissedEvent():
return messageDismissed();case CanvasGestureChangedEvent():
return gestureChanged(_that.active);case CanvasPulsedEvent():
return pulsed();case CanvasStrokeStreamedEvent():
return strokeStreamed(_that.samples);case CanvasStrokeCancelledEvent():
return strokeCancelled();case CanvasPointerMovedEvent():
return pointerMoved(_that.x,_that.y);case CanvasPointerLeftEvent():
return pointerLeft();case CanvasResumedEvent():
return resumed();case CanvasPausedEvent():
return paused();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function( String noteId,  String sketchId)?  sketchStarted,TResult? Function()?  stylusSeen,TResult? Function( CanvasTool tool)?  toolChanged,TResult? Function( String color)?  colorChanged,TResult? Function( double width)?  widthChanged,TResult? Function( bool on)?  fingerDrawsChanged,TResult? Function( List<PenSample> samples)?  strokeCommitted,TResult? Function( List<CanvasPoint> path,  double radius,  bool partial)?  erased,TResult? Function( List<CanvasPoint> polygon)?  lassoClosed,TResult? Function()?  selectionCleared,TResult? Function()?  selectionDeleted,TResult? Function( double dx,  double dy)?  selectionMoved,TResult? Function( String title)?  selectionMovedToNewCanvas,TResult? Function( double x,  double y,  String text)?  textCardAdded,TResult? Function( String noteId,  double x,  double y)?  noteCardAdded,TResult? Function( String id,  String text)?  cardTextChanged,TResult? Function( String id,  CanvasRect rect)?  cardMoved,TResult? Function( String id)?  cardDeleted,TResult? Function( String from,  String to)?  arrowAdded,TResult? Function()?  undone,TResult? Function()?  redone,TResult? Function()?  saveRequested,TResult? Function()?  messageDismissed,TResult? Function( bool active)?  gestureChanged,TResult? Function()?  pulsed,TResult? Function( List<PenSample> samples)?  strokeStreamed,TResult? Function()?  strokeCancelled,TResult? Function( double x,  double y)?  pointerMoved,TResult? Function()?  pointerLeft,TResult? Function()?  resumed,TResult? Function()?  paused,}) {final _that = this;
switch (_that) {
case CanvasStartedEvent() when started != null:
return started(_that.noteId);case CanvasSketchStartedEvent() when sketchStarted != null:
return sketchStarted(_that.noteId,_that.sketchId);case CanvasStylusSeenEvent() when stylusSeen != null:
return stylusSeen();case CanvasToolChangedEvent() when toolChanged != null:
return toolChanged(_that.tool);case CanvasColorChangedEvent() when colorChanged != null:
return colorChanged(_that.color);case CanvasWidthChangedEvent() when widthChanged != null:
return widthChanged(_that.width);case CanvasFingerDrawsChangedEvent() when fingerDrawsChanged != null:
return fingerDrawsChanged(_that.on);case CanvasStrokeCommittedEvent() when strokeCommitted != null:
return strokeCommitted(_that.samples);case CanvasErasedEvent() when erased != null:
return erased(_that.path,_that.radius,_that.partial);case CanvasLassoClosedEvent() when lassoClosed != null:
return lassoClosed(_that.polygon);case CanvasSelectionClearedEvent() when selectionCleared != null:
return selectionCleared();case CanvasSelectionDeletedEvent() when selectionDeleted != null:
return selectionDeleted();case CanvasSelectionMovedEvent() when selectionMoved != null:
return selectionMoved(_that.dx,_that.dy);case CanvasSelectionMovedToNewCanvasEvent() when selectionMovedToNewCanvas != null:
return selectionMovedToNewCanvas(_that.title);case CanvasTextCardAddedEvent() when textCardAdded != null:
return textCardAdded(_that.x,_that.y,_that.text);case CanvasNoteCardAddedEvent() when noteCardAdded != null:
return noteCardAdded(_that.noteId,_that.x,_that.y);case CanvasCardTextChangedEvent() when cardTextChanged != null:
return cardTextChanged(_that.id,_that.text);case CanvasCardMovedEvent() when cardMoved != null:
return cardMoved(_that.id,_that.rect);case CanvasCardDeletedEvent() when cardDeleted != null:
return cardDeleted(_that.id);case CanvasArrowAddedEvent() when arrowAdded != null:
return arrowAdded(_that.from,_that.to);case CanvasUndoneEvent() when undone != null:
return undone();case CanvasRedoneEvent() when redone != null:
return redone();case CanvasSaveRequestedEvent() when saveRequested != null:
return saveRequested();case CanvasMessageDismissedEvent() when messageDismissed != null:
return messageDismissed();case CanvasGestureChangedEvent() when gestureChanged != null:
return gestureChanged(_that.active);case CanvasPulsedEvent() when pulsed != null:
return pulsed();case CanvasStrokeStreamedEvent() when strokeStreamed != null:
return strokeStreamed(_that.samples);case CanvasStrokeCancelledEvent() when strokeCancelled != null:
return strokeCancelled();case CanvasPointerMovedEvent() when pointerMoved != null:
return pointerMoved(_that.x,_that.y);case CanvasPointerLeftEvent() when pointerLeft != null:
return pointerLeft();case CanvasResumedEvent() when resumed != null:
return resumed();case CanvasPausedEvent() when paused != null:
return paused();case _:
  return null;

}
}

}

/// @nodoc


class CanvasStartedEvent implements CanvasEvent {
  const CanvasStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasStartedEventCopyWith<CanvasStartedEvent> get copyWith => _$CanvasStartedEventCopyWithImpl<CanvasStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'CanvasEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $CanvasStartedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasStartedEventCopyWith(CanvasStartedEvent value, $Res Function(CanvasStartedEvent) _then) = _$CanvasStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$CanvasStartedEventCopyWithImpl<$Res>
    implements $CanvasStartedEventCopyWith<$Res> {
  _$CanvasStartedEventCopyWithImpl(this._self, this._then);

  final CanvasStartedEvent _self;
  final $Res Function(CanvasStartedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(CanvasStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasSketchStartedEvent implements CanvasEvent {
  const CanvasSketchStartedEvent(this.noteId, this.sketchId);
  

 final  String noteId;
 final  String sketchId;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasSketchStartedEventCopyWith<CanvasSketchStartedEvent> get copyWith => _$CanvasSketchStartedEventCopyWithImpl<CanvasSketchStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasSketchStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.sketchId, sketchId) || other.sketchId == sketchId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,sketchId);

@override
String toString() {
  return 'CanvasEvent.sketchStarted(noteId: $noteId, sketchId: $sketchId)';
}


}

/// @nodoc
abstract mixin class $CanvasSketchStartedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasSketchStartedEventCopyWith(CanvasSketchStartedEvent value, $Res Function(CanvasSketchStartedEvent) _then) = _$CanvasSketchStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, String sketchId
});




}
/// @nodoc
class _$CanvasSketchStartedEventCopyWithImpl<$Res>
    implements $CanvasSketchStartedEventCopyWith<$Res> {
  _$CanvasSketchStartedEventCopyWithImpl(this._self, this._then);

  final CanvasSketchStartedEvent _self;
  final $Res Function(CanvasSketchStartedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? sketchId = null,}) {
  return _then(CanvasSketchStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == sketchId ? _self.sketchId : sketchId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasStylusSeenEvent implements CanvasEvent {
  const CanvasStylusSeenEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasStylusSeenEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.stylusSeen()';
}


}




/// @nodoc


class CanvasToolChangedEvent implements CanvasEvent {
  const CanvasToolChangedEvent(this.tool);
  

 final  CanvasTool tool;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasToolChangedEventCopyWith<CanvasToolChangedEvent> get copyWith => _$CanvasToolChangedEventCopyWithImpl<CanvasToolChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasToolChangedEvent&&(identical(other.tool, tool) || other.tool == tool));
}


@override
int get hashCode => Object.hash(runtimeType,tool);

@override
String toString() {
  return 'CanvasEvent.toolChanged(tool: $tool)';
}


}

/// @nodoc
abstract mixin class $CanvasToolChangedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasToolChangedEventCopyWith(CanvasToolChangedEvent value, $Res Function(CanvasToolChangedEvent) _then) = _$CanvasToolChangedEventCopyWithImpl;
@useResult
$Res call({
 CanvasTool tool
});




}
/// @nodoc
class _$CanvasToolChangedEventCopyWithImpl<$Res>
    implements $CanvasToolChangedEventCopyWith<$Res> {
  _$CanvasToolChangedEventCopyWithImpl(this._self, this._then);

  final CanvasToolChangedEvent _self;
  final $Res Function(CanvasToolChangedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? tool = null,}) {
  return _then(CanvasToolChangedEvent(
null == tool ? _self.tool : tool // ignore: cast_nullable_to_non_nullable
as CanvasTool,
  ));
}


}

/// @nodoc


class CanvasColorChangedEvent implements CanvasEvent {
  const CanvasColorChangedEvent(this.color);
  

 final  String color;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasColorChangedEventCopyWith<CanvasColorChangedEvent> get copyWith => _$CanvasColorChangedEventCopyWithImpl<CanvasColorChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasColorChangedEvent&&(identical(other.color, color) || other.color == color));
}


@override
int get hashCode => Object.hash(runtimeType,color);

@override
String toString() {
  return 'CanvasEvent.colorChanged(color: $color)';
}


}

/// @nodoc
abstract mixin class $CanvasColorChangedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasColorChangedEventCopyWith(CanvasColorChangedEvent value, $Res Function(CanvasColorChangedEvent) _then) = _$CanvasColorChangedEventCopyWithImpl;
@useResult
$Res call({
 String color
});




}
/// @nodoc
class _$CanvasColorChangedEventCopyWithImpl<$Res>
    implements $CanvasColorChangedEventCopyWith<$Res> {
  _$CanvasColorChangedEventCopyWithImpl(this._self, this._then);

  final CanvasColorChangedEvent _self;
  final $Res Function(CanvasColorChangedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? color = null,}) {
  return _then(CanvasColorChangedEvent(
null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasWidthChangedEvent implements CanvasEvent {
  const CanvasWidthChangedEvent(this.width);
  

 final  double width;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasWidthChangedEventCopyWith<CanvasWidthChangedEvent> get copyWith => _$CanvasWidthChangedEventCopyWithImpl<CanvasWidthChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasWidthChangedEvent&&(identical(other.width, width) || other.width == width));
}


@override
int get hashCode => Object.hash(runtimeType,width);

@override
String toString() {
  return 'CanvasEvent.widthChanged(width: $width)';
}


}

/// @nodoc
abstract mixin class $CanvasWidthChangedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasWidthChangedEventCopyWith(CanvasWidthChangedEvent value, $Res Function(CanvasWidthChangedEvent) _then) = _$CanvasWidthChangedEventCopyWithImpl;
@useResult
$Res call({
 double width
});




}
/// @nodoc
class _$CanvasWidthChangedEventCopyWithImpl<$Res>
    implements $CanvasWidthChangedEventCopyWith<$Res> {
  _$CanvasWidthChangedEventCopyWithImpl(this._self, this._then);

  final CanvasWidthChangedEvent _self;
  final $Res Function(CanvasWidthChangedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? width = null,}) {
  return _then(CanvasWidthChangedEvent(
null == width ? _self.width : width // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class CanvasFingerDrawsChangedEvent implements CanvasEvent {
  const CanvasFingerDrawsChangedEvent({required this.on});
  

 final  bool on;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasFingerDrawsChangedEventCopyWith<CanvasFingerDrawsChangedEvent> get copyWith => _$CanvasFingerDrawsChangedEventCopyWithImpl<CanvasFingerDrawsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasFingerDrawsChangedEvent&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,on);

@override
String toString() {
  return 'CanvasEvent.fingerDrawsChanged(on: $on)';
}


}

/// @nodoc
abstract mixin class $CanvasFingerDrawsChangedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasFingerDrawsChangedEventCopyWith(CanvasFingerDrawsChangedEvent value, $Res Function(CanvasFingerDrawsChangedEvent) _then) = _$CanvasFingerDrawsChangedEventCopyWithImpl;
@useResult
$Res call({
 bool on
});




}
/// @nodoc
class _$CanvasFingerDrawsChangedEventCopyWithImpl<$Res>
    implements $CanvasFingerDrawsChangedEventCopyWith<$Res> {
  _$CanvasFingerDrawsChangedEventCopyWithImpl(this._self, this._then);

  final CanvasFingerDrawsChangedEvent _self;
  final $Res Function(CanvasFingerDrawsChangedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? on = null,}) {
  return _then(CanvasFingerDrawsChangedEvent(
on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class CanvasStrokeCommittedEvent implements CanvasEvent {
  const CanvasStrokeCommittedEvent(final  List<PenSample> samples): _samples = samples;
  

 final  List<PenSample> _samples;
 List<PenSample> get samples {
  if (_samples is EqualUnmodifiableListView) return _samples;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_samples);
}


/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasStrokeCommittedEventCopyWith<CanvasStrokeCommittedEvent> get copyWith => _$CanvasStrokeCommittedEventCopyWithImpl<CanvasStrokeCommittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasStrokeCommittedEvent&&const DeepCollectionEquality().equals(other._samples, _samples));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_samples));

@override
String toString() {
  return 'CanvasEvent.strokeCommitted(samples: $samples)';
}


}

/// @nodoc
abstract mixin class $CanvasStrokeCommittedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasStrokeCommittedEventCopyWith(CanvasStrokeCommittedEvent value, $Res Function(CanvasStrokeCommittedEvent) _then) = _$CanvasStrokeCommittedEventCopyWithImpl;
@useResult
$Res call({
 List<PenSample> samples
});




}
/// @nodoc
class _$CanvasStrokeCommittedEventCopyWithImpl<$Res>
    implements $CanvasStrokeCommittedEventCopyWith<$Res> {
  _$CanvasStrokeCommittedEventCopyWithImpl(this._self, this._then);

  final CanvasStrokeCommittedEvent _self;
  final $Res Function(CanvasStrokeCommittedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? samples = null,}) {
  return _then(CanvasStrokeCommittedEvent(
null == samples ? _self._samples : samples // ignore: cast_nullable_to_non_nullable
as List<PenSample>,
  ));
}


}

/// @nodoc


class CanvasErasedEvent implements CanvasEvent {
  const CanvasErasedEvent({required final  List<CanvasPoint> path, required this.radius, required this.partial}): _path = path;
  

 final  List<CanvasPoint> _path;
 List<CanvasPoint> get path {
  if (_path is EqualUnmodifiableListView) return _path;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_path);
}

 final  double radius;
 final  bool partial;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasErasedEventCopyWith<CanvasErasedEvent> get copyWith => _$CanvasErasedEventCopyWithImpl<CanvasErasedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasErasedEvent&&const DeepCollectionEquality().equals(other._path, _path)&&(identical(other.radius, radius) || other.radius == radius)&&(identical(other.partial, partial) || other.partial == partial));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_path),radius,partial);

@override
String toString() {
  return 'CanvasEvent.erased(path: $path, radius: $radius, partial: $partial)';
}


}

/// @nodoc
abstract mixin class $CanvasErasedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasErasedEventCopyWith(CanvasErasedEvent value, $Res Function(CanvasErasedEvent) _then) = _$CanvasErasedEventCopyWithImpl;
@useResult
$Res call({
 List<CanvasPoint> path, double radius, bool partial
});




}
/// @nodoc
class _$CanvasErasedEventCopyWithImpl<$Res>
    implements $CanvasErasedEventCopyWith<$Res> {
  _$CanvasErasedEventCopyWithImpl(this._self, this._then);

  final CanvasErasedEvent _self;
  final $Res Function(CanvasErasedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? path = null,Object? radius = null,Object? partial = null,}) {
  return _then(CanvasErasedEvent(
path: null == path ? _self._path : path // ignore: cast_nullable_to_non_nullable
as List<CanvasPoint>,radius: null == radius ? _self.radius : radius // ignore: cast_nullable_to_non_nullable
as double,partial: null == partial ? _self.partial : partial // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class CanvasLassoClosedEvent implements CanvasEvent {
  const CanvasLassoClosedEvent(final  List<CanvasPoint> polygon): _polygon = polygon;
  

 final  List<CanvasPoint> _polygon;
 List<CanvasPoint> get polygon {
  if (_polygon is EqualUnmodifiableListView) return _polygon;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_polygon);
}


/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasLassoClosedEventCopyWith<CanvasLassoClosedEvent> get copyWith => _$CanvasLassoClosedEventCopyWithImpl<CanvasLassoClosedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasLassoClosedEvent&&const DeepCollectionEquality().equals(other._polygon, _polygon));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_polygon));

@override
String toString() {
  return 'CanvasEvent.lassoClosed(polygon: $polygon)';
}


}

/// @nodoc
abstract mixin class $CanvasLassoClosedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasLassoClosedEventCopyWith(CanvasLassoClosedEvent value, $Res Function(CanvasLassoClosedEvent) _then) = _$CanvasLassoClosedEventCopyWithImpl;
@useResult
$Res call({
 List<CanvasPoint> polygon
});




}
/// @nodoc
class _$CanvasLassoClosedEventCopyWithImpl<$Res>
    implements $CanvasLassoClosedEventCopyWith<$Res> {
  _$CanvasLassoClosedEventCopyWithImpl(this._self, this._then);

  final CanvasLassoClosedEvent _self;
  final $Res Function(CanvasLassoClosedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? polygon = null,}) {
  return _then(CanvasLassoClosedEvent(
null == polygon ? _self._polygon : polygon // ignore: cast_nullable_to_non_nullable
as List<CanvasPoint>,
  ));
}


}

/// @nodoc


class CanvasSelectionClearedEvent implements CanvasEvent {
  const CanvasSelectionClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasSelectionClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.selectionCleared()';
}


}




/// @nodoc


class CanvasSelectionDeletedEvent implements CanvasEvent {
  const CanvasSelectionDeletedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasSelectionDeletedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.selectionDeleted()';
}


}




/// @nodoc


class CanvasSelectionMovedEvent implements CanvasEvent {
  const CanvasSelectionMovedEvent(this.dx, this.dy);
  

 final  double dx;
 final  double dy;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasSelectionMovedEventCopyWith<CanvasSelectionMovedEvent> get copyWith => _$CanvasSelectionMovedEventCopyWithImpl<CanvasSelectionMovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasSelectionMovedEvent&&(identical(other.dx, dx) || other.dx == dx)&&(identical(other.dy, dy) || other.dy == dy));
}


@override
int get hashCode => Object.hash(runtimeType,dx,dy);

@override
String toString() {
  return 'CanvasEvent.selectionMoved(dx: $dx, dy: $dy)';
}


}

/// @nodoc
abstract mixin class $CanvasSelectionMovedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasSelectionMovedEventCopyWith(CanvasSelectionMovedEvent value, $Res Function(CanvasSelectionMovedEvent) _then) = _$CanvasSelectionMovedEventCopyWithImpl;
@useResult
$Res call({
 double dx, double dy
});




}
/// @nodoc
class _$CanvasSelectionMovedEventCopyWithImpl<$Res>
    implements $CanvasSelectionMovedEventCopyWith<$Res> {
  _$CanvasSelectionMovedEventCopyWithImpl(this._self, this._then);

  final CanvasSelectionMovedEvent _self;
  final $Res Function(CanvasSelectionMovedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? dx = null,Object? dy = null,}) {
  return _then(CanvasSelectionMovedEvent(
null == dx ? _self.dx : dx // ignore: cast_nullable_to_non_nullable
as double,null == dy ? _self.dy : dy // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class CanvasSelectionMovedToNewCanvasEvent implements CanvasEvent {
  const CanvasSelectionMovedToNewCanvasEvent(this.title);
  

 final  String title;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasSelectionMovedToNewCanvasEventCopyWith<CanvasSelectionMovedToNewCanvasEvent> get copyWith => _$CanvasSelectionMovedToNewCanvasEventCopyWithImpl<CanvasSelectionMovedToNewCanvasEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasSelectionMovedToNewCanvasEvent&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'CanvasEvent.selectionMovedToNewCanvas(title: $title)';
}


}

/// @nodoc
abstract mixin class $CanvasSelectionMovedToNewCanvasEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasSelectionMovedToNewCanvasEventCopyWith(CanvasSelectionMovedToNewCanvasEvent value, $Res Function(CanvasSelectionMovedToNewCanvasEvent) _then) = _$CanvasSelectionMovedToNewCanvasEventCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$CanvasSelectionMovedToNewCanvasEventCopyWithImpl<$Res>
    implements $CanvasSelectionMovedToNewCanvasEventCopyWith<$Res> {
  _$CanvasSelectionMovedToNewCanvasEventCopyWithImpl(this._self, this._then);

  final CanvasSelectionMovedToNewCanvasEvent _self;
  final $Res Function(CanvasSelectionMovedToNewCanvasEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(CanvasSelectionMovedToNewCanvasEvent(
null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasTextCardAddedEvent implements CanvasEvent {
  const CanvasTextCardAddedEvent(this.x, this.y, this.text);
  

 final  double x;
 final  double y;
 final  String text;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasTextCardAddedEventCopyWith<CanvasTextCardAddedEvent> get copyWith => _$CanvasTextCardAddedEventCopyWithImpl<CanvasTextCardAddedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasTextCardAddedEvent&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,x,y,text);

@override
String toString() {
  return 'CanvasEvent.textCardAdded(x: $x, y: $y, text: $text)';
}


}

/// @nodoc
abstract mixin class $CanvasTextCardAddedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasTextCardAddedEventCopyWith(CanvasTextCardAddedEvent value, $Res Function(CanvasTextCardAddedEvent) _then) = _$CanvasTextCardAddedEventCopyWithImpl;
@useResult
$Res call({
 double x, double y, String text
});




}
/// @nodoc
class _$CanvasTextCardAddedEventCopyWithImpl<$Res>
    implements $CanvasTextCardAddedEventCopyWith<$Res> {
  _$CanvasTextCardAddedEventCopyWithImpl(this._self, this._then);

  final CanvasTextCardAddedEvent _self;
  final $Res Function(CanvasTextCardAddedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? x = null,Object? y = null,Object? text = null,}) {
  return _then(CanvasTextCardAddedEvent(
null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasNoteCardAddedEvent implements CanvasEvent {
  const CanvasNoteCardAddedEvent(this.noteId, this.x, this.y);
  

 final  String noteId;
 final  double x;
 final  double y;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasNoteCardAddedEventCopyWith<CanvasNoteCardAddedEvent> get copyWith => _$CanvasNoteCardAddedEventCopyWithImpl<CanvasNoteCardAddedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasNoteCardAddedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,x,y);

@override
String toString() {
  return 'CanvasEvent.noteCardAdded(noteId: $noteId, x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class $CanvasNoteCardAddedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasNoteCardAddedEventCopyWith(CanvasNoteCardAddedEvent value, $Res Function(CanvasNoteCardAddedEvent) _then) = _$CanvasNoteCardAddedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, double x, double y
});




}
/// @nodoc
class _$CanvasNoteCardAddedEventCopyWithImpl<$Res>
    implements $CanvasNoteCardAddedEventCopyWith<$Res> {
  _$CanvasNoteCardAddedEventCopyWithImpl(this._self, this._then);

  final CanvasNoteCardAddedEvent _self;
  final $Res Function(CanvasNoteCardAddedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? x = null,Object? y = null,}) {
  return _then(CanvasNoteCardAddedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class CanvasCardTextChangedEvent implements CanvasEvent {
  const CanvasCardTextChangedEvent(this.id, this.text);
  

 final  String id;
 final  String text;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasCardTextChangedEventCopyWith<CanvasCardTextChangedEvent> get copyWith => _$CanvasCardTextChangedEventCopyWithImpl<CanvasCardTextChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasCardTextChangedEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,id,text);

@override
String toString() {
  return 'CanvasEvent.cardTextChanged(id: $id, text: $text)';
}


}

/// @nodoc
abstract mixin class $CanvasCardTextChangedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasCardTextChangedEventCopyWith(CanvasCardTextChangedEvent value, $Res Function(CanvasCardTextChangedEvent) _then) = _$CanvasCardTextChangedEventCopyWithImpl;
@useResult
$Res call({
 String id, String text
});




}
/// @nodoc
class _$CanvasCardTextChangedEventCopyWithImpl<$Res>
    implements $CanvasCardTextChangedEventCopyWith<$Res> {
  _$CanvasCardTextChangedEventCopyWithImpl(this._self, this._then);

  final CanvasCardTextChangedEvent _self;
  final $Res Function(CanvasCardTextChangedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? text = null,}) {
  return _then(CanvasCardTextChangedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasCardMovedEvent implements CanvasEvent {
  const CanvasCardMovedEvent(this.id, this.rect);
  

 final  String id;
 final  CanvasRect rect;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasCardMovedEventCopyWith<CanvasCardMovedEvent> get copyWith => _$CanvasCardMovedEventCopyWithImpl<CanvasCardMovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasCardMovedEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.rect, rect) || other.rect == rect));
}


@override
int get hashCode => Object.hash(runtimeType,id,rect);

@override
String toString() {
  return 'CanvasEvent.cardMoved(id: $id, rect: $rect)';
}


}

/// @nodoc
abstract mixin class $CanvasCardMovedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasCardMovedEventCopyWith(CanvasCardMovedEvent value, $Res Function(CanvasCardMovedEvent) _then) = _$CanvasCardMovedEventCopyWithImpl;
@useResult
$Res call({
 String id, CanvasRect rect
});




}
/// @nodoc
class _$CanvasCardMovedEventCopyWithImpl<$Res>
    implements $CanvasCardMovedEventCopyWith<$Res> {
  _$CanvasCardMovedEventCopyWithImpl(this._self, this._then);

  final CanvasCardMovedEvent _self;
  final $Res Function(CanvasCardMovedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? rect = null,}) {
  return _then(CanvasCardMovedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,null == rect ? _self.rect : rect // ignore: cast_nullable_to_non_nullable
as CanvasRect,
  ));
}


}

/// @nodoc


class CanvasCardDeletedEvent implements CanvasEvent {
  const CanvasCardDeletedEvent(this.id);
  

 final  String id;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasCardDeletedEventCopyWith<CanvasCardDeletedEvent> get copyWith => _$CanvasCardDeletedEventCopyWithImpl<CanvasCardDeletedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasCardDeletedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'CanvasEvent.cardDeleted(id: $id)';
}


}

/// @nodoc
abstract mixin class $CanvasCardDeletedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasCardDeletedEventCopyWith(CanvasCardDeletedEvent value, $Res Function(CanvasCardDeletedEvent) _then) = _$CanvasCardDeletedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$CanvasCardDeletedEventCopyWithImpl<$Res>
    implements $CanvasCardDeletedEventCopyWith<$Res> {
  _$CanvasCardDeletedEventCopyWithImpl(this._self, this._then);

  final CanvasCardDeletedEvent _self;
  final $Res Function(CanvasCardDeletedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(CanvasCardDeletedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasArrowAddedEvent implements CanvasEvent {
  const CanvasArrowAddedEvent(this.from, this.to);
  

 final  String from;
 final  String to;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasArrowAddedEventCopyWith<CanvasArrowAddedEvent> get copyWith => _$CanvasArrowAddedEventCopyWithImpl<CanvasArrowAddedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasArrowAddedEvent&&(identical(other.from, from) || other.from == from)&&(identical(other.to, to) || other.to == to));
}


@override
int get hashCode => Object.hash(runtimeType,from,to);

@override
String toString() {
  return 'CanvasEvent.arrowAdded(from: $from, to: $to)';
}


}

/// @nodoc
abstract mixin class $CanvasArrowAddedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasArrowAddedEventCopyWith(CanvasArrowAddedEvent value, $Res Function(CanvasArrowAddedEvent) _then) = _$CanvasArrowAddedEventCopyWithImpl;
@useResult
$Res call({
 String from, String to
});




}
/// @nodoc
class _$CanvasArrowAddedEventCopyWithImpl<$Res>
    implements $CanvasArrowAddedEventCopyWith<$Res> {
  _$CanvasArrowAddedEventCopyWithImpl(this._self, this._then);

  final CanvasArrowAddedEvent _self;
  final $Res Function(CanvasArrowAddedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? from = null,Object? to = null,}) {
  return _then(CanvasArrowAddedEvent(
null == from ? _self.from : from // ignore: cast_nullable_to_non_nullable
as String,null == to ? _self.to : to // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CanvasUndoneEvent implements CanvasEvent {
  const CanvasUndoneEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasUndoneEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.undone()';
}


}




/// @nodoc


class CanvasRedoneEvent implements CanvasEvent {
  const CanvasRedoneEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasRedoneEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.redone()';
}


}




/// @nodoc


class CanvasSaveRequestedEvent implements CanvasEvent {
  const CanvasSaveRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasSaveRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.saveRequested()';
}


}




/// @nodoc


class CanvasMessageDismissedEvent implements CanvasEvent {
  const CanvasMessageDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasMessageDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.messageDismissed()';
}


}




/// @nodoc


class CanvasGestureChangedEvent implements CanvasEvent {
  const CanvasGestureChangedEvent({required this.active});
  

 final  bool active;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasGestureChangedEventCopyWith<CanvasGestureChangedEvent> get copyWith => _$CanvasGestureChangedEventCopyWithImpl<CanvasGestureChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasGestureChangedEvent&&(identical(other.active, active) || other.active == active));
}


@override
int get hashCode => Object.hash(runtimeType,active);

@override
String toString() {
  return 'CanvasEvent.gestureChanged(active: $active)';
}


}

/// @nodoc
abstract mixin class $CanvasGestureChangedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasGestureChangedEventCopyWith(CanvasGestureChangedEvent value, $Res Function(CanvasGestureChangedEvent) _then) = _$CanvasGestureChangedEventCopyWithImpl;
@useResult
$Res call({
 bool active
});




}
/// @nodoc
class _$CanvasGestureChangedEventCopyWithImpl<$Res>
    implements $CanvasGestureChangedEventCopyWith<$Res> {
  _$CanvasGestureChangedEventCopyWithImpl(this._self, this._then);

  final CanvasGestureChangedEvent _self;
  final $Res Function(CanvasGestureChangedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? active = null,}) {
  return _then(CanvasGestureChangedEvent(
active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class CanvasPulsedEvent implements CanvasEvent {
  const CanvasPulsedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasPulsedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.pulsed()';
}


}




/// @nodoc


class CanvasStrokeStreamedEvent implements CanvasEvent {
  const CanvasStrokeStreamedEvent(final  List<PenSample> samples): _samples = samples;
  

 final  List<PenSample> _samples;
 List<PenSample> get samples {
  if (_samples is EqualUnmodifiableListView) return _samples;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_samples);
}


/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasStrokeStreamedEventCopyWith<CanvasStrokeStreamedEvent> get copyWith => _$CanvasStrokeStreamedEventCopyWithImpl<CanvasStrokeStreamedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasStrokeStreamedEvent&&const DeepCollectionEquality().equals(other._samples, _samples));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_samples));

@override
String toString() {
  return 'CanvasEvent.strokeStreamed(samples: $samples)';
}


}

/// @nodoc
abstract mixin class $CanvasStrokeStreamedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasStrokeStreamedEventCopyWith(CanvasStrokeStreamedEvent value, $Res Function(CanvasStrokeStreamedEvent) _then) = _$CanvasStrokeStreamedEventCopyWithImpl;
@useResult
$Res call({
 List<PenSample> samples
});




}
/// @nodoc
class _$CanvasStrokeStreamedEventCopyWithImpl<$Res>
    implements $CanvasStrokeStreamedEventCopyWith<$Res> {
  _$CanvasStrokeStreamedEventCopyWithImpl(this._self, this._then);

  final CanvasStrokeStreamedEvent _self;
  final $Res Function(CanvasStrokeStreamedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? samples = null,}) {
  return _then(CanvasStrokeStreamedEvent(
null == samples ? _self._samples : samples // ignore: cast_nullable_to_non_nullable
as List<PenSample>,
  ));
}


}

/// @nodoc


class CanvasStrokeCancelledEvent implements CanvasEvent {
  const CanvasStrokeCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasStrokeCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.strokeCancelled()';
}


}




/// @nodoc


class CanvasPointerMovedEvent implements CanvasEvent {
  const CanvasPointerMovedEvent(this.x, this.y);
  

 final  double x;
 final  double y;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasPointerMovedEventCopyWith<CanvasPointerMovedEvent> get copyWith => _$CanvasPointerMovedEventCopyWithImpl<CanvasPointerMovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasPointerMovedEvent&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}


@override
int get hashCode => Object.hash(runtimeType,x,y);

@override
String toString() {
  return 'CanvasEvent.pointerMoved(x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class $CanvasPointerMovedEventCopyWith<$Res> implements $CanvasEventCopyWith<$Res> {
  factory $CanvasPointerMovedEventCopyWith(CanvasPointerMovedEvent value, $Res Function(CanvasPointerMovedEvent) _then) = _$CanvasPointerMovedEventCopyWithImpl;
@useResult
$Res call({
 double x, double y
});




}
/// @nodoc
class _$CanvasPointerMovedEventCopyWithImpl<$Res>
    implements $CanvasPointerMovedEventCopyWith<$Res> {
  _$CanvasPointerMovedEventCopyWithImpl(this._self, this._then);

  final CanvasPointerMovedEvent _self;
  final $Res Function(CanvasPointerMovedEvent) _then;

/// Create a copy of CanvasEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? x = null,Object? y = null,}) {
  return _then(CanvasPointerMovedEvent(
null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class CanvasPointerLeftEvent implements CanvasEvent {
  const CanvasPointerLeftEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasPointerLeftEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.pointerLeft()';
}


}




/// @nodoc


class CanvasResumedEvent implements CanvasEvent {
  const CanvasResumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasResumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.resumed()';
}


}




/// @nodoc


class CanvasPausedEvent implements CanvasEvent {
  const CanvasPausedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasPausedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CanvasEvent.paused()';
}


}




/// @nodoc
mixin _$CanvasState {

 CanvasStatus get status; CanvasView? get view; CanvasTool get tool;/// The pen colour, `#rrggbb`.
 String get color;/// The pen width, in canvas points.
 double get width;/// Whether a finger draws (until a pen is seen); else it pans.
 bool get fingerDraws;/// Whether a pen has touched this canvas.
 bool get stylusSeen;/// The strokes a lasso holds.
 List<StrokeKey> get selection;/// Counts every change of the view, so a stroke being settled can tell
/// that the library has taken it.
 int get edit; CanvasProblem? get problem;/// Where the others' pointers are on a shared canvas.
 List<CanvasPointerRow> get pointers;/// Counts the strokes someone else erased under the pen: the surface
/// drops the stroke it is drawing when this changes.
 int get erasedUnderPen;
/// Create a copy of CanvasState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CanvasStateCopyWith<CanvasState> get copyWith => _$CanvasStateCopyWithImpl<CanvasState>(this as CanvasState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CanvasState&&(identical(other.status, status) || other.status == status)&&(identical(other.view, view) || other.view == view)&&(identical(other.tool, tool) || other.tool == tool)&&(identical(other.color, color) || other.color == color)&&(identical(other.width, width) || other.width == width)&&(identical(other.fingerDraws, fingerDraws) || other.fingerDraws == fingerDraws)&&(identical(other.stylusSeen, stylusSeen) || other.stylusSeen == stylusSeen)&&const DeepCollectionEquality().equals(other.selection, selection)&&(identical(other.edit, edit) || other.edit == edit)&&(identical(other.problem, problem) || other.problem == problem)&&const DeepCollectionEquality().equals(other.pointers, pointers)&&(identical(other.erasedUnderPen, erasedUnderPen) || other.erasedUnderPen == erasedUnderPen));
}


@override
int get hashCode => Object.hash(runtimeType,status,view,tool,color,width,fingerDraws,stylusSeen,const DeepCollectionEquality().hash(selection),edit,problem,const DeepCollectionEquality().hash(pointers),erasedUnderPen);

@override
String toString() {
  return 'CanvasState(status: $status, view: $view, tool: $tool, color: $color, width: $width, fingerDraws: $fingerDraws, stylusSeen: $stylusSeen, selection: $selection, edit: $edit, problem: $problem, pointers: $pointers, erasedUnderPen: $erasedUnderPen)';
}


}

/// @nodoc
abstract mixin class $CanvasStateCopyWith<$Res>  {
  factory $CanvasStateCopyWith(CanvasState value, $Res Function(CanvasState) _then) = _$CanvasStateCopyWithImpl;
@useResult
$Res call({
 CanvasStatus status, CanvasView? view, CanvasTool tool, String color, double width, bool fingerDraws, bool stylusSeen, List<StrokeKey> selection, int edit, CanvasProblem? problem, List<CanvasPointerRow> pointers, int erasedUnderPen
});




}
/// @nodoc
class _$CanvasStateCopyWithImpl<$Res>
    implements $CanvasStateCopyWith<$Res> {
  _$CanvasStateCopyWithImpl(this._self, this._then);

  final CanvasState _self;
  final $Res Function(CanvasState) _then;

/// Create a copy of CanvasState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? view = freezed,Object? tool = null,Object? color = null,Object? width = null,Object? fingerDraws = null,Object? stylusSeen = null,Object? selection = null,Object? edit = null,Object? problem = freezed,Object? pointers = null,Object? erasedUnderPen = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CanvasStatus,view: freezed == view ? _self.view : view // ignore: cast_nullable_to_non_nullable
as CanvasView?,tool: null == tool ? _self.tool : tool // ignore: cast_nullable_to_non_nullable
as CanvasTool,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,width: null == width ? _self.width : width // ignore: cast_nullable_to_non_nullable
as double,fingerDraws: null == fingerDraws ? _self.fingerDraws : fingerDraws // ignore: cast_nullable_to_non_nullable
as bool,stylusSeen: null == stylusSeen ? _self.stylusSeen : stylusSeen // ignore: cast_nullable_to_non_nullable
as bool,selection: null == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as List<StrokeKey>,edit: null == edit ? _self.edit : edit // ignore: cast_nullable_to_non_nullable
as int,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as CanvasProblem?,pointers: null == pointers ? _self.pointers : pointers // ignore: cast_nullable_to_non_nullable
as List<CanvasPointerRow>,erasedUnderPen: null == erasedUnderPen ? _self.erasedUnderPen : erasedUnderPen // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [CanvasState].
extension CanvasStatePatterns on CanvasState {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CanvasState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CanvasState() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CanvasState value)  $default,){
final _that = this;
switch (_that) {
case _CanvasState():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CanvasState value)?  $default,){
final _that = this;
switch (_that) {
case _CanvasState() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( CanvasStatus status,  CanvasView? view,  CanvasTool tool,  String color,  double width,  bool fingerDraws,  bool stylusSeen,  List<StrokeKey> selection,  int edit,  CanvasProblem? problem,  List<CanvasPointerRow> pointers,  int erasedUnderPen)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CanvasState() when $default != null:
return $default(_that.status,_that.view,_that.tool,_that.color,_that.width,_that.fingerDraws,_that.stylusSeen,_that.selection,_that.edit,_that.problem,_that.pointers,_that.erasedUnderPen);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( CanvasStatus status,  CanvasView? view,  CanvasTool tool,  String color,  double width,  bool fingerDraws,  bool stylusSeen,  List<StrokeKey> selection,  int edit,  CanvasProblem? problem,  List<CanvasPointerRow> pointers,  int erasedUnderPen)  $default,) {final _that = this;
switch (_that) {
case _CanvasState():
return $default(_that.status,_that.view,_that.tool,_that.color,_that.width,_that.fingerDraws,_that.stylusSeen,_that.selection,_that.edit,_that.problem,_that.pointers,_that.erasedUnderPen);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( CanvasStatus status,  CanvasView? view,  CanvasTool tool,  String color,  double width,  bool fingerDraws,  bool stylusSeen,  List<StrokeKey> selection,  int edit,  CanvasProblem? problem,  List<CanvasPointerRow> pointers,  int erasedUnderPen)?  $default,) {final _that = this;
switch (_that) {
case _CanvasState() when $default != null:
return $default(_that.status,_that.view,_that.tool,_that.color,_that.width,_that.fingerDraws,_that.stylusSeen,_that.selection,_that.edit,_that.problem,_that.pointers,_that.erasedUnderPen);case _:
  return null;

}
}

}

/// @nodoc


class _CanvasState extends CanvasState {
  const _CanvasState({this.status = CanvasStatus.loading, this.view, this.tool = CanvasTool.pen, this.color = '#1e1e1e', this.width = 3.0, this.fingerDraws = true, this.stylusSeen = false, final  List<StrokeKey> selection = const <StrokeKey>[], this.edit = 0, this.problem, final  List<CanvasPointerRow> pointers = const <CanvasPointerRow>[], this.erasedUnderPen = 0}): _selection = selection,_pointers = pointers,super._();
  

@override@JsonKey() final  CanvasStatus status;
@override final  CanvasView? view;
@override@JsonKey() final  CanvasTool tool;
/// The pen colour, `#rrggbb`.
@override@JsonKey() final  String color;
/// The pen width, in canvas points.
@override@JsonKey() final  double width;
/// Whether a finger draws (until a pen is seen); else it pans.
@override@JsonKey() final  bool fingerDraws;
/// Whether a pen has touched this canvas.
@override@JsonKey() final  bool stylusSeen;
/// The strokes a lasso holds.
 final  List<StrokeKey> _selection;
/// The strokes a lasso holds.
@override@JsonKey() List<StrokeKey> get selection {
  if (_selection is EqualUnmodifiableListView) return _selection;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_selection);
}

/// Counts every change of the view, so a stroke being settled can tell
/// that the library has taken it.
@override@JsonKey() final  int edit;
@override final  CanvasProblem? problem;
/// Where the others' pointers are on a shared canvas.
 final  List<CanvasPointerRow> _pointers;
/// Where the others' pointers are on a shared canvas.
@override@JsonKey() List<CanvasPointerRow> get pointers {
  if (_pointers is EqualUnmodifiableListView) return _pointers;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_pointers);
}

/// Counts the strokes someone else erased under the pen: the surface
/// drops the stroke it is drawing when this changes.
@override@JsonKey() final  int erasedUnderPen;

/// Create a copy of CanvasState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CanvasStateCopyWith<_CanvasState> get copyWith => __$CanvasStateCopyWithImpl<_CanvasState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CanvasState&&(identical(other.status, status) || other.status == status)&&(identical(other.view, view) || other.view == view)&&(identical(other.tool, tool) || other.tool == tool)&&(identical(other.color, color) || other.color == color)&&(identical(other.width, width) || other.width == width)&&(identical(other.fingerDraws, fingerDraws) || other.fingerDraws == fingerDraws)&&(identical(other.stylusSeen, stylusSeen) || other.stylusSeen == stylusSeen)&&const DeepCollectionEquality().equals(other._selection, _selection)&&(identical(other.edit, edit) || other.edit == edit)&&(identical(other.problem, problem) || other.problem == problem)&&const DeepCollectionEquality().equals(other._pointers, _pointers)&&(identical(other.erasedUnderPen, erasedUnderPen) || other.erasedUnderPen == erasedUnderPen));
}


@override
int get hashCode => Object.hash(runtimeType,status,view,tool,color,width,fingerDraws,stylusSeen,const DeepCollectionEquality().hash(_selection),edit,problem,const DeepCollectionEquality().hash(_pointers),erasedUnderPen);

@override
String toString() {
  return 'CanvasState(status: $status, view: $view, tool: $tool, color: $color, width: $width, fingerDraws: $fingerDraws, stylusSeen: $stylusSeen, selection: $selection, edit: $edit, problem: $problem, pointers: $pointers, erasedUnderPen: $erasedUnderPen)';
}


}

/// @nodoc
abstract mixin class _$CanvasStateCopyWith<$Res> implements $CanvasStateCopyWith<$Res> {
  factory _$CanvasStateCopyWith(_CanvasState value, $Res Function(_CanvasState) _then) = __$CanvasStateCopyWithImpl;
@override @useResult
$Res call({
 CanvasStatus status, CanvasView? view, CanvasTool tool, String color, double width, bool fingerDraws, bool stylusSeen, List<StrokeKey> selection, int edit, CanvasProblem? problem, List<CanvasPointerRow> pointers, int erasedUnderPen
});




}
/// @nodoc
class __$CanvasStateCopyWithImpl<$Res>
    implements _$CanvasStateCopyWith<$Res> {
  __$CanvasStateCopyWithImpl(this._self, this._then);

  final _CanvasState _self;
  final $Res Function(_CanvasState) _then;

/// Create a copy of CanvasState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? view = freezed,Object? tool = null,Object? color = null,Object? width = null,Object? fingerDraws = null,Object? stylusSeen = null,Object? selection = null,Object? edit = null,Object? problem = freezed,Object? pointers = null,Object? erasedUnderPen = null,}) {
  return _then(_CanvasState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CanvasStatus,view: freezed == view ? _self.view : view // ignore: cast_nullable_to_non_nullable
as CanvasView?,tool: null == tool ? _self.tool : tool // ignore: cast_nullable_to_non_nullable
as CanvasTool,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,width: null == width ? _self.width : width // ignore: cast_nullable_to_non_nullable
as double,fingerDraws: null == fingerDraws ? _self.fingerDraws : fingerDraws // ignore: cast_nullable_to_non_nullable
as bool,stylusSeen: null == stylusSeen ? _self.stylusSeen : stylusSeen // ignore: cast_nullable_to_non_nullable
as bool,selection: null == selection ? _self._selection : selection // ignore: cast_nullable_to_non_nullable
as List<StrokeKey>,edit: null == edit ? _self.edit : edit // ignore: cast_nullable_to_non_nullable
as int,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as CanvasProblem?,pointers: null == pointers ? _self._pointers : pointers // ignore: cast_nullable_to_non_nullable
as List<CanvasPointerRow>,erasedUnderPen: null == erasedUnderPen ? _self.erasedUnderPen : erasedUnderPen // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
