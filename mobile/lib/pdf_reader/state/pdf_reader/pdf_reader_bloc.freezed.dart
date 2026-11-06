// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'pdf_reader_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PdfReaderEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent()';
}


}

/// @nodoc
class $PdfReaderEventCopyWith<$Res>  {
$PdfReaderEventCopyWith(PdfReaderEvent _, $Res Function(PdfReaderEvent) __);
}


/// Adds pattern-matching-related methods to [PdfReaderEvent].
extension PdfReaderEventPatterns on PdfReaderEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PdfReaderStartedEvent value)?  started,TResult Function( PdfReaderPasswordSubmittedEvent value)?  passwordSubmitted,TResult Function( PdfReaderRetriedEvent value)?  retried,TResult Function( PdfReaderZoomChangedEvent value)?  zoomChanged,TResult Function( PdfReaderInvertChangedEvent value)?  invertChanged,TResult Function( PdfReaderDarkChangedEvent value)?  darkChanged,TResult Function( PdfReaderPageChangedEvent value)?  pageChanged,TResult Function( PdfReaderJumpToEvent value)?  jumpTo,TResult Function( PdfReaderWordSelectedEvent value)?  wordSelected,TResult Function( PdfReaderLineSelectedEvent value)?  lineSelected,TResult Function( PdfReaderSelectionExtendedEvent value)?  selectionExtended,TResult Function( PdfReaderSelectionClearedEvent value)?  selectionCleared,TResult Function( PdfReaderSearchChangedEvent value)?  searchChanged,TResult Function( PdfReaderSearchRunEvent value)?  searchRun,TResult Function( PdfReaderSearchNextEvent value)?  searchNext,TResult Function( PdfReaderSearchPreviousEvent value)?  searchPrevious,TResult Function( PdfReaderSearchClosedEvent value)?  searchClosed,TResult Function( PdfReaderHighlightRequestedEvent value)?  highlightRequested,TResult Function( PdfReaderInkDrawingChangedEvent value)?  inkDrawingChanged,TResult Function( PdfReaderInkColorChangedEvent value)?  inkColorChanged,TResult Function( PdfReaderInkWidthChangedEvent value)?  inkWidthChanged,TResult Function( PdfReaderInkStrokeRequestedEvent value)?  inkStrokeRequested,TResult Function( PdfReaderHighlightFocusedEvent value)?  highlightFocused,TResult Function( PdfReaderNoticeDismissedEvent value)?  noticeDismissed,TResult Function( PdfReaderLockChangedEvent value)?  lockChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PdfReaderStartedEvent() when started != null:
return started(_that);case PdfReaderPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case PdfReaderRetriedEvent() when retried != null:
return retried(_that);case PdfReaderZoomChangedEvent() when zoomChanged != null:
return zoomChanged(_that);case PdfReaderInvertChangedEvent() when invertChanged != null:
return invertChanged(_that);case PdfReaderDarkChangedEvent() when darkChanged != null:
return darkChanged(_that);case PdfReaderPageChangedEvent() when pageChanged != null:
return pageChanged(_that);case PdfReaderJumpToEvent() when jumpTo != null:
return jumpTo(_that);case PdfReaderWordSelectedEvent() when wordSelected != null:
return wordSelected(_that);case PdfReaderLineSelectedEvent() when lineSelected != null:
return lineSelected(_that);case PdfReaderSelectionExtendedEvent() when selectionExtended != null:
return selectionExtended(_that);case PdfReaderSelectionClearedEvent() when selectionCleared != null:
return selectionCleared(_that);case PdfReaderSearchChangedEvent() when searchChanged != null:
return searchChanged(_that);case PdfReaderSearchRunEvent() when searchRun != null:
return searchRun(_that);case PdfReaderSearchNextEvent() when searchNext != null:
return searchNext(_that);case PdfReaderSearchPreviousEvent() when searchPrevious != null:
return searchPrevious(_that);case PdfReaderSearchClosedEvent() when searchClosed != null:
return searchClosed(_that);case PdfReaderHighlightRequestedEvent() when highlightRequested != null:
return highlightRequested(_that);case PdfReaderInkDrawingChangedEvent() when inkDrawingChanged != null:
return inkDrawingChanged(_that);case PdfReaderInkColorChangedEvent() when inkColorChanged != null:
return inkColorChanged(_that);case PdfReaderInkWidthChangedEvent() when inkWidthChanged != null:
return inkWidthChanged(_that);case PdfReaderInkStrokeRequestedEvent() when inkStrokeRequested != null:
return inkStrokeRequested(_that);case PdfReaderHighlightFocusedEvent() when highlightFocused != null:
return highlightFocused(_that);case PdfReaderNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case PdfReaderLockChangedEvent() when lockChanged != null:
return lockChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PdfReaderStartedEvent value)  started,required TResult Function( PdfReaderPasswordSubmittedEvent value)  passwordSubmitted,required TResult Function( PdfReaderRetriedEvent value)  retried,required TResult Function( PdfReaderZoomChangedEvent value)  zoomChanged,required TResult Function( PdfReaderInvertChangedEvent value)  invertChanged,required TResult Function( PdfReaderDarkChangedEvent value)  darkChanged,required TResult Function( PdfReaderPageChangedEvent value)  pageChanged,required TResult Function( PdfReaderJumpToEvent value)  jumpTo,required TResult Function( PdfReaderWordSelectedEvent value)  wordSelected,required TResult Function( PdfReaderLineSelectedEvent value)  lineSelected,required TResult Function( PdfReaderSelectionExtendedEvent value)  selectionExtended,required TResult Function( PdfReaderSelectionClearedEvent value)  selectionCleared,required TResult Function( PdfReaderSearchChangedEvent value)  searchChanged,required TResult Function( PdfReaderSearchRunEvent value)  searchRun,required TResult Function( PdfReaderSearchNextEvent value)  searchNext,required TResult Function( PdfReaderSearchPreviousEvent value)  searchPrevious,required TResult Function( PdfReaderSearchClosedEvent value)  searchClosed,required TResult Function( PdfReaderHighlightRequestedEvent value)  highlightRequested,required TResult Function( PdfReaderInkDrawingChangedEvent value)  inkDrawingChanged,required TResult Function( PdfReaderInkColorChangedEvent value)  inkColorChanged,required TResult Function( PdfReaderInkWidthChangedEvent value)  inkWidthChanged,required TResult Function( PdfReaderInkStrokeRequestedEvent value)  inkStrokeRequested,required TResult Function( PdfReaderHighlightFocusedEvent value)  highlightFocused,required TResult Function( PdfReaderNoticeDismissedEvent value)  noticeDismissed,required TResult Function( PdfReaderLockChangedEvent value)  lockChanged,}){
final _that = this;
switch (_that) {
case PdfReaderStartedEvent():
return started(_that);case PdfReaderPasswordSubmittedEvent():
return passwordSubmitted(_that);case PdfReaderRetriedEvent():
return retried(_that);case PdfReaderZoomChangedEvent():
return zoomChanged(_that);case PdfReaderInvertChangedEvent():
return invertChanged(_that);case PdfReaderDarkChangedEvent():
return darkChanged(_that);case PdfReaderPageChangedEvent():
return pageChanged(_that);case PdfReaderJumpToEvent():
return jumpTo(_that);case PdfReaderWordSelectedEvent():
return wordSelected(_that);case PdfReaderLineSelectedEvent():
return lineSelected(_that);case PdfReaderSelectionExtendedEvent():
return selectionExtended(_that);case PdfReaderSelectionClearedEvent():
return selectionCleared(_that);case PdfReaderSearchChangedEvent():
return searchChanged(_that);case PdfReaderSearchRunEvent():
return searchRun(_that);case PdfReaderSearchNextEvent():
return searchNext(_that);case PdfReaderSearchPreviousEvent():
return searchPrevious(_that);case PdfReaderSearchClosedEvent():
return searchClosed(_that);case PdfReaderHighlightRequestedEvent():
return highlightRequested(_that);case PdfReaderInkDrawingChangedEvent():
return inkDrawingChanged(_that);case PdfReaderInkColorChangedEvent():
return inkColorChanged(_that);case PdfReaderInkWidthChangedEvent():
return inkWidthChanged(_that);case PdfReaderInkStrokeRequestedEvent():
return inkStrokeRequested(_that);case PdfReaderHighlightFocusedEvent():
return highlightFocused(_that);case PdfReaderNoticeDismissedEvent():
return noticeDismissed(_that);case PdfReaderLockChangedEvent():
return lockChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PdfReaderStartedEvent value)?  started,TResult? Function( PdfReaderPasswordSubmittedEvent value)?  passwordSubmitted,TResult? Function( PdfReaderRetriedEvent value)?  retried,TResult? Function( PdfReaderZoomChangedEvent value)?  zoomChanged,TResult? Function( PdfReaderInvertChangedEvent value)?  invertChanged,TResult? Function( PdfReaderDarkChangedEvent value)?  darkChanged,TResult? Function( PdfReaderPageChangedEvent value)?  pageChanged,TResult? Function( PdfReaderJumpToEvent value)?  jumpTo,TResult? Function( PdfReaderWordSelectedEvent value)?  wordSelected,TResult? Function( PdfReaderLineSelectedEvent value)?  lineSelected,TResult? Function( PdfReaderSelectionExtendedEvent value)?  selectionExtended,TResult? Function( PdfReaderSelectionClearedEvent value)?  selectionCleared,TResult? Function( PdfReaderSearchChangedEvent value)?  searchChanged,TResult? Function( PdfReaderSearchRunEvent value)?  searchRun,TResult? Function( PdfReaderSearchNextEvent value)?  searchNext,TResult? Function( PdfReaderSearchPreviousEvent value)?  searchPrevious,TResult? Function( PdfReaderSearchClosedEvent value)?  searchClosed,TResult? Function( PdfReaderHighlightRequestedEvent value)?  highlightRequested,TResult? Function( PdfReaderInkDrawingChangedEvent value)?  inkDrawingChanged,TResult? Function( PdfReaderInkColorChangedEvent value)?  inkColorChanged,TResult? Function( PdfReaderInkWidthChangedEvent value)?  inkWidthChanged,TResult? Function( PdfReaderInkStrokeRequestedEvent value)?  inkStrokeRequested,TResult? Function( PdfReaderHighlightFocusedEvent value)?  highlightFocused,TResult? Function( PdfReaderNoticeDismissedEvent value)?  noticeDismissed,TResult? Function( PdfReaderLockChangedEvent value)?  lockChanged,}){
final _that = this;
switch (_that) {
case PdfReaderStartedEvent() when started != null:
return started(_that);case PdfReaderPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case PdfReaderRetriedEvent() when retried != null:
return retried(_that);case PdfReaderZoomChangedEvent() when zoomChanged != null:
return zoomChanged(_that);case PdfReaderInvertChangedEvent() when invertChanged != null:
return invertChanged(_that);case PdfReaderDarkChangedEvent() when darkChanged != null:
return darkChanged(_that);case PdfReaderPageChangedEvent() when pageChanged != null:
return pageChanged(_that);case PdfReaderJumpToEvent() when jumpTo != null:
return jumpTo(_that);case PdfReaderWordSelectedEvent() when wordSelected != null:
return wordSelected(_that);case PdfReaderLineSelectedEvent() when lineSelected != null:
return lineSelected(_that);case PdfReaderSelectionExtendedEvent() when selectionExtended != null:
return selectionExtended(_that);case PdfReaderSelectionClearedEvent() when selectionCleared != null:
return selectionCleared(_that);case PdfReaderSearchChangedEvent() when searchChanged != null:
return searchChanged(_that);case PdfReaderSearchRunEvent() when searchRun != null:
return searchRun(_that);case PdfReaderSearchNextEvent() when searchNext != null:
return searchNext(_that);case PdfReaderSearchPreviousEvent() when searchPrevious != null:
return searchPrevious(_that);case PdfReaderSearchClosedEvent() when searchClosed != null:
return searchClosed(_that);case PdfReaderHighlightRequestedEvent() when highlightRequested != null:
return highlightRequested(_that);case PdfReaderInkDrawingChangedEvent() when inkDrawingChanged != null:
return inkDrawingChanged(_that);case PdfReaderInkColorChangedEvent() when inkColorChanged != null:
return inkColorChanged(_that);case PdfReaderInkWidthChangedEvent() when inkWidthChanged != null:
return inkWidthChanged(_that);case PdfReaderInkStrokeRequestedEvent() when inkStrokeRequested != null:
return inkStrokeRequested(_that);case PdfReaderHighlightFocusedEvent() when highlightFocused != null:
return highlightFocused(_that);case PdfReaderNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case PdfReaderLockChangedEvent() when lockChanged != null:
return lockChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId,  String file,  String name,  int? page,  String? highlight,  bool dark)?  started,TResult Function( String password)?  passwordSubmitted,TResult Function()?  retried,TResult Function( PdfZoom zoom)?  zoomChanged,TResult Function( bool invert)?  invertChanged,TResult Function( bool dark)?  darkChanged,TResult Function( int page)?  pageChanged,TResult Function( int page)?  jumpTo,TResult Function( int page,  double x,  double y)?  wordSelected,TResult Function( int page,  double x,  double y)?  lineSelected,TResult Function( int page,  double x,  double y,  bool fromStart)?  selectionExtended,TResult Function()?  selectionCleared,TResult Function( String query)?  searchChanged,TResult Function( String query)?  searchRun,TResult Function()?  searchNext,TResult Function()?  searchPrevious,TResult Function()?  searchClosed,TResult Function( String color,  bool quote)?  highlightRequested,TResult Function( bool enabled)?  inkDrawingChanged,TResult Function( String color)?  inkColorChanged,TResult Function( double width)?  inkWidthChanged,TResult Function( int page,  PdfInkStroke stroke)?  inkStrokeRequested,TResult Function( String? id)?  highlightFocused,TResult Function()?  noticeDismissed,TResult Function()?  lockChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PdfReaderStartedEvent() when started != null:
return started(_that.noteId,_that.file,_that.name,_that.page,_that.highlight,_that.dark);case PdfReaderPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.password);case PdfReaderRetriedEvent() when retried != null:
return retried();case PdfReaderZoomChangedEvent() when zoomChanged != null:
return zoomChanged(_that.zoom);case PdfReaderInvertChangedEvent() when invertChanged != null:
return invertChanged(_that.invert);case PdfReaderDarkChangedEvent() when darkChanged != null:
return darkChanged(_that.dark);case PdfReaderPageChangedEvent() when pageChanged != null:
return pageChanged(_that.page);case PdfReaderJumpToEvent() when jumpTo != null:
return jumpTo(_that.page);case PdfReaderWordSelectedEvent() when wordSelected != null:
return wordSelected(_that.page,_that.x,_that.y);case PdfReaderLineSelectedEvent() when lineSelected != null:
return lineSelected(_that.page,_that.x,_that.y);case PdfReaderSelectionExtendedEvent() when selectionExtended != null:
return selectionExtended(_that.page,_that.x,_that.y,_that.fromStart);case PdfReaderSelectionClearedEvent() when selectionCleared != null:
return selectionCleared();case PdfReaderSearchChangedEvent() when searchChanged != null:
return searchChanged(_that.query);case PdfReaderSearchRunEvent() when searchRun != null:
return searchRun(_that.query);case PdfReaderSearchNextEvent() when searchNext != null:
return searchNext();case PdfReaderSearchPreviousEvent() when searchPrevious != null:
return searchPrevious();case PdfReaderSearchClosedEvent() when searchClosed != null:
return searchClosed();case PdfReaderHighlightRequestedEvent() when highlightRequested != null:
return highlightRequested(_that.color,_that.quote);case PdfReaderInkDrawingChangedEvent() when inkDrawingChanged != null:
return inkDrawingChanged(_that.enabled);case PdfReaderInkColorChangedEvent() when inkColorChanged != null:
return inkColorChanged(_that.color);case PdfReaderInkWidthChangedEvent() when inkWidthChanged != null:
return inkWidthChanged(_that.width);case PdfReaderInkStrokeRequestedEvent() when inkStrokeRequested != null:
return inkStrokeRequested(_that.page,_that.stroke);case PdfReaderHighlightFocusedEvent() when highlightFocused != null:
return highlightFocused(_that.id);case PdfReaderNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case PdfReaderLockChangedEvent() when lockChanged != null:
return lockChanged();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId,  String file,  String name,  int? page,  String? highlight,  bool dark)  started,required TResult Function( String password)  passwordSubmitted,required TResult Function()  retried,required TResult Function( PdfZoom zoom)  zoomChanged,required TResult Function( bool invert)  invertChanged,required TResult Function( bool dark)  darkChanged,required TResult Function( int page)  pageChanged,required TResult Function( int page)  jumpTo,required TResult Function( int page,  double x,  double y)  wordSelected,required TResult Function( int page,  double x,  double y)  lineSelected,required TResult Function( int page,  double x,  double y,  bool fromStart)  selectionExtended,required TResult Function()  selectionCleared,required TResult Function( String query)  searchChanged,required TResult Function( String query)  searchRun,required TResult Function()  searchNext,required TResult Function()  searchPrevious,required TResult Function()  searchClosed,required TResult Function( String color,  bool quote)  highlightRequested,required TResult Function( bool enabled)  inkDrawingChanged,required TResult Function( String color)  inkColorChanged,required TResult Function( double width)  inkWidthChanged,required TResult Function( int page,  PdfInkStroke stroke)  inkStrokeRequested,required TResult Function( String? id)  highlightFocused,required TResult Function()  noticeDismissed,required TResult Function()  lockChanged,}) {final _that = this;
switch (_that) {
case PdfReaderStartedEvent():
return started(_that.noteId,_that.file,_that.name,_that.page,_that.highlight,_that.dark);case PdfReaderPasswordSubmittedEvent():
return passwordSubmitted(_that.password);case PdfReaderRetriedEvent():
return retried();case PdfReaderZoomChangedEvent():
return zoomChanged(_that.zoom);case PdfReaderInvertChangedEvent():
return invertChanged(_that.invert);case PdfReaderDarkChangedEvent():
return darkChanged(_that.dark);case PdfReaderPageChangedEvent():
return pageChanged(_that.page);case PdfReaderJumpToEvent():
return jumpTo(_that.page);case PdfReaderWordSelectedEvent():
return wordSelected(_that.page,_that.x,_that.y);case PdfReaderLineSelectedEvent():
return lineSelected(_that.page,_that.x,_that.y);case PdfReaderSelectionExtendedEvent():
return selectionExtended(_that.page,_that.x,_that.y,_that.fromStart);case PdfReaderSelectionClearedEvent():
return selectionCleared();case PdfReaderSearchChangedEvent():
return searchChanged(_that.query);case PdfReaderSearchRunEvent():
return searchRun(_that.query);case PdfReaderSearchNextEvent():
return searchNext();case PdfReaderSearchPreviousEvent():
return searchPrevious();case PdfReaderSearchClosedEvent():
return searchClosed();case PdfReaderHighlightRequestedEvent():
return highlightRequested(_that.color,_that.quote);case PdfReaderInkDrawingChangedEvent():
return inkDrawingChanged(_that.enabled);case PdfReaderInkColorChangedEvent():
return inkColorChanged(_that.color);case PdfReaderInkWidthChangedEvent():
return inkWidthChanged(_that.width);case PdfReaderInkStrokeRequestedEvent():
return inkStrokeRequested(_that.page,_that.stroke);case PdfReaderHighlightFocusedEvent():
return highlightFocused(_that.id);case PdfReaderNoticeDismissedEvent():
return noticeDismissed();case PdfReaderLockChangedEvent():
return lockChanged();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId,  String file,  String name,  int? page,  String? highlight,  bool dark)?  started,TResult? Function( String password)?  passwordSubmitted,TResult? Function()?  retried,TResult? Function( PdfZoom zoom)?  zoomChanged,TResult? Function( bool invert)?  invertChanged,TResult? Function( bool dark)?  darkChanged,TResult? Function( int page)?  pageChanged,TResult? Function( int page)?  jumpTo,TResult? Function( int page,  double x,  double y)?  wordSelected,TResult? Function( int page,  double x,  double y)?  lineSelected,TResult? Function( int page,  double x,  double y,  bool fromStart)?  selectionExtended,TResult? Function()?  selectionCleared,TResult? Function( String query)?  searchChanged,TResult? Function( String query)?  searchRun,TResult? Function()?  searchNext,TResult? Function()?  searchPrevious,TResult? Function()?  searchClosed,TResult? Function( String color,  bool quote)?  highlightRequested,TResult? Function( bool enabled)?  inkDrawingChanged,TResult? Function( String color)?  inkColorChanged,TResult? Function( double width)?  inkWidthChanged,TResult? Function( int page,  PdfInkStroke stroke)?  inkStrokeRequested,TResult? Function( String? id)?  highlightFocused,TResult? Function()?  noticeDismissed,TResult? Function()?  lockChanged,}) {final _that = this;
switch (_that) {
case PdfReaderStartedEvent() when started != null:
return started(_that.noteId,_that.file,_that.name,_that.page,_that.highlight,_that.dark);case PdfReaderPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.password);case PdfReaderRetriedEvent() when retried != null:
return retried();case PdfReaderZoomChangedEvent() when zoomChanged != null:
return zoomChanged(_that.zoom);case PdfReaderInvertChangedEvent() when invertChanged != null:
return invertChanged(_that.invert);case PdfReaderDarkChangedEvent() when darkChanged != null:
return darkChanged(_that.dark);case PdfReaderPageChangedEvent() when pageChanged != null:
return pageChanged(_that.page);case PdfReaderJumpToEvent() when jumpTo != null:
return jumpTo(_that.page);case PdfReaderWordSelectedEvent() when wordSelected != null:
return wordSelected(_that.page,_that.x,_that.y);case PdfReaderLineSelectedEvent() when lineSelected != null:
return lineSelected(_that.page,_that.x,_that.y);case PdfReaderSelectionExtendedEvent() when selectionExtended != null:
return selectionExtended(_that.page,_that.x,_that.y,_that.fromStart);case PdfReaderSelectionClearedEvent() when selectionCleared != null:
return selectionCleared();case PdfReaderSearchChangedEvent() when searchChanged != null:
return searchChanged(_that.query);case PdfReaderSearchRunEvent() when searchRun != null:
return searchRun(_that.query);case PdfReaderSearchNextEvent() when searchNext != null:
return searchNext();case PdfReaderSearchPreviousEvent() when searchPrevious != null:
return searchPrevious();case PdfReaderSearchClosedEvent() when searchClosed != null:
return searchClosed();case PdfReaderHighlightRequestedEvent() when highlightRequested != null:
return highlightRequested(_that.color,_that.quote);case PdfReaderInkDrawingChangedEvent() when inkDrawingChanged != null:
return inkDrawingChanged(_that.enabled);case PdfReaderInkColorChangedEvent() when inkColorChanged != null:
return inkColorChanged(_that.color);case PdfReaderInkWidthChangedEvent() when inkWidthChanged != null:
return inkWidthChanged(_that.width);case PdfReaderInkStrokeRequestedEvent() when inkStrokeRequested != null:
return inkStrokeRequested(_that.page,_that.stroke);case PdfReaderHighlightFocusedEvent() when highlightFocused != null:
return highlightFocused(_that.id);case PdfReaderNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case PdfReaderLockChangedEvent() when lockChanged != null:
return lockChanged();case _:
  return null;

}
}

}

