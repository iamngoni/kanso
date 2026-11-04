// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'recording_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RecordingEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent()';
}


}

/// @nodoc
class $RecordingEventCopyWith<$Res>  {
$RecordingEventCopyWith(RecordingEvent _, $Res Function(RecordingEvent) __);
}


/// Adds pattern-matching-related methods to [RecordingEvent].
extension RecordingEventPatterns on RecordingEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RecordingStartedEvent value)?  started,TResult Function( RecordingTickedEvent value)?  ticked,TResult Function( RecordingPausedEvent value)?  paused,TResult Function( RecordingResumedEvent value)?  resumed,TResult Function( RecordingStoppedEvent value)?  stopped,TResult Function( RecordingDiscardedEvent value)?  discarded,TResult Function( RecordingFoundCheckedEvent value)?  foundChecked,TResult Function( RecordingRecoveredEvent value)?  recovered,TResult Function( RecordingFoundDiscardedEvent value)?  foundDiscarded,TResult Function( RecordingNoticeClearedEvent value)?  noticeCleared,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RecordingStartedEvent() when started != null:
return started(_that);case RecordingTickedEvent() when ticked != null:
return ticked(_that);case RecordingPausedEvent() when paused != null:
return paused(_that);case RecordingResumedEvent() when resumed != null:
return resumed(_that);case RecordingStoppedEvent() when stopped != null:
return stopped(_that);case RecordingDiscardedEvent() when discarded != null:
return discarded(_that);case RecordingFoundCheckedEvent() when foundChecked != null:
return foundChecked(_that);case RecordingRecoveredEvent() when recovered != null:
return recovered(_that);case RecordingFoundDiscardedEvent() when foundDiscarded != null:
return foundDiscarded(_that);case RecordingNoticeClearedEvent() when noticeCleared != null:
return noticeCleared(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RecordingStartedEvent value)  started,required TResult Function( RecordingTickedEvent value)  ticked,required TResult Function( RecordingPausedEvent value)  paused,required TResult Function( RecordingResumedEvent value)  resumed,required TResult Function( RecordingStoppedEvent value)  stopped,required TResult Function( RecordingDiscardedEvent value)  discarded,required TResult Function( RecordingFoundCheckedEvent value)  foundChecked,required TResult Function( RecordingRecoveredEvent value)  recovered,required TResult Function( RecordingFoundDiscardedEvent value)  foundDiscarded,required TResult Function( RecordingNoticeClearedEvent value)  noticeCleared,}){
final _that = this;
switch (_that) {
case RecordingStartedEvent():
return started(_that);case RecordingTickedEvent():
return ticked(_that);case RecordingPausedEvent():
return paused(_that);case RecordingResumedEvent():
return resumed(_that);case RecordingStoppedEvent():
return stopped(_that);case RecordingDiscardedEvent():
return discarded(_that);case RecordingFoundCheckedEvent():
return foundChecked(_that);case RecordingRecoveredEvent():
return recovered(_that);case RecordingFoundDiscardedEvent():
return foundDiscarded(_that);case RecordingNoticeClearedEvent():
return noticeCleared(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RecordingStartedEvent value)?  started,TResult? Function( RecordingTickedEvent value)?  ticked,TResult? Function( RecordingPausedEvent value)?  paused,TResult? Function( RecordingResumedEvent value)?  resumed,TResult? Function( RecordingStoppedEvent value)?  stopped,TResult? Function( RecordingDiscardedEvent value)?  discarded,TResult? Function( RecordingFoundCheckedEvent value)?  foundChecked,TResult? Function( RecordingRecoveredEvent value)?  recovered,TResult? Function( RecordingFoundDiscardedEvent value)?  foundDiscarded,TResult? Function( RecordingNoticeClearedEvent value)?  noticeCleared,}){
final _that = this;
switch (_that) {
case RecordingStartedEvent() when started != null:
return started(_that);case RecordingTickedEvent() when ticked != null:
return ticked(_that);case RecordingPausedEvent() when paused != null:
return paused(_that);case RecordingResumedEvent() when resumed != null:
return resumed(_that);case RecordingStoppedEvent() when stopped != null:
return stopped(_that);case RecordingDiscardedEvent() when discarded != null:
return discarded(_that);case RecordingFoundCheckedEvent() when foundChecked != null:
return foundChecked(_that);case RecordingRecoveredEvent() when recovered != null:
return recovered(_that);case RecordingFoundDiscardedEvent() when foundDiscarded != null:
return foundDiscarded(_that);case RecordingNoticeClearedEvent() when noticeCleared != null:
return noticeCleared(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String? noteId)?  started,TResult Function()?  ticked,TResult Function()?  paused,TResult Function()?  resumed,TResult Function()?  stopped,TResult Function()?  discarded,TResult Function()?  foundChecked,TResult Function( String id)?  recovered,TResult Function( String id)?  foundDiscarded,TResult Function()?  noticeCleared,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RecordingStartedEvent() when started != null:
return started(_that.noteId);case RecordingTickedEvent() when ticked != null:
return ticked();case RecordingPausedEvent() when paused != null:
return paused();case RecordingResumedEvent() when resumed != null:
return resumed();case RecordingStoppedEvent() when stopped != null:
return stopped();case RecordingDiscardedEvent() when discarded != null:
return discarded();case RecordingFoundCheckedEvent() when foundChecked != null:
return foundChecked();case RecordingRecoveredEvent() when recovered != null:
return recovered(_that.id);case RecordingFoundDiscardedEvent() when foundDiscarded != null:
return foundDiscarded(_that.id);case RecordingNoticeClearedEvent() when noticeCleared != null:
return noticeCleared();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String? noteId)  started,required TResult Function()  ticked,required TResult Function()  paused,required TResult Function()  resumed,required TResult Function()  stopped,required TResult Function()  discarded,required TResult Function()  foundChecked,required TResult Function( String id)  recovered,required TResult Function( String id)  foundDiscarded,required TResult Function()  noticeCleared,}) {final _that = this;
switch (_that) {
case RecordingStartedEvent():
return started(_that.noteId);case RecordingTickedEvent():
return ticked();case RecordingPausedEvent():
return paused();case RecordingResumedEvent():
return resumed();case RecordingStoppedEvent():
return stopped();case RecordingDiscardedEvent():
return discarded();case RecordingFoundCheckedEvent():
return foundChecked();case RecordingRecoveredEvent():
return recovered(_that.id);case RecordingFoundDiscardedEvent():
return foundDiscarded(_that.id);case RecordingNoticeClearedEvent():
return noticeCleared();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String? noteId)?  started,TResult? Function()?  ticked,TResult? Function()?  paused,TResult? Function()?  resumed,TResult? Function()?  stopped,TResult? Function()?  discarded,TResult? Function()?  foundChecked,TResult? Function( String id)?  recovered,TResult? Function( String id)?  foundDiscarded,TResult? Function()?  noticeCleared,}) {final _that = this;
switch (_that) {
case RecordingStartedEvent() when started != null:
return started(_that.noteId);case RecordingTickedEvent() when ticked != null:
return ticked();case RecordingPausedEvent() when paused != null:
return paused();case RecordingResumedEvent() when resumed != null:
return resumed();case RecordingStoppedEvent() when stopped != null:
return stopped();case RecordingDiscardedEvent() when discarded != null:
return discarded();case RecordingFoundCheckedEvent() when foundChecked != null:
return foundChecked();case RecordingRecoveredEvent() when recovered != null:
return recovered(_that.id);case RecordingFoundDiscardedEvent() when foundDiscarded != null:
return foundDiscarded(_that.id);case RecordingNoticeClearedEvent() when noticeCleared != null:
return noticeCleared();case _:
  return null;

}
}

}

