// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'replace_notes_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ReplaceNotesEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ReplaceNotesEvent()';
}


}

/// @nodoc
class $ReplaceNotesEventCopyWith<$Res>  {
$ReplaceNotesEventCopyWith(ReplaceNotesEvent _, $Res Function(ReplaceNotesEvent) __);
}


/// Adds pattern-matching-related methods to [ReplaceNotesEvent].
extension ReplaceNotesEventPatterns on ReplaceNotesEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ReplaceNotesFindChangedEvent value)?  findChanged,TResult Function( ReplaceNotesReplacementChangedEvent value)?  replacementChanged,TResult Function( ReplaceNotesOptionToggledEvent value)?  optionToggled,TResult Function( ReplaceNotesPreviewDueEvent value)?  previewDue,TResult Function( ReplaceNotesNoteToggledEvent value)?  noteToggled,TResult Function( ReplaceNotesApplyRequestedEvent value)?  applyRequested,TResult Function( ReplaceNotesUndoRequestedEvent value)?  undoRequested,TResult Function( ReplaceNotesResultDismissedEvent value)?  resultDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ReplaceNotesFindChangedEvent() when findChanged != null:
return findChanged(_that);case ReplaceNotesReplacementChangedEvent() when replacementChanged != null:
return replacementChanged(_that);case ReplaceNotesOptionToggledEvent() when optionToggled != null:
return optionToggled(_that);case ReplaceNotesPreviewDueEvent() when previewDue != null:
return previewDue(_that);case ReplaceNotesNoteToggledEvent() when noteToggled != null:
return noteToggled(_that);case ReplaceNotesApplyRequestedEvent() when applyRequested != null:
return applyRequested(_that);case ReplaceNotesUndoRequestedEvent() when undoRequested != null:
return undoRequested(_that);case ReplaceNotesResultDismissedEvent() when resultDismissed != null:
return resultDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ReplaceNotesFindChangedEvent value)  findChanged,required TResult Function( ReplaceNotesReplacementChangedEvent value)  replacementChanged,required TResult Function( ReplaceNotesOptionToggledEvent value)  optionToggled,required TResult Function( ReplaceNotesPreviewDueEvent value)  previewDue,required TResult Function( ReplaceNotesNoteToggledEvent value)  noteToggled,required TResult Function( ReplaceNotesApplyRequestedEvent value)  applyRequested,required TResult Function( ReplaceNotesUndoRequestedEvent value)  undoRequested,required TResult Function( ReplaceNotesResultDismissedEvent value)  resultDismissed,}){
final _that = this;
switch (_that) {
case ReplaceNotesFindChangedEvent():
return findChanged(_that);case ReplaceNotesReplacementChangedEvent():
return replacementChanged(_that);case ReplaceNotesOptionToggledEvent():
return optionToggled(_that);case ReplaceNotesPreviewDueEvent():
return previewDue(_that);case ReplaceNotesNoteToggledEvent():
return noteToggled(_that);case ReplaceNotesApplyRequestedEvent():
return applyRequested(_that);case ReplaceNotesUndoRequestedEvent():
return undoRequested(_that);case ReplaceNotesResultDismissedEvent():
return resultDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ReplaceNotesFindChangedEvent value)?  findChanged,TResult? Function( ReplaceNotesReplacementChangedEvent value)?  replacementChanged,TResult? Function( ReplaceNotesOptionToggledEvent value)?  optionToggled,TResult? Function( ReplaceNotesPreviewDueEvent value)?  previewDue,TResult? Function( ReplaceNotesNoteToggledEvent value)?  noteToggled,TResult? Function( ReplaceNotesApplyRequestedEvent value)?  applyRequested,TResult? Function( ReplaceNotesUndoRequestedEvent value)?  undoRequested,TResult? Function( ReplaceNotesResultDismissedEvent value)?  resultDismissed,}){
final _that = this;
switch (_that) {
case ReplaceNotesFindChangedEvent() when findChanged != null:
return findChanged(_that);case ReplaceNotesReplacementChangedEvent() when replacementChanged != null:
return replacementChanged(_that);case ReplaceNotesOptionToggledEvent() when optionToggled != null:
return optionToggled(_that);case ReplaceNotesPreviewDueEvent() when previewDue != null:
return previewDue(_that);case ReplaceNotesNoteToggledEvent() when noteToggled != null:
return noteToggled(_that);case ReplaceNotesApplyRequestedEvent() when applyRequested != null:
return applyRequested(_that);case ReplaceNotesUndoRequestedEvent() when undoRequested != null:
return undoRequested(_that);case ReplaceNotesResultDismissedEvent() when resultDismissed != null:
return resultDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String text)?  findChanged,TResult Function( String text)?  replacementChanged,TResult Function( ReplaceOption option)?  optionToggled,TResult Function()?  previewDue,TResult Function( String noteId)?  noteToggled,TResult Function()?  applyRequested,TResult Function()?  undoRequested,TResult Function()?  resultDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ReplaceNotesFindChangedEvent() when findChanged != null:
return findChanged(_that.text);case ReplaceNotesReplacementChangedEvent() when replacementChanged != null:
return replacementChanged(_that.text);case ReplaceNotesOptionToggledEvent() when optionToggled != null:
return optionToggled(_that.option);case ReplaceNotesPreviewDueEvent() when previewDue != null:
return previewDue();case ReplaceNotesNoteToggledEvent() when noteToggled != null:
return noteToggled(_that.noteId);case ReplaceNotesApplyRequestedEvent() when applyRequested != null:
return applyRequested();case ReplaceNotesUndoRequestedEvent() when undoRequested != null:
return undoRequested();case ReplaceNotesResultDismissedEvent() when resultDismissed != null:
return resultDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String text)  findChanged,required TResult Function( String text)  replacementChanged,required TResult Function( ReplaceOption option)  optionToggled,required TResult Function()  previewDue,required TResult Function( String noteId)  noteToggled,required TResult Function()  applyRequested,required TResult Function()  undoRequested,required TResult Function()  resultDismissed,}) {final _that = this;
switch (_that) {
case ReplaceNotesFindChangedEvent():
return findChanged(_that.text);case ReplaceNotesReplacementChangedEvent():
return replacementChanged(_that.text);case ReplaceNotesOptionToggledEvent():
return optionToggled(_that.option);case ReplaceNotesPreviewDueEvent():
return previewDue();case ReplaceNotesNoteToggledEvent():
return noteToggled(_that.noteId);case ReplaceNotesApplyRequestedEvent():
return applyRequested();case ReplaceNotesUndoRequestedEvent():
return undoRequested();case ReplaceNotesResultDismissedEvent():
return resultDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String text)?  findChanged,TResult? Function( String text)?  replacementChanged,TResult? Function( ReplaceOption option)?  optionToggled,TResult? Function()?  previewDue,TResult? Function( String noteId)?  noteToggled,TResult? Function()?  applyRequested,TResult? Function()?  undoRequested,TResult? Function()?  resultDismissed,}) {final _that = this;
switch (_that) {
case ReplaceNotesFindChangedEvent() when findChanged != null:
return findChanged(_that.text);case ReplaceNotesReplacementChangedEvent() when replacementChanged != null:
return replacementChanged(_that.text);case ReplaceNotesOptionToggledEvent() when optionToggled != null:
return optionToggled(_that.option);case ReplaceNotesPreviewDueEvent() when previewDue != null:
return previewDue();case ReplaceNotesNoteToggledEvent() when noteToggled != null:
return noteToggled(_that.noteId);case ReplaceNotesApplyRequestedEvent() when applyRequested != null:
return applyRequested();case ReplaceNotesUndoRequestedEvent() when undoRequested != null:
return undoRequested();case ReplaceNotesResultDismissedEvent() when resultDismissed != null:
return resultDismissed();case _:
  return null;

}
}

}