/// @nodoc


class PdfReaderStartedEvent implements PdfReaderEvent {
  const PdfReaderStartedEvent({required this.noteId, required this.file, required this.name, this.page, this.highlight, this.dark = false});
  

 final  String noteId;
 final  String file;
 final  String name;
 final  int? page;
 final  String? highlight;
@JsonKey() final  bool dark;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderStartedEventCopyWith<PdfReaderStartedEvent> get copyWith => _$PdfReaderStartedEventCopyWithImpl<PdfReaderStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.file, file) || other.file == file)&&(identical(other.name, name) || other.name == name)&&(identical(other.page, page) || other.page == page)&&(identical(other.highlight, highlight) || other.highlight == highlight)&&(identical(other.dark, dark) || other.dark == dark));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,file,name,page,highlight,dark);

@override
String toString() {
  return 'PdfReaderEvent.started(noteId: $noteId, file: $file, name: $name, page: $page, highlight: $highlight, dark: $dark)';
}


}

/// @nodoc
abstract mixin class $PdfReaderStartedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderStartedEventCopyWith(PdfReaderStartedEvent value, $Res Function(PdfReaderStartedEvent) _then) = _$PdfReaderStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, String file, String name, int? page, String? highlight, bool dark
});




}
/// @nodoc
class _$PdfReaderStartedEventCopyWithImpl<$Res>
    implements $PdfReaderStartedEventCopyWith<$Res> {
  _$PdfReaderStartedEventCopyWithImpl(this._self, this._then);

  final PdfReaderStartedEvent _self;
  final $Res Function(PdfReaderStartedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? file = null,Object? name = null,Object? page = freezed,Object? highlight = freezed,Object? dark = null,}) {
  return _then(PdfReaderStartedEvent(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,page: freezed == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int?,highlight: freezed == highlight ? _self.highlight : highlight // ignore: cast_nullable_to_non_nullable
as String?,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PdfReaderPasswordSubmittedEvent implements PdfReaderEvent {
  const PdfReaderPasswordSubmittedEvent(this.password);
  

 final  String password;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderPasswordSubmittedEventCopyWith<PdfReaderPasswordSubmittedEvent> get copyWith => _$PdfReaderPasswordSubmittedEventCopyWithImpl<PdfReaderPasswordSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderPasswordSubmittedEvent&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,password);

@override
String toString() {
  return 'PdfReaderEvent.passwordSubmitted(password: $password)';
}


}

/// @nodoc
abstract mixin class $PdfReaderPasswordSubmittedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderPasswordSubmittedEventCopyWith(PdfReaderPasswordSubmittedEvent value, $Res Function(PdfReaderPasswordSubmittedEvent) _then) = _$PdfReaderPasswordSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String password
});




}
/// @nodoc
class _$PdfReaderPasswordSubmittedEventCopyWithImpl<$Res>
    implements $PdfReaderPasswordSubmittedEventCopyWith<$Res> {
  _$PdfReaderPasswordSubmittedEventCopyWithImpl(this._self, this._then);

  final PdfReaderPasswordSubmittedEvent _self;
  final $Res Function(PdfReaderPasswordSubmittedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = null,}) {
  return _then(PdfReaderPasswordSubmittedEvent(
null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfReaderRetriedEvent implements PdfReaderEvent {
  const PdfReaderRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.retried()';
}


}




/// @nodoc


class PdfReaderZoomChangedEvent implements PdfReaderEvent {
  const PdfReaderZoomChangedEvent(this.zoom);
  

 final  PdfZoom zoom;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderZoomChangedEventCopyWith<PdfReaderZoomChangedEvent> get copyWith => _$PdfReaderZoomChangedEventCopyWithImpl<PdfReaderZoomChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderZoomChangedEvent&&(identical(other.zoom, zoom) || other.zoom == zoom));
}


@override
int get hashCode => Object.hash(runtimeType,zoom);

@override
String toString() {
  return 'PdfReaderEvent.zoomChanged(zoom: $zoom)';
}


}