/// @nodoc


class RecordingStartedEvent implements RecordingEvent {
  const RecordingStartedEvent({this.noteId});
  

 final  String? noteId;

/// Create a copy of RecordingEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecordingStartedEventCopyWith<RecordingStartedEvent> get copyWith => _$RecordingStartedEventCopyWithImpl<RecordingStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'RecordingEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $RecordingStartedEventCopyWith<$Res> implements $RecordingEventCopyWith<$Res> {
  factory $RecordingStartedEventCopyWith(RecordingStartedEvent value, $Res Function(RecordingStartedEvent) _then) = _$RecordingStartedEventCopyWithImpl;
@useResult
$Res call({
 String? noteId
});




}
/// @nodoc
class _$RecordingStartedEventCopyWithImpl<$Res>
    implements $RecordingStartedEventCopyWith<$Res> {
  _$RecordingStartedEventCopyWithImpl(this._self, this._then);

  final RecordingStartedEvent _self;
  final $Res Function(RecordingStartedEvent) _then;

/// Create a copy of RecordingEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = freezed,}) {
  return _then(RecordingStartedEvent(
noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class RecordingTickedEvent implements RecordingEvent {
  const RecordingTickedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingTickedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.ticked()';
}


}




/// @nodoc


class RecordingPausedEvent implements RecordingEvent {
  const RecordingPausedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingPausedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.paused()';
}


}




/// @nodoc


class RecordingResumedEvent implements RecordingEvent {
  const RecordingResumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingResumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.resumed()';
}


}




