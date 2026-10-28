// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'assist_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AssistEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent()';
}


}

/// @nodoc
class $AssistEventCopyWith<$Res>  {
$AssistEventCopyWith(AssistEvent _, $Res Function(AssistEvent) __);
}


/// Adds pattern-matching-related methods to [AssistEvent].
extension AssistEventPatterns on AssistEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AssistRequestedEvent value)?  requested,TResult Function( AssistSummaryRequestedEvent value)?  summaryRequested,TResult Function( AssistUpdateReceivedEvent value)?  updateReceived,TResult Function( AssistStreamEndedEvent value)?  streamEnded,TResult Function( AssistStoppedEvent value)?  stopped,TResult Function( AssistChangeNoticedEvent value)?  changeNoticed,TResult Function( AssistDiscardedEvent value)?  discarded,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AssistRequestedEvent() when requested != null:
return requested(_that);case AssistSummaryRequestedEvent() when summaryRequested != null:
return summaryRequested(_that);case AssistUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that);case AssistStreamEndedEvent() when streamEnded != null:
return streamEnded(_that);case AssistStoppedEvent() when stopped != null:
return stopped(_that);case AssistChangeNoticedEvent() when changeNoticed != null:
return changeNoticed(_that);case AssistDiscardedEvent() when discarded != null:
return discarded(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AssistRequestedEvent value)  requested,required TResult Function( AssistSummaryRequestedEvent value)  summaryRequested,required TResult Function( AssistUpdateReceivedEvent value)  updateReceived,required TResult Function( AssistStreamEndedEvent value)  streamEnded,required TResult Function( AssistStoppedEvent value)  stopped,required TResult Function( AssistChangeNoticedEvent value)  changeNoticed,required TResult Function( AssistDiscardedEvent value)  discarded,}){
final _that = this;
switch (_that) {
case AssistRequestedEvent():
return requested(_that);case AssistSummaryRequestedEvent():
return summaryRequested(_that);case AssistUpdateReceivedEvent():
return updateReceived(_that);case AssistStreamEndedEvent():
return streamEnded(_that);case AssistStoppedEvent():
return stopped(_that);case AssistChangeNoticedEvent():
return changeNoticed(_that);case AssistDiscardedEvent():
return discarded(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AssistRequestedEvent value)?  requested,TResult? Function( AssistSummaryRequestedEvent value)?  summaryRequested,TResult? Function( AssistUpdateReceivedEvent value)?  updateReceived,TResult? Function( AssistStreamEndedEvent value)?  streamEnded,TResult? Function( AssistStoppedEvent value)?  stopped,TResult? Function( AssistChangeNoticedEvent value)?  changeNoticed,TResult? Function( AssistDiscardedEvent value)?  discarded,}){
final _that = this;
switch (_that) {
case AssistRequestedEvent() when requested != null:
return requested(_that);case AssistSummaryRequestedEvent() when summaryRequested != null:
return summaryRequested(_that);case AssistUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that);case AssistStreamEndedEvent() when streamEnded != null:
return streamEnded(_that);case AssistStoppedEvent() when stopped != null:
return stopped(_that);case AssistChangeNoticedEvent() when changeNoticed != null:
return changeNoticed(_that);case AssistDiscardedEvent() when discarded != null:
return discarded(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId,  AssistAction action,  String before,  String selection)?  requested,TResult Function( String noteId)?  summaryRequested,TResult Function( AssistUpdate update)?  updateReceived,TResult Function()?  streamEnded,TResult Function()?  stopped,TResult Function()?  changeNoticed,TResult Function()?  discarded,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AssistRequestedEvent() when requested != null:
return requested(_that.noteId,_that.action,_that.before,_that.selection);case AssistSummaryRequestedEvent() when summaryRequested != null:
return summaryRequested(_that.noteId);case AssistUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that.update);case AssistStreamEndedEvent() when streamEnded != null:
return streamEnded();case AssistStoppedEvent() when stopped != null:
return stopped();case AssistChangeNoticedEvent() when changeNoticed != null:
return changeNoticed();case AssistDiscardedEvent() when discarded != null:
return discarded();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId,  AssistAction action,  String before,  String selection)  requested,required TResult Function( String noteId)  summaryRequested,required TResult Function( AssistUpdate update)  updateReceived,required TResult Function()  streamEnded,required TResult Function()  stopped,required TResult Function()  changeNoticed,required TResult Function()  discarded,}) {final _that = this;
switch (_that) {
case AssistRequestedEvent():
return requested(_that.noteId,_that.action,_that.before,_that.selection);case AssistSummaryRequestedEvent():
return summaryRequested(_that.noteId);case AssistUpdateReceivedEvent():
return updateReceived(_that.update);case AssistStreamEndedEvent():
return streamEnded();case AssistStoppedEvent():
return stopped();case AssistChangeNoticedEvent():
return changeNoticed();case AssistDiscardedEvent():
return discarded();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId,  AssistAction action,  String before,  String selection)?  requested,TResult? Function( String noteId)?  summaryRequested,TResult? Function( AssistUpdate update)?  updateReceived,TResult? Function()?  streamEnded,TResult? Function()?  stopped,TResult? Function()?  changeNoticed,TResult? Function()?  discarded,}) {final _that = this;
switch (_that) {
case AssistRequestedEvent() when requested != null:
return requested(_that.noteId,_that.action,_that.before,_that.selection);case AssistSummaryRequestedEvent() when summaryRequested != null:
return summaryRequested(_that.noteId);case AssistUpdateReceivedEvent() when updateReceived != null:
return updateReceived(_that.update);case AssistStreamEndedEvent() when streamEnded != null:
return streamEnded();case AssistStoppedEvent() when stopped != null:
return stopped();case AssistChangeNoticedEvent() when changeNoticed != null:
return changeNoticed();case AssistDiscardedEvent() when discarded != null:
return discarded();case _:
  return null;

}
}

}