/// @nodoc
abstract mixin class $PdfReaderZoomChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderZoomChangedEventCopyWith(PdfReaderZoomChangedEvent value, $Res Function(PdfReaderZoomChangedEvent) _then) = _$PdfReaderZoomChangedEventCopyWithImpl;
@useResult
$Res call({
 PdfZoom zoom
});




}
/// @nodoc
class _$PdfReaderZoomChangedEventCopyWithImpl<$Res>
    implements $PdfReaderZoomChangedEventCopyWith<$Res> {
  _$PdfReaderZoomChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderZoomChangedEvent _self;
  final $Res Function(PdfReaderZoomChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? zoom = null,}) {
  return _then(PdfReaderZoomChangedEvent(
null == zoom ? _self.zoom : zoom // ignore: cast_nullable_to_non_nullable
as PdfZoom,
  ));
}


}

/// @nodoc


class PdfReaderInvertChangedEvent implements PdfReaderEvent {
  const PdfReaderInvertChangedEvent({required this.invert});
  

 final  bool invert;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderInvertChangedEventCopyWith<PdfReaderInvertChangedEvent> get copyWith => _$PdfReaderInvertChangedEventCopyWithImpl<PdfReaderInvertChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderInvertChangedEvent&&(identical(other.invert, invert) || other.invert == invert));
}