/// @nodoc


class RecordingStoppedEvent implements RecordingEvent {
  const RecordingStoppedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingStoppedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.stopped()';
}


}




/// @nodoc


class RecordingDiscardedEvent implements RecordingEvent {
  const RecordingDiscardedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingDiscardedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.discarded()';
}


}




/// @nodoc


class RecordingFoundCheckedEvent implements RecordingEvent {
  const RecordingFoundCheckedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingFoundCheckedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.foundChecked()';
}


}




/// @nodoc


class RecordingRecoveredEvent implements RecordingEvent {
  const RecordingRecoveredEvent(this.id);
  

 final  String id;

/// Create a copy of RecordingEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecordingRecoveredEventCopyWith<RecordingRecoveredEvent> get copyWith => _$RecordingRecoveredEventCopyWithImpl<RecordingRecoveredEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingRecoveredEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'RecordingEvent.recovered(id: $id)';
}


}

/// @nodoc
abstract mixin class $RecordingRecoveredEventCopyWith<$Res> implements $RecordingEventCopyWith<$Res> {
  factory $RecordingRecoveredEventCopyWith(RecordingRecoveredEvent value, $Res Function(RecordingRecoveredEvent) _then) = _$RecordingRecoveredEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$RecordingRecoveredEventCopyWithImpl<$Res>
    implements $RecordingRecoveredEventCopyWith<$Res> {
  _$RecordingRecoveredEventCopyWithImpl(this._self, this._then);

  final RecordingRecoveredEvent _self;
  final $Res Function(RecordingRecoveredEvent) _then;

/// Create a copy of RecordingEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(RecordingRecoveredEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RecordingFoundDiscardedEvent implements RecordingEvent {
  const RecordingFoundDiscardedEvent(this.id);
  

 final  String id;

/// Create a copy of RecordingEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecordingFoundDiscardedEventCopyWith<RecordingFoundDiscardedEvent> get copyWith => _$RecordingFoundDiscardedEventCopyWithImpl<RecordingFoundDiscardedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingFoundDiscardedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'RecordingEvent.foundDiscarded(id: $id)';
}


}

/// @nodoc
abstract mixin class $RecordingFoundDiscardedEventCopyWith<$Res> implements $RecordingEventCopyWith<$Res> {
  factory $RecordingFoundDiscardedEventCopyWith(RecordingFoundDiscardedEvent value, $Res Function(RecordingFoundDiscardedEvent) _then) = _$RecordingFoundDiscardedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$RecordingFoundDiscardedEventCopyWithImpl<$Res>
    implements $RecordingFoundDiscardedEventCopyWith<$Res> {
  _$RecordingFoundDiscardedEventCopyWithImpl(this._self, this._then);

  final RecordingFoundDiscardedEvent _self;
  final $Res Function(RecordingFoundDiscardedEvent) _then;

/// Create a copy of RecordingEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(RecordingFoundDiscardedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class RecordingNoticeClearedEvent implements RecordingEvent {
  const RecordingNoticeClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingNoticeClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecordingEvent.noticeCleared()';
}


}




/// @nodoc
mixin _$RecordingState {

 RecordingPhase get phase;/// Audio time recorded, pause excluded.
 int get seconds;/// The level meter, 0 to 1.
 double get level; bool get paused;/// The 5-hour warning has been raised.
 bool get warned;/// The note being recorded into; null for a new note.
 String? get noteId;/// How the last recording ended by itself (cap, microphone, disk).
 RecordingEnd? get ended;/// The last recording's note, until the screen has shown it.
 RecordingSaved? get saved; RecordingProblem? get problem;/// Recordings a crash left.
 List<RecordingFound> get found;
/// Create a copy of RecordingState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecordingStateCopyWith<RecordingState> get copyWith => _$RecordingStateCopyWithImpl<RecordingState>(this as RecordingState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecordingState&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.seconds, seconds) || other.seconds == seconds)&&(identical(other.level, level) || other.level == level)&&(identical(other.paused, paused) || other.paused == paused)&&(identical(other.warned, warned) || other.warned == warned)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.ended, ended) || other.ended == ended)&&(identical(other.saved, saved) || other.saved == saved)&&(identical(other.problem, problem) || other.problem == problem)&&const DeepCollectionEquality().equals(other.found, found));
}