/// @nodoc


class AssistRequestedEvent implements AssistEvent {
  const AssistRequestedEvent({required this.noteId, required this.action, required this.before, required this.selection});
  

 final  String noteId;
 final  AssistAction action;
 final  String before;
 final  String selection;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistRequestedEventCopyWith<AssistRequestedEvent> get copyWith => _$AssistRequestedEventCopyWithImpl<AssistRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.action, action) || other.action == action)&&(identical(other.before, before) || other.before == before)&&(identical(other.selection, selection) || other.selection == selection));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,action,before,selection);

@override
String toString() {
  return 'AssistEvent.requested(noteId: $noteId, action: $action, before: $before, selection: $selection)';
}


}

/// @nodoc
abstract mixin class $AssistRequestedEventCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistRequestedEventCopyWith(AssistRequestedEvent value, $Res Function(AssistRequestedEvent) _then) = _$AssistRequestedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, AssistAction action, String before, String selection
});




}
/// @nodoc
class _$AssistRequestedEventCopyWithImpl<$Res>
    implements $AssistRequestedEventCopyWith<$Res> {
  _$AssistRequestedEventCopyWithImpl(this._self, this._then);

  final AssistRequestedEvent _self;
  final $Res Function(AssistRequestedEvent) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? action = null,Object? before = null,Object? selection = null,}) {
  return _then(AssistRequestedEvent(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction,before: null == before ? _self.before : before // ignore: cast_nullable_to_non_nullable
as String,selection: null == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistSummaryRequestedEvent implements AssistEvent {
  const AssistSummaryRequestedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistSummaryRequestedEventCopyWith<AssistSummaryRequestedEvent> get copyWith => _$AssistSummaryRequestedEventCopyWithImpl<AssistSummaryRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistSummaryRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'AssistEvent.summaryRequested(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $AssistSummaryRequestedEventCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistSummaryRequestedEventCopyWith(AssistSummaryRequestedEvent value, $Res Function(AssistSummaryRequestedEvent) _then) = _$AssistSummaryRequestedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$AssistSummaryRequestedEventCopyWithImpl<$Res>
    implements $AssistSummaryRequestedEventCopyWith<$Res> {
  _$AssistSummaryRequestedEventCopyWithImpl(this._self, this._then);

  final AssistSummaryRequestedEvent _self;
  final $Res Function(AssistSummaryRequestedEvent) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(AssistSummaryRequestedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistUpdateReceivedEvent implements AssistEvent {
  const AssistUpdateReceivedEvent(this.update);
  

 final  AssistUpdate update;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistUpdateReceivedEventCopyWith<AssistUpdateReceivedEvent> get copyWith => _$AssistUpdateReceivedEventCopyWithImpl<AssistUpdateReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistUpdateReceivedEvent&&(identical(other.update, update) || other.update == update));
}


@override
int get hashCode => Object.hash(runtimeType,update);

@override
String toString() {
  return 'AssistEvent.updateReceived(update: $update)';
}


}

/// @nodoc
abstract mixin class $AssistUpdateReceivedEventCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistUpdateReceivedEventCopyWith(AssistUpdateReceivedEvent value, $Res Function(AssistUpdateReceivedEvent) _then) = _$AssistUpdateReceivedEventCopyWithImpl;
@useResult
$Res call({
 AssistUpdate update
});


$AssistUpdateCopyWith<$Res> get update;

}
/// @nodoc
class _$AssistUpdateReceivedEventCopyWithImpl<$Res>
    implements $AssistUpdateReceivedEventCopyWith<$Res> {
  _$AssistUpdateReceivedEventCopyWithImpl(this._self, this._then);

  final AssistUpdateReceivedEvent _self;
  final $Res Function(AssistUpdateReceivedEvent) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? update = null,}) {
  return _then(AssistUpdateReceivedEvent(
null == update ? _self.update : update // ignore: cast_nullable_to_non_nullable
as AssistUpdate,
  ));
}

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AssistUpdateCopyWith<$Res> get update {
  
  return $AssistUpdateCopyWith<$Res>(_self.update, (value) {
    return _then(_self.copyWith(update: value));
  });
}
}

/// @nodoc


class AssistStreamEndedEvent implements AssistEvent {
  const AssistStreamEndedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistStreamEndedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent.streamEnded()';
}


}