/// @nodoc


class ReplaceNotesFindChangedEvent implements ReplaceNotesEvent {
  const ReplaceNotesFindChangedEvent(this.text);
  

 final  String text;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReplaceNotesFindChangedEventCopyWith<ReplaceNotesFindChangedEvent> get copyWith => _$ReplaceNotesFindChangedEventCopyWithImpl<ReplaceNotesFindChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesFindChangedEvent&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'ReplaceNotesEvent.findChanged(text: $text)';
}


}

/// @nodoc
abstract mixin class $ReplaceNotesFindChangedEventCopyWith<$Res> implements $ReplaceNotesEventCopyWith<$Res> {
  factory $ReplaceNotesFindChangedEventCopyWith(ReplaceNotesFindChangedEvent value, $Res Function(ReplaceNotesFindChangedEvent) _then) = _$ReplaceNotesFindChangedEventCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$ReplaceNotesFindChangedEventCopyWithImpl<$Res>
    implements $ReplaceNotesFindChangedEventCopyWith<$Res> {
  _$ReplaceNotesFindChangedEventCopyWithImpl(this._self, this._then);

  final ReplaceNotesFindChangedEvent _self;
  final $Res Function(ReplaceNotesFindChangedEvent) _then;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(ReplaceNotesFindChangedEvent(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ReplaceNotesReplacementChangedEvent implements ReplaceNotesEvent {
  const ReplaceNotesReplacementChangedEvent(this.text);
  

 final  String text;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReplaceNotesReplacementChangedEventCopyWith<ReplaceNotesReplacementChangedEvent> get copyWith => _$ReplaceNotesReplacementChangedEventCopyWithImpl<ReplaceNotesReplacementChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesReplacementChangedEvent&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'ReplaceNotesEvent.replacementChanged(text: $text)';
}


}

/// @nodoc
abstract mixin class $ReplaceNotesReplacementChangedEventCopyWith<$Res> implements $ReplaceNotesEventCopyWith<$Res> {
  factory $ReplaceNotesReplacementChangedEventCopyWith(ReplaceNotesReplacementChangedEvent value, $Res Function(ReplaceNotesReplacementChangedEvent) _then) = _$ReplaceNotesReplacementChangedEventCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$ReplaceNotesReplacementChangedEventCopyWithImpl<$Res>
    implements $ReplaceNotesReplacementChangedEventCopyWith<$Res> {
  _$ReplaceNotesReplacementChangedEventCopyWithImpl(this._self, this._then);

  final ReplaceNotesReplacementChangedEvent _self;
  final $Res Function(ReplaceNotesReplacementChangedEvent) _then;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(ReplaceNotesReplacementChangedEvent(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ReplaceNotesOptionToggledEvent implements ReplaceNotesEvent {
  const ReplaceNotesOptionToggledEvent(this.option);
  

 final  ReplaceOption option;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReplaceNotesOptionToggledEventCopyWith<ReplaceNotesOptionToggledEvent> get copyWith => _$ReplaceNotesOptionToggledEventCopyWithImpl<ReplaceNotesOptionToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesOptionToggledEvent&&(identical(other.option, option) || other.option == option));
}


@override
int get hashCode => Object.hash(runtimeType,option);

@override
String toString() {
  return 'ReplaceNotesEvent.optionToggled(option: $option)';
}


}

/// @nodoc
abstract mixin class $ReplaceNotesOptionToggledEventCopyWith<$Res> implements $ReplaceNotesEventCopyWith<$Res> {
  factory $ReplaceNotesOptionToggledEventCopyWith(ReplaceNotesOptionToggledEvent value, $Res Function(ReplaceNotesOptionToggledEvent) _then) = _$ReplaceNotesOptionToggledEventCopyWithImpl;
@useResult
$Res call({
 ReplaceOption option
});




}
/// @nodoc
class _$ReplaceNotesOptionToggledEventCopyWithImpl<$Res>
    implements $ReplaceNotesOptionToggledEventCopyWith<$Res> {
  _$ReplaceNotesOptionToggledEventCopyWithImpl(this._self, this._then);

  final ReplaceNotesOptionToggledEvent _self;
  final $Res Function(ReplaceNotesOptionToggledEvent) _then;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? option = null,}) {
  return _then(ReplaceNotesOptionToggledEvent(
null == option ? _self.option : option // ignore: cast_nullable_to_non_nullable
as ReplaceOption,
  ));
}


}

/// @nodoc


class ReplaceNotesPreviewDueEvent implements ReplaceNotesEvent {
  const ReplaceNotesPreviewDueEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesPreviewDueEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ReplaceNotesEvent.previewDue()';
}


}




/// @nodoc


class ReplaceNotesNoteToggledEvent implements ReplaceNotesEvent {
  const ReplaceNotesNoteToggledEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReplaceNotesNoteToggledEventCopyWith<ReplaceNotesNoteToggledEvent> get copyWith => _$ReplaceNotesNoteToggledEventCopyWithImpl<ReplaceNotesNoteToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesNoteToggledEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'ReplaceNotesEvent.noteToggled(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $ReplaceNotesNoteToggledEventCopyWith<$Res> implements $ReplaceNotesEventCopyWith<$Res> {
  factory $ReplaceNotesNoteToggledEventCopyWith(ReplaceNotesNoteToggledEvent value, $Res Function(ReplaceNotesNoteToggledEvent) _then) = _$ReplaceNotesNoteToggledEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$ReplaceNotesNoteToggledEventCopyWithImpl<$Res>
    implements $ReplaceNotesNoteToggledEventCopyWith<$Res> {
  _$ReplaceNotesNoteToggledEventCopyWithImpl(this._self, this._then);

  final ReplaceNotesNoteToggledEvent _self;
  final $Res Function(ReplaceNotesNoteToggledEvent) _then;

/// Create a copy of ReplaceNotesEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(ReplaceNotesNoteToggledEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ReplaceNotesApplyRequestedEvent implements ReplaceNotesEvent {
  const ReplaceNotesApplyRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesApplyRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ReplaceNotesEvent.applyRequested()';
}


}




/// @nodoc


class ReplaceNotesUndoRequestedEvent implements ReplaceNotesEvent {
  const ReplaceNotesUndoRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesUndoRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ReplaceNotesEvent.undoRequested()';
}


}




/// @nodoc


class ReplaceNotesResultDismissedEvent implements ReplaceNotesEvent {
  const ReplaceNotesResultDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesResultDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ReplaceNotesEvent.resultDismissed()';
}


}