@override
int get hashCode => Object.hash(runtimeType,invert);

@override
String toString() {
  return 'PdfReaderEvent.invertChanged(invert: $invert)';
}


}

/// @nodoc
abstract mixin class $PdfReaderInvertChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderInvertChangedEventCopyWith(PdfReaderInvertChangedEvent value, $Res Function(PdfReaderInvertChangedEvent) _then) = _$PdfReaderInvertChangedEventCopyWithImpl;
@useResult
$Res call({
 bool invert
});




}
/// @nodoc
class _$PdfReaderInvertChangedEventCopyWithImpl<$Res>
    implements $PdfReaderInvertChangedEventCopyWith<$Res> {
  _$PdfReaderInvertChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderInvertChangedEvent _self;
  final $Res Function(PdfReaderInvertChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? invert = null,}) {
  return _then(PdfReaderInvertChangedEvent(
invert: null == invert ? _self.invert : invert // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PdfReaderDarkChangedEvent implements PdfReaderEvent {
  const PdfReaderDarkChangedEvent({required this.dark});
  

 final  bool dark;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderDarkChangedEventCopyWith<PdfReaderDarkChangedEvent> get copyWith => _$PdfReaderDarkChangedEventCopyWithImpl<PdfReaderDarkChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderDarkChangedEvent&&(identical(other.dark, dark) || other.dark == dark));
}


@override
int get hashCode => Object.hash(runtimeType,dark);

@override
String toString() {
  return 'PdfReaderEvent.darkChanged(dark: $dark)';
}


}

