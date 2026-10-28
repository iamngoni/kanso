// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'shared_note_status_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SharedNoteStatusEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteStatusEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNoteStatusEvent()';
}


}

/// @nodoc
class $SharedNoteStatusEventCopyWith<$Res>  {
$SharedNoteStatusEventCopyWith(SharedNoteStatusEvent _, $Res Function(SharedNoteStatusEvent) __);
}


/// Adds pattern-matching-related methods to [SharedNoteStatusEvent].
extension SharedNoteStatusEventPatterns on SharedNoteStatusEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SharedNoteStatusStartedEvent value)?  started,TResult Function( SharedNoteStatusRefreshedEvent value)?  refreshed,TResult Function( SharedNoteStatusProblemsDismissedEvent value)?  problemsDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SharedNoteStatusStartedEvent() when started != null:
return started(_that);case SharedNoteStatusRefreshedEvent() when refreshed != null:
return refreshed(_that);case SharedNoteStatusProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SharedNoteStatusStartedEvent value)  started,required TResult Function( SharedNoteStatusRefreshedEvent value)  refreshed,required TResult Function( SharedNoteStatusProblemsDismissedEvent value)  problemsDismissed,}){
final _that = this;
switch (_that) {
case SharedNoteStatusStartedEvent():
return started(_that);case SharedNoteStatusRefreshedEvent():
return refreshed(_that);case SharedNoteStatusProblemsDismissedEvent():
return problemsDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SharedNoteStatusStartedEvent value)?  started,TResult? Function( SharedNoteStatusRefreshedEvent value)?  refreshed,TResult? Function( SharedNoteStatusProblemsDismissedEvent value)?  problemsDismissed,}){
final _that = this;
switch (_that) {
case SharedNoteStatusStartedEvent() when started != null:
return started(_that);case SharedNoteStatusRefreshedEvent() when refreshed != null:
return refreshed(_that);case SharedNoteStatusProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function()?  refreshed,TResult Function()?  problemsDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SharedNoteStatusStartedEvent() when started != null:
return started(_that.noteId);case SharedNoteStatusRefreshedEvent() when refreshed != null:
return refreshed();case SharedNoteStatusProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function()  refreshed,required TResult Function()  problemsDismissed,}) {final _that = this;
switch (_that) {
case SharedNoteStatusStartedEvent():
return started(_that.noteId);case SharedNoteStatusRefreshedEvent():
return refreshed();case SharedNoteStatusProblemsDismissedEvent():
return problemsDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function()?  refreshed,TResult? Function()?  problemsDismissed,}) {final _that = this;
switch (_that) {
case SharedNoteStatusStartedEvent() when started != null:
return started(_that.noteId);case SharedNoteStatusRefreshedEvent() when refreshed != null:
return refreshed();case SharedNoteStatusProblemsDismissedEvent() when problemsDismissed != null:
return problemsDismissed();case _:
  return null;

}
}

}

/// @nodoc


class SharedNoteStatusStartedEvent implements SharedNoteStatusEvent {
  const SharedNoteStatusStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of SharedNoteStatusEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNoteStatusStartedEventCopyWith<SharedNoteStatusStartedEvent> get copyWith => _$SharedNoteStatusStartedEventCopyWithImpl<SharedNoteStatusStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteStatusStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'SharedNoteStatusEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $SharedNoteStatusStartedEventCopyWith<$Res> implements $SharedNoteStatusEventCopyWith<$Res> {
  factory $SharedNoteStatusStartedEventCopyWith(SharedNoteStatusStartedEvent value, $Res Function(SharedNoteStatusStartedEvent) _then) = _$SharedNoteStatusStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$SharedNoteStatusStartedEventCopyWithImpl<$Res>
    implements $SharedNoteStatusStartedEventCopyWith<$Res> {
  _$SharedNoteStatusStartedEventCopyWithImpl(this._self, this._then);

  final SharedNoteStatusStartedEvent _self;
  final $Res Function(SharedNoteStatusStartedEvent) _then;

/// Create a copy of SharedNoteStatusEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(SharedNoteStatusStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SharedNoteStatusRefreshedEvent implements SharedNoteStatusEvent {
  const SharedNoteStatusRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteStatusRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNoteStatusEvent.refreshed()';
}


}




/// @nodoc


class SharedNoteStatusProblemsDismissedEvent implements SharedNoteStatusEvent {
  const SharedNoteStatusProblemsDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteStatusProblemsDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNoteStatusEvent.problemsDismissed()';
}


}