@override
int get hashCode => Object.hash(runtimeType,phase,seconds,level,paused,warned,noteId,ended,saved,problem,const DeepCollectionEquality().hash(found));

@override
String toString() {
  return 'RecordingState(phase: $phase, seconds: $seconds, level: $level, paused: $paused, warned: $warned, noteId: $noteId, ended: $ended, saved: $saved, problem: $problem, found: $found)';
}


}

/// @nodoc
abstract mixin class $RecordingStateCopyWith<$Res>  {
  factory $RecordingStateCopyWith(RecordingState value, $Res Function(RecordingState) _then) = _$RecordingStateCopyWithImpl;
@useResult
$Res call({
 RecordingPhase phase, int seconds, double level, bool paused, bool warned, String? noteId, RecordingEnd? ended, RecordingSaved? saved, RecordingProblem? problem, List<RecordingFound> found
});




}
/// @nodoc
class _$RecordingStateCopyWithImpl<$Res>
    implements $RecordingStateCopyWith<$Res> {
  _$RecordingStateCopyWithImpl(this._self, this._then);

  final RecordingState _self;
  final $Res Function(RecordingState) _then;

/// Create a copy of RecordingState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? phase = null,Object? seconds = null,Object? level = null,Object? paused = null,Object? warned = null,Object? noteId = freezed,Object? ended = freezed,Object? saved = freezed,Object? problem = freezed,Object? found = null,}) {
  return _then(_self.copyWith(
phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as RecordingPhase,seconds: null == seconds ? _self.seconds : seconds // ignore: cast_nullable_to_non_nullable
as int,level: null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as double,paused: null == paused ? _self.paused : paused // ignore: cast_nullable_to_non_nullable
as bool,warned: null == warned ? _self.warned : warned // ignore: cast_nullable_to_non_nullable
as bool,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,ended: freezed == ended ? _self.ended : ended // ignore: cast_nullable_to_non_nullable
as RecordingEnd?,saved: freezed == saved ? _self.saved : saved // ignore: cast_nullable_to_non_nullable
as RecordingSaved?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as RecordingProblem?,found: null == found ? _self.found : found // ignore: cast_nullable_to_non_nullable
as List<RecordingFound>,
  ));
}

}