/// @nodoc
abstract mixin class $PdfReaderDarkChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderDarkChangedEventCopyWith(PdfReaderDarkChangedEvent value, $Res Function(PdfReaderDarkChangedEvent) _then) = _$PdfReaderDarkChangedEventCopyWithImpl;
@useResult
$Res call({
 bool dark
});




}
/// @nodoc
class _$PdfReaderDarkChangedEventCopyWithImpl<$Res>
    implements $PdfReaderDarkChangedEventCopyWith<$Res> {
  _$PdfReaderDarkChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderDarkChangedEvent _self;
  final $Res Function(PdfReaderDarkChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? dark = null,}) {
  return _then(PdfReaderDarkChangedEvent(
dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PdfReaderPageChangedEvent implements PdfReaderEvent {
  const PdfReaderPageChangedEvent(this.page);
  

 final  int page;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderPageChangedEventCopyWith<PdfReaderPageChangedEvent> get copyWith => _$PdfReaderPageChangedEventCopyWithImpl<PdfReaderPageChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderPageChangedEvent&&(identical(other.page, page) || other.page == page));
}


@override
int get hashCode => Object.hash(runtimeType,page);

@override
String toString() {
  return 'PdfReaderEvent.pageChanged(page: $page)';
}


}

/// @nodoc
abstract mixin class $PdfReaderPageChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderPageChangedEventCopyWith(PdfReaderPageChangedEvent value, $Res Function(PdfReaderPageChangedEvent) _then) = _$PdfReaderPageChangedEventCopyWithImpl;
@useResult
$Res call({
 int page
});




}
/// @nodoc
class _$PdfReaderPageChangedEventCopyWithImpl<$Res>
    implements $PdfReaderPageChangedEventCopyWith<$Res> {
  _$PdfReaderPageChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderPageChangedEvent _self;
  final $Res Function(PdfReaderPageChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? page = null,}) {
  return _then(PdfReaderPageChangedEvent(
null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PdfReaderJumpToEvent implements PdfReaderEvent {
  const PdfReaderJumpToEvent(this.page);
  

 final  int page;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderJumpToEventCopyWith<PdfReaderJumpToEvent> get copyWith => _$PdfReaderJumpToEventCopyWithImpl<PdfReaderJumpToEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderJumpToEvent&&(identical(other.page, page) || other.page == page));
}


@override
int get hashCode => Object.hash(runtimeType,page);

@override
String toString() {
  return 'PdfReaderEvent.jumpTo(page: $page)';
}


}

/// @nodoc
abstract mixin class $PdfReaderJumpToEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderJumpToEventCopyWith(PdfReaderJumpToEvent value, $Res Function(PdfReaderJumpToEvent) _then) = _$PdfReaderJumpToEventCopyWithImpl;
@useResult
$Res call({
 int page
});




}
/// @nodoc
class _$PdfReaderJumpToEventCopyWithImpl<$Res>
    implements $PdfReaderJumpToEventCopyWith<$Res> {
  _$PdfReaderJumpToEventCopyWithImpl(this._self, this._then);

  final PdfReaderJumpToEvent _self;
  final $Res Function(PdfReaderJumpToEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? page = null,}) {
  return _then(PdfReaderJumpToEvent(
null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PdfReaderWordSelectedEvent implements PdfReaderEvent {
  const PdfReaderWordSelectedEvent(this.page, this.x, this.y);
  

 final  int page;
 final  double x;
 final  double y;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderWordSelectedEventCopyWith<PdfReaderWordSelectedEvent> get copyWith => _$PdfReaderWordSelectedEventCopyWithImpl<PdfReaderWordSelectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderWordSelectedEvent&&(identical(other.page, page) || other.page == page)&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}


@override
int get hashCode => Object.hash(runtimeType,page,x,y);

@override
String toString() {
  return 'PdfReaderEvent.wordSelected(page: $page, x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class $PdfReaderWordSelectedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderWordSelectedEventCopyWith(PdfReaderWordSelectedEvent value, $Res Function(PdfReaderWordSelectedEvent) _then) = _$PdfReaderWordSelectedEventCopyWithImpl;
@useResult
$Res call({
 int page, double x, double y
});




}
/// @nodoc
class _$PdfReaderWordSelectedEventCopyWithImpl<$Res>
    implements $PdfReaderWordSelectedEventCopyWith<$Res> {
  _$PdfReaderWordSelectedEventCopyWithImpl(this._self, this._then);

  final PdfReaderWordSelectedEvent _self;
  final $Res Function(PdfReaderWordSelectedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? page = null,Object? x = null,Object? y = null,}) {
  return _then(PdfReaderWordSelectedEvent(
null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PdfReaderLineSelectedEvent implements PdfReaderEvent {
  const PdfReaderLineSelectedEvent(this.page, this.x, this.y);
  

 final  int page;
 final  double x;
 final  double y;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderLineSelectedEventCopyWith<PdfReaderLineSelectedEvent> get copyWith => _$PdfReaderLineSelectedEventCopyWithImpl<PdfReaderLineSelectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderLineSelectedEvent&&(identical(other.page, page) || other.page == page)&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y));
}


@override
int get hashCode => Object.hash(runtimeType,page,x,y);

@override
String toString() {
  return 'PdfReaderEvent.lineSelected(page: $page, x: $x, y: $y)';
}


}

/// @nodoc
abstract mixin class $PdfReaderLineSelectedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderLineSelectedEventCopyWith(PdfReaderLineSelectedEvent value, $Res Function(PdfReaderLineSelectedEvent) _then) = _$PdfReaderLineSelectedEventCopyWithImpl;
@useResult
$Res call({
 int page, double x, double y
});




}
/// @nodoc
class _$PdfReaderLineSelectedEventCopyWithImpl<$Res>
    implements $PdfReaderLineSelectedEventCopyWith<$Res> {
  _$PdfReaderLineSelectedEventCopyWithImpl(this._self, this._then);

  final PdfReaderLineSelectedEvent _self;
  final $Res Function(PdfReaderLineSelectedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? page = null,Object? x = null,Object? y = null,}) {
  return _then(PdfReaderLineSelectedEvent(
null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PdfReaderSelectionExtendedEvent implements PdfReaderEvent {
  const PdfReaderSelectionExtendedEvent(this.page, this.x, this.y, {required this.fromStart});
  

 final  int page;
 final  double x;
 final  double y;
 final  bool fromStart;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderSelectionExtendedEventCopyWith<PdfReaderSelectionExtendedEvent> get copyWith => _$PdfReaderSelectionExtendedEventCopyWithImpl<PdfReaderSelectionExtendedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSelectionExtendedEvent&&(identical(other.page, page) || other.page == page)&&(identical(other.x, x) || other.x == x)&&(identical(other.y, y) || other.y == y)&&(identical(other.fromStart, fromStart) || other.fromStart == fromStart));
}


@override
int get hashCode => Object.hash(runtimeType,page,x,y,fromStart);

@override
String toString() {
  return 'PdfReaderEvent.selectionExtended(page: $page, x: $x, y: $y, fromStart: $fromStart)';
}


}

/// @nodoc
abstract mixin class $PdfReaderSelectionExtendedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderSelectionExtendedEventCopyWith(PdfReaderSelectionExtendedEvent value, $Res Function(PdfReaderSelectionExtendedEvent) _then) = _$PdfReaderSelectionExtendedEventCopyWithImpl;
@useResult
$Res call({
 int page, double x, double y, bool fromStart
});




}
/// @nodoc
class _$PdfReaderSelectionExtendedEventCopyWithImpl<$Res>
    implements $PdfReaderSelectionExtendedEventCopyWith<$Res> {
  _$PdfReaderSelectionExtendedEventCopyWithImpl(this._self, this._then);

  final PdfReaderSelectionExtendedEvent _self;
  final $Res Function(PdfReaderSelectionExtendedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? page = null,Object? x = null,Object? y = null,Object? fromStart = null,}) {
  return _then(PdfReaderSelectionExtendedEvent(
null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,null == x ? _self.x : x // ignore: cast_nullable_to_non_nullable
as double,null == y ? _self.y : y // ignore: cast_nullable_to_non_nullable
as double,fromStart: null == fromStart ? _self.fromStart : fromStart // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PdfReaderSelectionClearedEvent implements PdfReaderEvent {
  const PdfReaderSelectionClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSelectionClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.selectionCleared()';
}


}




/// @nodoc


class PdfReaderSearchChangedEvent implements PdfReaderEvent {
  const PdfReaderSearchChangedEvent(this.query);
  

 final  String query;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderSearchChangedEventCopyWith<PdfReaderSearchChangedEvent> get copyWith => _$PdfReaderSearchChangedEventCopyWithImpl<PdfReaderSearchChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSearchChangedEvent&&(identical(other.query, query) || other.query == query));
}


@override
int get hashCode => Object.hash(runtimeType,query);

@override
String toString() {
  return 'PdfReaderEvent.searchChanged(query: $query)';
}


}

/// @nodoc
abstract mixin class $PdfReaderSearchChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderSearchChangedEventCopyWith(PdfReaderSearchChangedEvent value, $Res Function(PdfReaderSearchChangedEvent) _then) = _$PdfReaderSearchChangedEventCopyWithImpl;
@useResult
$Res call({
 String query
});




}
/// @nodoc
class _$PdfReaderSearchChangedEventCopyWithImpl<$Res>
    implements $PdfReaderSearchChangedEventCopyWith<$Res> {
  _$PdfReaderSearchChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderSearchChangedEvent _self;
  final $Res Function(PdfReaderSearchChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,}) {
  return _then(PdfReaderSearchChangedEvent(
null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfReaderSearchRunEvent implements PdfReaderEvent {
  const PdfReaderSearchRunEvent(this.query);
  

 final  String query;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderSearchRunEventCopyWith<PdfReaderSearchRunEvent> get copyWith => _$PdfReaderSearchRunEventCopyWithImpl<PdfReaderSearchRunEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSearchRunEvent&&(identical(other.query, query) || other.query == query));
}


@override
int get hashCode => Object.hash(runtimeType,query);

@override
String toString() {
  return 'PdfReaderEvent.searchRun(query: $query)';
}


}

/// @nodoc
abstract mixin class $PdfReaderSearchRunEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderSearchRunEventCopyWith(PdfReaderSearchRunEvent value, $Res Function(PdfReaderSearchRunEvent) _then) = _$PdfReaderSearchRunEventCopyWithImpl;
@useResult
$Res call({
 String query
});




}
/// @nodoc
class _$PdfReaderSearchRunEventCopyWithImpl<$Res>
    implements $PdfReaderSearchRunEventCopyWith<$Res> {
  _$PdfReaderSearchRunEventCopyWithImpl(this._self, this._then);

  final PdfReaderSearchRunEvent _self;
  final $Res Function(PdfReaderSearchRunEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,}) {
  return _then(PdfReaderSearchRunEvent(
null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfReaderSearchNextEvent implements PdfReaderEvent {
  const PdfReaderSearchNextEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSearchNextEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.searchNext()';
}


}




/// @nodoc


class PdfReaderSearchPreviousEvent implements PdfReaderEvent {
  const PdfReaderSearchPreviousEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSearchPreviousEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.searchPrevious()';
}


}




/// @nodoc


class PdfReaderSearchClosedEvent implements PdfReaderEvent {
  const PdfReaderSearchClosedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderSearchClosedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.searchClosed()';
}


}




/// @nodoc


class PdfReaderHighlightRequestedEvent implements PdfReaderEvent {
  const PdfReaderHighlightRequestedEvent({required this.color, required this.quote});
  

 final  String color;
 final  bool quote;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderHighlightRequestedEventCopyWith<PdfReaderHighlightRequestedEvent> get copyWith => _$PdfReaderHighlightRequestedEventCopyWithImpl<PdfReaderHighlightRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderHighlightRequestedEvent&&(identical(other.color, color) || other.color == color)&&(identical(other.quote, quote) || other.quote == quote));
}


@override
int get hashCode => Object.hash(runtimeType,color,quote);

@override
String toString() {
  return 'PdfReaderEvent.highlightRequested(color: $color, quote: $quote)';
}


}