/// @nodoc


class AssistStoppedEvent implements AssistEvent {
  const AssistStoppedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistStoppedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent.stopped()';
}


}




/// @nodoc


class AssistChangeNoticedEvent implements AssistEvent {
  const AssistChangeNoticedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistChangeNoticedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent.changeNoticed()';
}


}




/// @nodoc


class AssistDiscardedEvent implements AssistEvent {
  const AssistDiscardedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistDiscardedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent.discarded()';
}


}




/// @nodoc
mixin _$AssistState {

 AssistStatus get status;/// What was asked for; null before the person has chosen.
 AssistAction? get action;/// Whether the suggestion is for the whole note (its summary) and not
/// for text selected in it.
 bool get wholeNote;/// The suggestion so far, or whole once [status] is ready.
 String get suggestion;/// Whether the suggestion was not applied because the note's text changed
/// since it was asked for.
 bool get changed;/// Why the request failed, in plain words, while [status] is failed.
 String? get failure;
/// Create a copy of AssistState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistStateCopyWith<AssistState> get copyWith => _$AssistStateCopyWithImpl<AssistState>(this as AssistState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistState&&(identical(other.status, status) || other.status == status)&&(identical(other.action, action) || other.action == action)&&(identical(other.wholeNote, wholeNote) || other.wholeNote == wholeNote)&&(identical(other.suggestion, suggestion) || other.suggestion == suggestion)&&(identical(other.changed, changed) || other.changed == changed)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,status,action,wholeNote,suggestion,changed,failure);

@override
String toString() {
  return 'AssistState(status: $status, action: $action, wholeNote: $wholeNote, suggestion: $suggestion, changed: $changed, failure: $failure)';
}


}