/// Adds pattern-matching-related methods to [RecordingState].
extension RecordingStatePatterns on RecordingState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RecordingState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RecordingState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RecordingState value)  $default,){
final _that = this;
switch (_that) {
case _RecordingState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RecordingState value)?  $default,){
final _that = this;
switch (_that) {
case _RecordingState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( RecordingPhase phase,  int seconds,  double level,  bool paused,  bool warned,  String? noteId,  RecordingEnd? ended,  RecordingSaved? saved,  RecordingProblem? problem,  List<RecordingFound> found)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RecordingState() when $default != null:
return $default(_that.phase,_that.seconds,_that.level,_that.paused,_that.warned,_that.noteId,_that.ended,_that.saved,_that.problem,_that.found);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( RecordingPhase phase,  int seconds,  double level,  bool paused,  bool warned,  String? noteId,  RecordingEnd? ended,  RecordingSaved? saved,  RecordingProblem? problem,  List<RecordingFound> found)  $default,) {final _that = this;
switch (_that) {
case _RecordingState():
return $default(_that.phase,_that.seconds,_that.level,_that.paused,_that.warned,_that.noteId,_that.ended,_that.saved,_that.problem,_that.found);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( RecordingPhase phase,  int seconds,  double level,  bool paused,  bool warned,  String? noteId,  RecordingEnd? ended,  RecordingSaved? saved,  RecordingProblem? problem,  List<RecordingFound> found)?  $default,) {final _that = this;
switch (_that) {
case _RecordingState() when $default != null:
return $default(_that.phase,_that.seconds,_that.level,_that.paused,_that.warned,_that.noteId,_that.ended,_that.saved,_that.problem,_that.found);case _:
  return null;

}
}

}

/// @nodoc


class _RecordingState extends RecordingState {
  const _RecordingState({this.phase = RecordingPhase.idle, this.seconds = 0, this.level = 0.0, this.paused = false, this.warned = false, this.noteId, this.ended, this.saved, this.problem, final  List<RecordingFound> found = const <RecordingFound>[]}): _found = found,super._();
  

@override@JsonKey() final  RecordingPhase phase;
/// Audio time recorded, pause excluded.
@override@JsonKey() final  int seconds;
/// The level meter, 0 to 1.
@override@JsonKey() final  double level;
@override@JsonKey() final  bool paused;
/// The 5-hour warning has been raised.
@override@JsonKey() final  bool warned;
/// The note being recorded into; null for a new note.
@override final  String? noteId;
/// How the last recording ended by itself (cap, microphone, disk).
@override final  RecordingEnd? ended;
/// The last recording's note, until the screen has shown it.
@override final  RecordingSaved? saved;
@override final  RecordingProblem? problem;
/// Recordings a crash left.
 final  List<RecordingFound> _found;
/// Recordings a crash left.
@override@JsonKey() List<RecordingFound> get found {
  if (_found is EqualUnmodifiableListView) return _found;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_found);
}


/// Create a copy of RecordingState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RecordingStateCopyWith<_RecordingState> get copyWith => __$RecordingStateCopyWithImpl<_RecordingState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RecordingState&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.seconds, seconds) || other.seconds == seconds)&&(identical(other.level, level) || other.level == level)&&(identical(other.paused, paused) || other.paused == paused)&&(identical(other.warned, warned) || other.warned == warned)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.ended, ended) || other.ended == ended)&&(identical(other.saved, saved) || other.saved == saved)&&(identical(other.problem, problem) || other.problem == problem)&&const DeepCollectionEquality().equals(other._found, _found));
}


@override
int get hashCode => Object.hash(runtimeType,phase,seconds,level,paused,warned,noteId,ended,saved,problem,const DeepCollectionEquality().hash(_found));

@override
String toString() {
  return 'RecordingState(phase: $phase, seconds: $seconds, level: $level, paused: $paused, warned: $warned, noteId: $noteId, ended: $ended, saved: $saved, problem: $problem, found: $found)';
}


}

/// @nodoc
abstract mixin class _$RecordingStateCopyWith<$Res> implements $RecordingStateCopyWith<$Res> {
  factory _$RecordingStateCopyWith(_RecordingState value, $Res Function(_RecordingState) _then) = __$RecordingStateCopyWithImpl;
@override @useResult
$Res call({
 RecordingPhase phase, int seconds, double level, bool paused, bool warned, String? noteId, RecordingEnd? ended, RecordingSaved? saved, RecordingProblem? problem, List<RecordingFound> found
});




}
/// @nodoc
class __$RecordingStateCopyWithImpl<$Res>
    implements _$RecordingStateCopyWith<$Res> {
  __$RecordingStateCopyWithImpl(this._self, this._then);

  final _RecordingState _self;
  final $Res Function(_RecordingState) _then;

/// Create a copy of RecordingState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? phase = null,Object? seconds = null,Object? level = null,Object? paused = null,Object? warned = null,Object? noteId = freezed,Object? ended = freezed,Object? saved = freezed,Object? problem = freezed,Object? found = null,}) {
  return _then(_RecordingState(
phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as RecordingPhase,seconds: null == seconds ? _self.seconds : seconds // ignore: cast_nullable_to_non_nullable
as int,level: null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as double,paused: null == paused ? _self.paused : paused // ignore: cast_nullable_to_non_nullable
as bool,warned: null == warned ? _self.warned : warned // ignore: cast_nullable_to_non_nullable
as bool,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,ended: freezed == ended ? _self.ended : ended // ignore: cast_nullable_to_non_nullable
as RecordingEnd?,saved: freezed == saved ? _self.saved : saved // ignore: cast_nullable_to_non_nullable
as RecordingSaved?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as RecordingProblem?,found: null == found ? _self._found : found // ignore: cast_nullable_to_non_nullable
as List<RecordingFound>,
  ));
}


}

// dart format on