/// @nodoc
abstract mixin class $PdfReaderHighlightRequestedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderHighlightRequestedEventCopyWith(PdfReaderHighlightRequestedEvent value, $Res Function(PdfReaderHighlightRequestedEvent) _then) = _$PdfReaderHighlightRequestedEventCopyWithImpl;
@useResult
$Res call({
 String color, bool quote
});




}
/// @nodoc
class _$PdfReaderHighlightRequestedEventCopyWithImpl<$Res>
    implements $PdfReaderHighlightRequestedEventCopyWith<$Res> {
  _$PdfReaderHighlightRequestedEventCopyWithImpl(this._self, this._then);

  final PdfReaderHighlightRequestedEvent _self;
  final $Res Function(PdfReaderHighlightRequestedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? color = null,Object? quote = null,}) {
  return _then(PdfReaderHighlightRequestedEvent(
color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PdfReaderInkDrawingChangedEvent implements PdfReaderEvent {
  const PdfReaderInkDrawingChangedEvent(this.enabled);
  

 final  bool enabled;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderInkDrawingChangedEventCopyWith<PdfReaderInkDrawingChangedEvent> get copyWith => _$PdfReaderInkDrawingChangedEventCopyWithImpl<PdfReaderInkDrawingChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderInkDrawingChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'PdfReaderEvent.inkDrawingChanged(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $PdfReaderInkDrawingChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderInkDrawingChangedEventCopyWith(PdfReaderInkDrawingChangedEvent value, $Res Function(PdfReaderInkDrawingChangedEvent) _then) = _$PdfReaderInkDrawingChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$PdfReaderInkDrawingChangedEventCopyWithImpl<$Res>
    implements $PdfReaderInkDrawingChangedEventCopyWith<$Res> {
  _$PdfReaderInkDrawingChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderInkDrawingChangedEvent _self;
  final $Res Function(PdfReaderInkDrawingChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(PdfReaderInkDrawingChangedEvent(
null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PdfReaderInkColorChangedEvent implements PdfReaderEvent {
  const PdfReaderInkColorChangedEvent(this.color);
  

 final  String color;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderInkColorChangedEventCopyWith<PdfReaderInkColorChangedEvent> get copyWith => _$PdfReaderInkColorChangedEventCopyWithImpl<PdfReaderInkColorChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderInkColorChangedEvent&&(identical(other.color, color) || other.color == color));
}


@override
int get hashCode => Object.hash(runtimeType,color);

@override
String toString() {
  return 'PdfReaderEvent.inkColorChanged(color: $color)';
}


}

/// @nodoc
abstract mixin class $PdfReaderInkColorChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderInkColorChangedEventCopyWith(PdfReaderInkColorChangedEvent value, $Res Function(PdfReaderInkColorChangedEvent) _then) = _$PdfReaderInkColorChangedEventCopyWithImpl;
@useResult
$Res call({
 String color
});




}
/// @nodoc
class _$PdfReaderInkColorChangedEventCopyWithImpl<$Res>
    implements $PdfReaderInkColorChangedEventCopyWith<$Res> {
  _$PdfReaderInkColorChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderInkColorChangedEvent _self;
  final $Res Function(PdfReaderInkColorChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? color = null,}) {
  return _then(PdfReaderInkColorChangedEvent(
null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PdfReaderInkWidthChangedEvent implements PdfReaderEvent {
  const PdfReaderInkWidthChangedEvent(this.width);
  

 final  double width;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderInkWidthChangedEventCopyWith<PdfReaderInkWidthChangedEvent> get copyWith => _$PdfReaderInkWidthChangedEventCopyWithImpl<PdfReaderInkWidthChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderInkWidthChangedEvent&&(identical(other.width, width) || other.width == width));
}


@override
int get hashCode => Object.hash(runtimeType,width);

@override
String toString() {
  return 'PdfReaderEvent.inkWidthChanged(width: $width)';
}


}

/// @nodoc
abstract mixin class $PdfReaderInkWidthChangedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderInkWidthChangedEventCopyWith(PdfReaderInkWidthChangedEvent value, $Res Function(PdfReaderInkWidthChangedEvent) _then) = _$PdfReaderInkWidthChangedEventCopyWithImpl;
@useResult
$Res call({
 double width
});




}
/// @nodoc
class _$PdfReaderInkWidthChangedEventCopyWithImpl<$Res>
    implements $PdfReaderInkWidthChangedEventCopyWith<$Res> {
  _$PdfReaderInkWidthChangedEventCopyWithImpl(this._self, this._then);

  final PdfReaderInkWidthChangedEvent _self;
  final $Res Function(PdfReaderInkWidthChangedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? width = null,}) {
  return _then(PdfReaderInkWidthChangedEvent(
null == width ? _self.width : width // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PdfReaderInkStrokeRequestedEvent implements PdfReaderEvent {
  const PdfReaderInkStrokeRequestedEvent({required this.page, required this.stroke});
  

 final  int page;
 final  PdfInkStroke stroke;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderInkStrokeRequestedEventCopyWith<PdfReaderInkStrokeRequestedEvent> get copyWith => _$PdfReaderInkStrokeRequestedEventCopyWithImpl<PdfReaderInkStrokeRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderInkStrokeRequestedEvent&&(identical(other.page, page) || other.page == page)&&(identical(other.stroke, stroke) || other.stroke == stroke));
}


@override
int get hashCode => Object.hash(runtimeType,page,stroke);

@override
String toString() {
  return 'PdfReaderEvent.inkStrokeRequested(page: $page, stroke: $stroke)';
}


}

/// @nodoc
abstract mixin class $PdfReaderInkStrokeRequestedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderInkStrokeRequestedEventCopyWith(PdfReaderInkStrokeRequestedEvent value, $Res Function(PdfReaderInkStrokeRequestedEvent) _then) = _$PdfReaderInkStrokeRequestedEventCopyWithImpl;
@useResult
$Res call({
 int page, PdfInkStroke stroke
});




}
/// @nodoc
class _$PdfReaderInkStrokeRequestedEventCopyWithImpl<$Res>
    implements $PdfReaderInkStrokeRequestedEventCopyWith<$Res> {
  _$PdfReaderInkStrokeRequestedEventCopyWithImpl(this._self, this._then);

  final PdfReaderInkStrokeRequestedEvent _self;
  final $Res Function(PdfReaderInkStrokeRequestedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? page = null,Object? stroke = null,}) {
  return _then(PdfReaderInkStrokeRequestedEvent(
page: null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,stroke: null == stroke ? _self.stroke : stroke // ignore: cast_nullable_to_non_nullable
as PdfInkStroke,
  ));
}


}

/// @nodoc


class PdfReaderHighlightFocusedEvent implements PdfReaderEvent {
  const PdfReaderHighlightFocusedEvent(this.id);
  

 final  String? id;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderHighlightFocusedEventCopyWith<PdfReaderHighlightFocusedEvent> get copyWith => _$PdfReaderHighlightFocusedEventCopyWithImpl<PdfReaderHighlightFocusedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderHighlightFocusedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'PdfReaderEvent.highlightFocused(id: $id)';
}


}

/// @nodoc
abstract mixin class $PdfReaderHighlightFocusedEventCopyWith<$Res> implements $PdfReaderEventCopyWith<$Res> {
  factory $PdfReaderHighlightFocusedEventCopyWith(PdfReaderHighlightFocusedEvent value, $Res Function(PdfReaderHighlightFocusedEvent) _then) = _$PdfReaderHighlightFocusedEventCopyWithImpl;
@useResult
$Res call({
 String? id
});




}
/// @nodoc
class _$PdfReaderHighlightFocusedEventCopyWithImpl<$Res>
    implements $PdfReaderHighlightFocusedEventCopyWith<$Res> {
  _$PdfReaderHighlightFocusedEventCopyWithImpl(this._self, this._then);

  final PdfReaderHighlightFocusedEvent _self;
  final $Res Function(PdfReaderHighlightFocusedEvent) _then;

/// Create a copy of PdfReaderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = freezed,}) {
  return _then(PdfReaderHighlightFocusedEvent(
freezed == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class PdfReaderNoticeDismissedEvent implements PdfReaderEvent {
  const PdfReaderNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.noticeDismissed()';
}


}




/// @nodoc


class PdfReaderLockChangedEvent implements PdfReaderEvent {
  const PdfReaderLockChangedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderLockChangedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PdfReaderEvent.lockChanged()';
}


}




/// @nodoc
mixin _$PdfReaderState {

/// The note the attachment belongs to.
 String get noteId;/// The attachment (`file:<id>`).
 String get file;/// The file's name, shown in the bar.
 String get name;/// Where opening stands.
 PdfReaderStatus get status;/// What opened: its pages, their sizes and whether highlights can be
/// written.
 PdfInfo? get info;/// The note's highlights for the document, with the revision they were
/// read at.
 PdfMarks? get marks;/// Why it was not opened, when the document itself said so.
 PdfDeclined? get declined;/// Why opening it failed, when it did.
 ApplicationError? get error;/// Why the highlights could not be read; the pages are still readable.
 ApplicationError? get marksError;/// The page width, as a multiple of the screen's.
 PdfZoom get zoom;/// Whether pages are drawn for a dark screen.
 bool get invert;/// The page in view, 1-based.
 int get page;/// Counts the times the page was asked for (a search's hit, a highlight,
/// a typed page): the view scrolls when it changes, not when a scroll
/// itself moved [page].
 int get jump;/// The selection, when one is made.
 PdfSelection? get selection;/// The text searched for.
 String get query;/// The hits of [query]; null until a search has run.
 PdfFound? get found;/// Which hit of [found] is shown, 0-based.
 int get hit;/// Whether a selection is being highlighted.
 bool get adding;/// Whether page gestures draw pencil ink.
 bool get inkDrawing;/// The selected pencil color.
 String get inkColor;/// The selected pencil width in PDF page units.
 double get inkWidth;/// Whether a pencil stroke is being saved.
 bool get addingInk;/// Whether the password last typed was wrong.
 bool get passwordWrong;/// The highlight a link named, to keep in view.
 String? get focusHighlight;/// Why the last action did nothing, said once and dismissed.
 ApplicationError? get notice;
/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PdfReaderStateCopyWith<PdfReaderState> get copyWith => _$PdfReaderStateCopyWithImpl<PdfReaderState>(this as PdfReaderState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PdfReaderState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.file, file) || other.file == file)&&(identical(other.name, name) || other.name == name)&&(identical(other.status, status) || other.status == status)&&(identical(other.info, info) || other.info == info)&&(identical(other.marks, marks) || other.marks == marks)&&(identical(other.declined, declined) || other.declined == declined)&&(identical(other.error, error) || other.error == error)&&(identical(other.marksError, marksError) || other.marksError == marksError)&&(identical(other.zoom, zoom) || other.zoom == zoom)&&(identical(other.invert, invert) || other.invert == invert)&&(identical(other.page, page) || other.page == page)&&(identical(other.jump, jump) || other.jump == jump)&&(identical(other.selection, selection) || other.selection == selection)&&(identical(other.query, query) || other.query == query)&&(identical(other.found, found) || other.found == found)&&(identical(other.hit, hit) || other.hit == hit)&&(identical(other.adding, adding) || other.adding == adding)&&(identical(other.inkDrawing, inkDrawing) || other.inkDrawing == inkDrawing)&&(identical(other.inkColor, inkColor) || other.inkColor == inkColor)&&(identical(other.inkWidth, inkWidth) || other.inkWidth == inkWidth)&&(identical(other.addingInk, addingInk) || other.addingInk == addingInk)&&(identical(other.passwordWrong, passwordWrong) || other.passwordWrong == passwordWrong)&&(identical(other.focusHighlight, focusHighlight) || other.focusHighlight == focusHighlight)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hashAll([runtimeType,noteId,file,name,status,info,marks,declined,error,marksError,zoom,invert,page,jump,selection,query,found,hit,adding,inkDrawing,inkColor,inkWidth,addingInk,passwordWrong,focusHighlight,notice]);

@override
String toString() {
  return 'PdfReaderState(noteId: $noteId, file: $file, name: $name, status: $status, info: $info, marks: $marks, declined: $declined, error: $error, marksError: $marksError, zoom: $zoom, invert: $invert, page: $page, jump: $jump, selection: $selection, query: $query, found: $found, hit: $hit, adding: $adding, inkDrawing: $inkDrawing, inkColor: $inkColor, inkWidth: $inkWidth, addingInk: $addingInk, passwordWrong: $passwordWrong, focusHighlight: $focusHighlight, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $PdfReaderStateCopyWith<$Res>  {
  factory $PdfReaderStateCopyWith(PdfReaderState value, $Res Function(PdfReaderState) _then) = _$PdfReaderStateCopyWithImpl;
@useResult
$Res call({
 String noteId, String file, String name, PdfReaderStatus status, PdfInfo? info, PdfMarks? marks, PdfDeclined? declined, ApplicationError? error, ApplicationError? marksError, PdfZoom zoom, bool invert, int page, int jump, PdfSelection? selection, String query, PdfFound? found, int hit, bool adding, bool inkDrawing, String inkColor, double inkWidth, bool addingInk, bool passwordWrong, String? focusHighlight, ApplicationError? notice
});


$PdfDeclinedCopyWith<$Res>? get declined;$ApplicationErrorCopyWith<$Res>? get error;$ApplicationErrorCopyWith<$Res>? get marksError;$ApplicationErrorCopyWith<$Res>? get notice;

}
/// @nodoc
class _$PdfReaderStateCopyWithImpl<$Res>
    implements $PdfReaderStateCopyWith<$Res> {
  _$PdfReaderStateCopyWithImpl(this._self, this._then);

  final PdfReaderState _self;
  final $Res Function(PdfReaderState) _then;

/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? file = null,Object? name = null,Object? status = null,Object? info = freezed,Object? marks = freezed,Object? declined = freezed,Object? error = freezed,Object? marksError = freezed,Object? zoom = null,Object? invert = null,Object? page = null,Object? jump = null,Object? selection = freezed,Object? query = null,Object? found = freezed,Object? hit = null,Object? adding = null,Object? inkDrawing = null,Object? inkColor = null,Object? inkWidth = null,Object? addingInk = null,Object? passwordWrong = null,Object? focusHighlight = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as PdfReaderStatus,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as PdfInfo?,marks: freezed == marks ? _self.marks : marks // ignore: cast_nullable_to_non_nullable
as PdfMarks?,declined: freezed == declined ? _self.declined : declined // ignore: cast_nullable_to_non_nullable
as PdfDeclined?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,marksError: freezed == marksError ? _self.marksError : marksError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,zoom: null == zoom ? _self.zoom : zoom // ignore: cast_nullable_to_non_nullable
as PdfZoom,invert: null == invert ? _self.invert : invert // ignore: cast_nullable_to_non_nullable
as bool,page: null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,jump: null == jump ? _self.jump : jump // ignore: cast_nullable_to_non_nullable
as int,selection: freezed == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as PdfSelection?,query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,found: freezed == found ? _self.found : found // ignore: cast_nullable_to_non_nullable
as PdfFound?,hit: null == hit ? _self.hit : hit // ignore: cast_nullable_to_non_nullable
as int,adding: null == adding ? _self.adding : adding // ignore: cast_nullable_to_non_nullable
as bool,inkDrawing: null == inkDrawing ? _self.inkDrawing : inkDrawing // ignore: cast_nullable_to_non_nullable
as bool,inkColor: null == inkColor ? _self.inkColor : inkColor // ignore: cast_nullable_to_non_nullable
as String,inkWidth: null == inkWidth ? _self.inkWidth : inkWidth // ignore: cast_nullable_to_non_nullable
as double,addingInk: null == addingInk ? _self.addingInk : addingInk // ignore: cast_nullable_to_non_nullable
as bool,passwordWrong: null == passwordWrong ? _self.passwordWrong : passwordWrong // ignore: cast_nullable_to_non_nullable
as bool,focusHighlight: freezed == focusHighlight ? _self.focusHighlight : focusHighlight // ignore: cast_nullable_to_non_nullable
as String?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PdfDeclinedCopyWith<$Res>? get declined {
    if (_self.declined == null) {
    return null;
  }

  return $PdfDeclinedCopyWith<$Res>(_self.declined!, (value) {
    return _then(_self.copyWith(declined: value));
  });
}/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get marksError {
    if (_self.marksError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.marksError!, (value) {
    return _then(_self.copyWith(marksError: value));
  });
}/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}
}


/// Adds pattern-matching-related methods to [PdfReaderState].
extension PdfReaderStatePatterns on PdfReaderState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PdfReaderState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PdfReaderState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PdfReaderState value)  $default,){
final _that = this;
switch (_that) {
case _PdfReaderState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PdfReaderState value)?  $default,){
final _that = this;
switch (_that) {
case _PdfReaderState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String file,  String name,  PdfReaderStatus status,  PdfInfo? info,  PdfMarks? marks,  PdfDeclined? declined,  ApplicationError? error,  ApplicationError? marksError,  PdfZoom zoom,  bool invert,  int page,  int jump,  PdfSelection? selection,  String query,  PdfFound? found,  int hit,  bool adding,  bool inkDrawing,  String inkColor,  double inkWidth,  bool addingInk,  bool passwordWrong,  String? focusHighlight,  ApplicationError? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PdfReaderState() when $default != null:
return $default(_that.noteId,_that.file,_that.name,_that.status,_that.info,_that.marks,_that.declined,_that.error,_that.marksError,_that.zoom,_that.invert,_that.page,_that.jump,_that.selection,_that.query,_that.found,_that.hit,_that.adding,_that.inkDrawing,_that.inkColor,_that.inkWidth,_that.addingInk,_that.passwordWrong,_that.focusHighlight,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String file,  String name,  PdfReaderStatus status,  PdfInfo? info,  PdfMarks? marks,  PdfDeclined? declined,  ApplicationError? error,  ApplicationError? marksError,  PdfZoom zoom,  bool invert,  int page,  int jump,  PdfSelection? selection,  String query,  PdfFound? found,  int hit,  bool adding,  bool inkDrawing,  String inkColor,  double inkWidth,  bool addingInk,  bool passwordWrong,  String? focusHighlight,  ApplicationError? notice)  $default,) {final _that = this;
switch (_that) {
case _PdfReaderState():
return $default(_that.noteId,_that.file,_that.name,_that.status,_that.info,_that.marks,_that.declined,_that.error,_that.marksError,_that.zoom,_that.invert,_that.page,_that.jump,_that.selection,_that.query,_that.found,_that.hit,_that.adding,_that.inkDrawing,_that.inkColor,_that.inkWidth,_that.addingInk,_that.passwordWrong,_that.focusHighlight,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String file,  String name,  PdfReaderStatus status,  PdfInfo? info,  PdfMarks? marks,  PdfDeclined? declined,  ApplicationError? error,  ApplicationError? marksError,  PdfZoom zoom,  bool invert,  int page,  int jump,  PdfSelection? selection,  String query,  PdfFound? found,  int hit,  bool adding,  bool inkDrawing,  String inkColor,  double inkWidth,  bool addingInk,  bool passwordWrong,  String? focusHighlight,  ApplicationError? notice)?  $default,) {final _that = this;
switch (_that) {
case _PdfReaderState() when $default != null:
return $default(_that.noteId,_that.file,_that.name,_that.status,_that.info,_that.marks,_that.declined,_that.error,_that.marksError,_that.zoom,_that.invert,_that.page,_that.jump,_that.selection,_that.query,_that.found,_that.hit,_that.adding,_that.inkDrawing,_that.inkColor,_that.inkWidth,_that.addingInk,_that.passwordWrong,_that.focusHighlight,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _PdfReaderState implements PdfReaderState {
  const _PdfReaderState({this.noteId = '', this.file = '', this.name = '', this.status = PdfReaderStatus.opening, this.info, this.marks, this.declined, this.error, this.marksError, this.zoom = PdfZoom.fitWidth, this.invert = false, this.page = 1, this.jump = 0, this.selection, this.query = '', this.found, this.hit = 0, this.adding = false, this.inkDrawing = false, this.inkColor = '#1e1e1e', this.inkWidth = 4, this.addingInk = false, this.passwordWrong = false, this.focusHighlight, this.notice});
  

/// The note the attachment belongs to.
@override@JsonKey() final  String noteId;
/// The attachment (`file:<id>`).
@override@JsonKey() final  String file;
/// The file's name, shown in the bar.
@override@JsonKey() final  String name;
/// Where opening stands.
@override@JsonKey() final  PdfReaderStatus status;
/// What opened: its pages, their sizes and whether highlights can be
/// written.
@override final  PdfInfo? info;
/// The note's highlights for the document, with the revision they were
/// read at.
@override final  PdfMarks? marks;
/// Why it was not opened, when the document itself said so.
@override final  PdfDeclined? declined;
/// Why opening it failed, when it did.
@override final  ApplicationError? error;
/// Why the highlights could not be read; the pages are still readable.
@override final  ApplicationError? marksError;
/// The page width, as a multiple of the screen's.
@override@JsonKey() final  PdfZoom zoom;
/// Whether pages are drawn for a dark screen.
@override@JsonKey() final  bool invert;
/// The page in view, 1-based.
@override@JsonKey() final  int page;
/// Counts the times the page was asked for (a search's hit, a highlight,
/// a typed page): the view scrolls when it changes, not when a scroll
/// itself moved [page].
@override@JsonKey() final  int jump;
/// The selection, when one is made.
@override final  PdfSelection? selection;
/// The text searched for.
@override@JsonKey() final  String query;
/// The hits of [query]; null until a search has run.
@override final  PdfFound? found;
/// Which hit of [found] is shown, 0-based.
@override@JsonKey() final  int hit;
/// Whether a selection is being highlighted.
@override@JsonKey() final  bool adding;
/// Whether page gestures draw pencil ink.
@override@JsonKey() final  bool inkDrawing;
/// The selected pencil color.
@override@JsonKey() final  String inkColor;
/// The selected pencil width in PDF page units.
@override@JsonKey() final  double inkWidth;
/// Whether a pencil stroke is being saved.
@override@JsonKey() final  bool addingInk;
/// Whether the password last typed was wrong.
@override@JsonKey() final  bool passwordWrong;
/// The highlight a link named, to keep in view.
@override final  String? focusHighlight;
/// Why the last action did nothing, said once and dismissed.
@override final  ApplicationError? notice;

/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PdfReaderStateCopyWith<_PdfReaderState> get copyWith => __$PdfReaderStateCopyWithImpl<_PdfReaderState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PdfReaderState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.file, file) || other.file == file)&&(identical(other.name, name) || other.name == name)&&(identical(other.status, status) || other.status == status)&&(identical(other.info, info) || other.info == info)&&(identical(other.marks, marks) || other.marks == marks)&&(identical(other.declined, declined) || other.declined == declined)&&(identical(other.error, error) || other.error == error)&&(identical(other.marksError, marksError) || other.marksError == marksError)&&(identical(other.zoom, zoom) || other.zoom == zoom)&&(identical(other.invert, invert) || other.invert == invert)&&(identical(other.page, page) || other.page == page)&&(identical(other.jump, jump) || other.jump == jump)&&(identical(other.selection, selection) || other.selection == selection)&&(identical(other.query, query) || other.query == query)&&(identical(other.found, found) || other.found == found)&&(identical(other.hit, hit) || other.hit == hit)&&(identical(other.adding, adding) || other.adding == adding)&&(identical(other.inkDrawing, inkDrawing) || other.inkDrawing == inkDrawing)&&(identical(other.inkColor, inkColor) || other.inkColor == inkColor)&&(identical(other.inkWidth, inkWidth) || other.inkWidth == inkWidth)&&(identical(other.addingInk, addingInk) || other.addingInk == addingInk)&&(identical(other.passwordWrong, passwordWrong) || other.passwordWrong == passwordWrong)&&(identical(other.focusHighlight, focusHighlight) || other.focusHighlight == focusHighlight)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hashAll([runtimeType,noteId,file,name,status,info,marks,declined,error,marksError,zoom,invert,page,jump,selection,query,found,hit,adding,inkDrawing,inkColor,inkWidth,addingInk,passwordWrong,focusHighlight,notice]);

@override
String toString() {
  return 'PdfReaderState(noteId: $noteId, file: $file, name: $name, status: $status, info: $info, marks: $marks, declined: $declined, error: $error, marksError: $marksError, zoom: $zoom, invert: $invert, page: $page, jump: $jump, selection: $selection, query: $query, found: $found, hit: $hit, adding: $adding, inkDrawing: $inkDrawing, inkColor: $inkColor, inkWidth: $inkWidth, addingInk: $addingInk, passwordWrong: $passwordWrong, focusHighlight: $focusHighlight, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$PdfReaderStateCopyWith<$Res> implements $PdfReaderStateCopyWith<$Res> {
  factory _$PdfReaderStateCopyWith(_PdfReaderState value, $Res Function(_PdfReaderState) _then) = __$PdfReaderStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String file, String name, PdfReaderStatus status, PdfInfo? info, PdfMarks? marks, PdfDeclined? declined, ApplicationError? error, ApplicationError? marksError, PdfZoom zoom, bool invert, int page, int jump, PdfSelection? selection, String query, PdfFound? found, int hit, bool adding, bool inkDrawing, String inkColor, double inkWidth, bool addingInk, bool passwordWrong, String? focusHighlight, ApplicationError? notice
});


@override $PdfDeclinedCopyWith<$Res>? get declined;@override $ApplicationErrorCopyWith<$Res>? get error;@override $ApplicationErrorCopyWith<$Res>? get marksError;@override $ApplicationErrorCopyWith<$Res>? get notice;

}
/// @nodoc
class __$PdfReaderStateCopyWithImpl<$Res>
    implements _$PdfReaderStateCopyWith<$Res> {
  __$PdfReaderStateCopyWithImpl(this._self, this._then);

  final _PdfReaderState _self;
  final $Res Function(_PdfReaderState) _then;

/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? file = null,Object? name = null,Object? status = null,Object? info = freezed,Object? marks = freezed,Object? declined = freezed,Object? error = freezed,Object? marksError = freezed,Object? zoom = null,Object? invert = null,Object? page = null,Object? jump = null,Object? selection = freezed,Object? query = null,Object? found = freezed,Object? hit = null,Object? adding = null,Object? inkDrawing = null,Object? inkColor = null,Object? inkWidth = null,Object? addingInk = null,Object? passwordWrong = null,Object? focusHighlight = freezed,Object? notice = freezed,}) {
  return _then(_PdfReaderState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as PdfReaderStatus,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as PdfInfo?,marks: freezed == marks ? _self.marks : marks // ignore: cast_nullable_to_non_nullable
as PdfMarks?,declined: freezed == declined ? _self.declined : declined // ignore: cast_nullable_to_non_nullable
as PdfDeclined?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,marksError: freezed == marksError ? _self.marksError : marksError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,zoom: null == zoom ? _self.zoom : zoom // ignore: cast_nullable_to_non_nullable
as PdfZoom,invert: null == invert ? _self.invert : invert // ignore: cast_nullable_to_non_nullable
as bool,page: null == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int,jump: null == jump ? _self.jump : jump // ignore: cast_nullable_to_non_nullable
as int,selection: freezed == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as PdfSelection?,query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as String,found: freezed == found ? _self.found : found // ignore: cast_nullable_to_non_nullable
as PdfFound?,hit: null == hit ? _self.hit : hit // ignore: cast_nullable_to_non_nullable
as int,adding: null == adding ? _self.adding : adding // ignore: cast_nullable_to_non_nullable
as bool,inkDrawing: null == inkDrawing ? _self.inkDrawing : inkDrawing // ignore: cast_nullable_to_non_nullable
as bool,inkColor: null == inkColor ? _self.inkColor : inkColor // ignore: cast_nullable_to_non_nullable
as String,inkWidth: null == inkWidth ? _self.inkWidth : inkWidth // ignore: cast_nullable_to_non_nullable
as double,addingInk: null == addingInk ? _self.addingInk : addingInk // ignore: cast_nullable_to_non_nullable
as bool,passwordWrong: null == passwordWrong ? _self.passwordWrong : passwordWrong // ignore: cast_nullable_to_non_nullable
as bool,focusHighlight: freezed == focusHighlight ? _self.focusHighlight : focusHighlight // ignore: cast_nullable_to_non_nullable
as String?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PdfDeclinedCopyWith<$Res>? get declined {
    if (_self.declined == null) {
    return null;
  }

  return $PdfDeclinedCopyWith<$Res>(_self.declined!, (value) {
    return _then(_self.copyWith(declined: value));
  });
}/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get marksError {
    if (_self.marksError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.marksError!, (value) {
    return _then(_self.copyWith(marksError: value));
  });
}/// Create a copy of PdfReaderState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}
}

// dart format on