/// @nodoc
mixin _$ReplaceNotesState {

/// What to find and the toggles.
 ReplaceOptions get options;/// What to put instead.
 String get replacement; ReplacePreviewStatus get previewStatus;/// Every match, note by note; null until there is text to find.
 ReplacePreview? get preview;/// The notes left out of the replace, by id.
 Set<String> get excluded;/// The replace is running.
 bool get applying;/// What the last replace did, to say and to undo.
 ReplaceOutcome? get outcome;/// The undo is running.
 bool get undoing;/// What undoing the last replace did.
 ReplaceUndone? get undone;/// Why the last replace or undo did not happen.
 ReplaceProblem? get problem;
/// Create a copy of ReplaceNotesState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReplaceNotesStateCopyWith<ReplaceNotesState> get copyWith => _$ReplaceNotesStateCopyWithImpl<ReplaceNotesState>(this as ReplaceNotesState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReplaceNotesState&&(identical(other.options, options) || other.options == options)&&(identical(other.replacement, replacement) || other.replacement == replacement)&&(identical(other.previewStatus, previewStatus) || other.previewStatus == previewStatus)&&(identical(other.preview, preview) || other.preview == preview)&&const DeepCollectionEquality().equals(other.excluded, excluded)&&(identical(other.applying, applying) || other.applying == applying)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.undoing, undoing) || other.undoing == undoing)&&(identical(other.undone, undone) || other.undone == undone)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,options,replacement,previewStatus,preview,const DeepCollectionEquality().hash(excluded),applying,outcome,undoing,undone,problem);

@override
String toString() {
  return 'ReplaceNotesState(options: $options, replacement: $replacement, previewStatus: $previewStatus, preview: $preview, excluded: $excluded, applying: $applying, outcome: $outcome, undoing: $undoing, undone: $undone, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $ReplaceNotesStateCopyWith<$Res>  {
  factory $ReplaceNotesStateCopyWith(ReplaceNotesState value, $Res Function(ReplaceNotesState) _then) = _$ReplaceNotesStateCopyWithImpl;
@useResult
$Res call({
 ReplaceOptions options, String replacement, ReplacePreviewStatus previewStatus, ReplacePreview? preview, Set<String> excluded, bool applying, ReplaceOutcome? outcome, bool undoing, ReplaceUndone? undone, ReplaceProblem? problem
});




}
/// @nodoc
class _$ReplaceNotesStateCopyWithImpl<$Res>
    implements $ReplaceNotesStateCopyWith<$Res> {
  _$ReplaceNotesStateCopyWithImpl(this._self, this._then);

  final ReplaceNotesState _self;
  final $Res Function(ReplaceNotesState) _then;

/// Create a copy of ReplaceNotesState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? options = null,Object? replacement = null,Object? previewStatus = null,Object? preview = freezed,Object? excluded = null,Object? applying = null,Object? outcome = freezed,Object? undoing = null,Object? undone = freezed,Object? problem = freezed,}) {
  return _then(_self.copyWith(
options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as ReplaceOptions,replacement: null == replacement ? _self.replacement : replacement // ignore: cast_nullable_to_non_nullable
as String,previewStatus: null == previewStatus ? _self.previewStatus : previewStatus // ignore: cast_nullable_to_non_nullable
as ReplacePreviewStatus,preview: freezed == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as ReplacePreview?,excluded: null == excluded ? _self.excluded : excluded // ignore: cast_nullable_to_non_nullable
as Set<String>,applying: null == applying ? _self.applying : applying // ignore: cast_nullable_to_non_nullable
as bool,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as ReplaceOutcome?,undoing: null == undoing ? _self.undoing : undoing // ignore: cast_nullable_to_non_nullable
as bool,undone: freezed == undone ? _self.undone : undone // ignore: cast_nullable_to_non_nullable
as ReplaceUndone?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ReplaceProblem?,
  ));
}

}