/// @nodoc
mixin _$SharedNoteStatusState {

/// The note this is the status of.
 String get noteId;/// Its status; null while it is read, and for a note that isn't
/// shared.
 LiveStatus? get status;/// Why the status could not be read, or the problems not dismissed.
 ApplicationError? get error;/// Bumped each time the status is read again: someone's cursor may have
/// moved, and the editor asks where the cursors are now.
 int get beat;
/// Create a copy of SharedNoteStatusState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNoteStatusStateCopyWith<SharedNoteStatusState> get copyWith => _$SharedNoteStatusStateCopyWithImpl<SharedNoteStatusState>(this as SharedNoteStatusState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNoteStatusState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error)&&(identical(other.beat, beat) || other.beat == beat));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,error,beat);

@override
String toString() {
  return 'SharedNoteStatusState(noteId: $noteId, status: $status, error: $error, beat: $beat)';
}


}

/// @nodoc
abstract mixin class $SharedNoteStatusStateCopyWith<$Res>  {
  factory $SharedNoteStatusStateCopyWith(SharedNoteStatusState value, $Res Function(SharedNoteStatusState) _then) = _$SharedNoteStatusStateCopyWithImpl;
@useResult
$Res call({
 String noteId, LiveStatus? status, ApplicationError? error, int beat
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$SharedNoteStatusStateCopyWithImpl<$Res>
    implements $SharedNoteStatusStateCopyWith<$Res> {
  _$SharedNoteStatusStateCopyWithImpl(this._self, this._then);

  final SharedNoteStatusState _self;
  final $Res Function(SharedNoteStatusState) _then;

/// Create a copy of SharedNoteStatusState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? status = freezed,Object? error = freezed,Object? beat = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: freezed == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LiveStatus?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,beat: null == beat ? _self.beat : beat // ignore: cast_nullable_to_non_nullable
as int,
  ));
}
/// Create a copy of SharedNoteStatusState
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
}
}


/// Adds pattern-matching-related methods to [SharedNoteStatusState].
extension SharedNoteStatusStatePatterns on SharedNoteStatusState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SharedNoteStatusState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SharedNoteStatusState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SharedNoteStatusState value)  $default,){
final _that = this;
switch (_that) {
case _SharedNoteStatusState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SharedNoteStatusState value)?  $default,){
final _that = this;
switch (_that) {
case _SharedNoteStatusState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  LiveStatus? status,  ApplicationError? error,  int beat)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SharedNoteStatusState() when $default != null:
return $default(_that.noteId,_that.status,_that.error,_that.beat);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  LiveStatus? status,  ApplicationError? error,  int beat)  $default,) {final _that = this;
switch (_that) {
case _SharedNoteStatusState():
return $default(_that.noteId,_that.status,_that.error,_that.beat);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  LiveStatus? status,  ApplicationError? error,  int beat)?  $default,) {final _that = this;
switch (_that) {
case _SharedNoteStatusState() when $default != null:
return $default(_that.noteId,_that.status,_that.error,_that.beat);case _:
  return null;

}
}

}

/// @nodoc


class _SharedNoteStatusState extends SharedNoteStatusState {
  const _SharedNoteStatusState({this.noteId = '', this.status, this.error, this.beat = 0}): super._();
  

/// The note this is the status of.
@override@JsonKey() final  String noteId;
/// Its status; null while it is read, and for a note that isn't
/// shared.
@override final  LiveStatus? status;
/// Why the status could not be read, or the problems not dismissed.
@override final  ApplicationError? error;
/// Bumped each time the status is read again: someone's cursor may have
/// moved, and the editor asks where the cursors are now.
@override@JsonKey() final  int beat;

/// Create a copy of SharedNoteStatusState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SharedNoteStatusStateCopyWith<_SharedNoteStatusState> get copyWith => __$SharedNoteStatusStateCopyWithImpl<_SharedNoteStatusState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SharedNoteStatusState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error)&&(identical(other.beat, beat) || other.beat == beat));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,error,beat);

@override
String toString() {
  return 'SharedNoteStatusState(noteId: $noteId, status: $status, error: $error, beat: $beat)';
}


}

/// @nodoc
abstract mixin class _$SharedNoteStatusStateCopyWith<$Res> implements $SharedNoteStatusStateCopyWith<$Res> {
  factory _$SharedNoteStatusStateCopyWith(_SharedNoteStatusState value, $Res Function(_SharedNoteStatusState) _then) = __$SharedNoteStatusStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, LiveStatus? status, ApplicationError? error, int beat
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$SharedNoteStatusStateCopyWithImpl<$Res>
    implements _$SharedNoteStatusStateCopyWith<$Res> {
  __$SharedNoteStatusStateCopyWithImpl(this._self, this._then);

  final _SharedNoteStatusState _self;
  final $Res Function(_SharedNoteStatusState) _then;

/// Create a copy of SharedNoteStatusState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? status = freezed,Object? error = freezed,Object? beat = null,}) {
  return _then(_SharedNoteStatusState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: freezed == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LiveStatus?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,beat: null == beat ? _self.beat : beat // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

/// Create a copy of SharedNoteStatusState
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
}
}

// dart format on