/// @nodoc
abstract mixin class $AssistStateCopyWith<$Res>  {
  factory $AssistStateCopyWith(AssistState value, $Res Function(AssistState) _then) = _$AssistStateCopyWithImpl;
@useResult
$Res call({
 AssistStatus status, AssistAction? action, bool wholeNote, String suggestion, bool changed, String? failure
});




}
/// @nodoc
class _$AssistStateCopyWithImpl<$Res>
    implements $AssistStateCopyWith<$Res> {
  _$AssistStateCopyWithImpl(this._self, this._then);

  final AssistState _self;
  final $Res Function(AssistState) _then;

/// Create a copy of AssistState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? action = freezed,Object? wholeNote = null,Object? suggestion = null,Object? changed = null,Object? failure = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as AssistStatus,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction?,wholeNote: null == wholeNote ? _self.wholeNote : wholeNote // ignore: cast_nullable_to_non_nullable
as bool,suggestion: null == suggestion ? _self.suggestion : suggestion // ignore: cast_nullable_to_non_nullable
as String,changed: null == changed ? _self.changed : changed // ignore: cast_nullable_to_non_nullable
as bool,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [AssistState].
extension AssistStatePatterns on AssistState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AssistState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AssistState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AssistState value)  $default,){
final _that = this;
switch (_that) {
case _AssistState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AssistState value)?  $default,){
final _that = this;
switch (_that) {
case _AssistState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( AssistStatus status,  AssistAction? action,  bool wholeNote,  String suggestion,  bool changed,  String? failure)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AssistState() when $default != null:
return $default(_that.status,_that.action,_that.wholeNote,_that.suggestion,_that.changed,_that.failure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( AssistStatus status,  AssistAction? action,  bool wholeNote,  String suggestion,  bool changed,  String? failure)  $default,) {final _that = this;
switch (_that) {
case _AssistState():
return $default(_that.status,_that.action,_that.wholeNote,_that.suggestion,_that.changed,_that.failure);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( AssistStatus status,  AssistAction? action,  bool wholeNote,  String suggestion,  bool changed,  String? failure)?  $default,) {final _that = this;
switch (_that) {
case _AssistState() when $default != null:
return $default(_that.status,_that.action,_that.wholeNote,_that.suggestion,_that.changed,_that.failure);case _:
  return null;

}
}

}

/// @nodoc


class _AssistState implements AssistState {
  const _AssistState({this.status = AssistStatus.idle, this.action, this.wholeNote = false, this.suggestion = '', this.changed = false, this.failure});
  

@override@JsonKey() final  AssistStatus status;
/// What was asked for; null before the person has chosen.
@override final  AssistAction? action;
/// Whether the suggestion is for the whole note (its summary) and not
/// for text selected in it.
@override@JsonKey() final  bool wholeNote;
/// The suggestion so far, or whole once [status] is ready.
@override@JsonKey() final  String suggestion;
/// Whether the suggestion was not applied because the note's text changed
/// since it was asked for.
@override@JsonKey() final  bool changed;
/// Why the request failed, in plain words, while [status] is failed.
@override final  String? failure;

/// Create a copy of AssistState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AssistStateCopyWith<_AssistState> get copyWith => __$AssistStateCopyWithImpl<_AssistState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AssistState&&(identical(other.status, status) || other.status == status)&&(identical(other.action, action) || other.action == action)&&(identical(other.wholeNote, wholeNote) || other.wholeNote == wholeNote)&&(identical(other.suggestion, suggestion) || other.suggestion == suggestion)&&(identical(other.changed, changed) || other.changed == changed)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,status,action,wholeNote,suggestion,changed,failure);

@override
String toString() {
  return 'AssistState(status: $status, action: $action, wholeNote: $wholeNote, suggestion: $suggestion, changed: $changed, failure: $failure)';
}


}

/// @nodoc
abstract mixin class _$AssistStateCopyWith<$Res> implements $AssistStateCopyWith<$Res> {
  factory _$AssistStateCopyWith(_AssistState value, $Res Function(_AssistState) _then) = __$AssistStateCopyWithImpl;
@override @useResult
$Res call({
 AssistStatus status, AssistAction? action, bool wholeNote, String suggestion, bool changed, String? failure
});




}
/// @nodoc
class __$AssistStateCopyWithImpl<$Res>
    implements _$AssistStateCopyWith<$Res> {
  __$AssistStateCopyWithImpl(this._self, this._then);

  final _AssistState _self;
  final $Res Function(_AssistState) _then;

/// Create a copy of AssistState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? action = freezed,Object? wholeNote = null,Object? suggestion = null,Object? changed = null,Object? failure = freezed,}) {
  return _then(_AssistState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as AssistStatus,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as AssistAction?,wholeNote: null == wholeNote ? _self.wholeNote : wholeNote // ignore: cast_nullable_to_non_nullable
as bool,suggestion: null == suggestion ? _self.suggestion : suggestion // ignore: cast_nullable_to_non_nullable
as String,changed: null == changed ? _self.changed : changed // ignore: cast_nullable_to_non_nullable
as bool,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