/// Adds pattern-matching-related methods to [ReplaceNotesState].
extension ReplaceNotesStatePatterns on ReplaceNotesState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ReplaceNotesState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ReplaceNotesState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ReplaceNotesState value)  $default,){
final _that = this;
switch (_that) {
case _ReplaceNotesState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ReplaceNotesState value)?  $default,){
final _that = this;
switch (_that) {
case _ReplaceNotesState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ReplaceOptions options,  String replacement,  ReplacePreviewStatus previewStatus,  ReplacePreview? preview,  Set<String> excluded,  bool applying,  ReplaceOutcome? outcome,  bool undoing,  ReplaceUndone? undone,  ReplaceProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ReplaceNotesState() when $default != null:
return $default(_that.options,_that.replacement,_that.previewStatus,_that.preview,_that.excluded,_that.applying,_that.outcome,_that.undoing,_that.undone,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ReplaceOptions options,  String replacement,  ReplacePreviewStatus previewStatus,  ReplacePreview? preview,  Set<String> excluded,  bool applying,  ReplaceOutcome? outcome,  bool undoing,  ReplaceUndone? undone,  ReplaceProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _ReplaceNotesState():
return $default(_that.options,_that.replacement,_that.previewStatus,_that.preview,_that.excluded,_that.applying,_that.outcome,_that.undoing,_that.undone,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ReplaceOptions options,  String replacement,  ReplacePreviewStatus previewStatus,  ReplacePreview? preview,  Set<String> excluded,  bool applying,  ReplaceOutcome? outcome,  bool undoing,  ReplaceUndone? undone,  ReplaceProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _ReplaceNotesState() when $default != null:
return $default(_that.options,_that.replacement,_that.previewStatus,_that.preview,_that.excluded,_that.applying,_that.outcome,_that.undoing,_that.undone,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _ReplaceNotesState extends ReplaceNotesState {
  const _ReplaceNotesState({this.options = const ReplaceOptions(find: '', matchCase: false, wholeWord: false, regex: false, skipCode: false, includeTrash: false), this.replacement = '', this.previewStatus = ReplacePreviewStatus.idle, this.preview, final  Set<String> excluded = const <String>{}, this.applying = false, this.outcome, this.undoing = false, this.undone, this.problem}): _excluded = excluded,super._();
  

/// What to find and the toggles.
@override@JsonKey() final  ReplaceOptions options;
/// What to put instead.
@override@JsonKey() final  String replacement;
@override@JsonKey() final  ReplacePreviewStatus previewStatus;
/// Every match, note by note; null until there is text to find.
@override final  ReplacePreview? preview;
/// The notes left out of the replace, by id.
 final  Set<String> _excluded;
/// The notes left out of the replace, by id.
@override@JsonKey() Set<String> get excluded {
  if (_excluded is EqualUnmodifiableSetView) return _excluded;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_excluded);
}

/// The replace is running.
@override@JsonKey() final  bool applying;
/// What the last replace did, to say and to undo.
@override final  ReplaceOutcome? outcome;
/// The undo is running.
@override@JsonKey() final  bool undoing;
/// What undoing the last replace did.
@override final  ReplaceUndone? undone;
/// Why the last replace or undo did not happen.
@override final  ReplaceProblem? problem;

/// Create a copy of ReplaceNotesState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ReplaceNotesStateCopyWith<_ReplaceNotesState> get copyWith => __$ReplaceNotesStateCopyWithImpl<_ReplaceNotesState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ReplaceNotesState&&(identical(other.options, options) || other.options == options)&&(identical(other.replacement, replacement) || other.replacement == replacement)&&(identical(other.previewStatus, previewStatus) || other.previewStatus == previewStatus)&&(identical(other.preview, preview) || other.preview == preview)&&const DeepCollectionEquality().equals(other._excluded, _excluded)&&(identical(other.applying, applying) || other.applying == applying)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.undoing, undoing) || other.undoing == undoing)&&(identical(other.undone, undone) || other.undone == undone)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,options,replacement,previewStatus,preview,const DeepCollectionEquality().hash(_excluded),applying,outcome,undoing,undone,problem);

@override
String toString() {
  return 'ReplaceNotesState(options: $options, replacement: $replacement, previewStatus: $previewStatus, preview: $preview, excluded: $excluded, applying: $applying, outcome: $outcome, undoing: $undoing, undone: $undone, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$ReplaceNotesStateCopyWith<$Res> implements $ReplaceNotesStateCopyWith<$Res> {
  factory _$ReplaceNotesStateCopyWith(_ReplaceNotesState value, $Res Function(_ReplaceNotesState) _then) = __$ReplaceNotesStateCopyWithImpl;
@override @useResult
$Res call({
 ReplaceOptions options, String replacement, ReplacePreviewStatus previewStatus, ReplacePreview? preview, Set<String> excluded, bool applying, ReplaceOutcome? outcome, bool undoing, ReplaceUndone? undone, ReplaceProblem? problem
});




}
/// @nodoc
class __$ReplaceNotesStateCopyWithImpl<$Res>
    implements _$ReplaceNotesStateCopyWith<$Res> {
  __$ReplaceNotesStateCopyWithImpl(this._self, this._then);

  final _ReplaceNotesState _self;
  final $Res Function(_ReplaceNotesState) _then;

/// Create a copy of ReplaceNotesState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? options = null,Object? replacement = null,Object? previewStatus = null,Object? preview = freezed,Object? excluded = null,Object? applying = null,Object? outcome = freezed,Object? undoing = null,Object? undone = freezed,Object? problem = freezed,}) {
  return _then(_ReplaceNotesState(
options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as ReplaceOptions,replacement: null == replacement ? _self.replacement : replacement // ignore: cast_nullable_to_non_nullable
as String,previewStatus: null == previewStatus ? _self.previewStatus : previewStatus // ignore: cast_nullable_to_non_nullable
as ReplacePreviewStatus,preview: freezed == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as ReplacePreview?,excluded: null == excluded ? _self._excluded : excluded // ignore: cast_nullable_to_non_nullable
as Set<String>,applying: null == applying ? _self.applying : applying // ignore: cast_nullable_to_non_nullable
as bool,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as ReplaceOutcome?,undoing: null == undoing ? _self.undoing : undoing // ignore: cast_nullable_to_non_nullable
as bool,undone: freezed == undone ? _self.undone : undone // ignore: cast_nullable_to_non_nullable
as ReplaceUndone?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ReplaceProblem?,
  ));
}


}

// dart format on
