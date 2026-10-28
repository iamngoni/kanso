// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'daily_note_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DailyNoteEvent {

 DateTime get now;
/// Create a copy of DailyNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DailyNoteEventCopyWith<DailyNoteEvent> get copyWith => _$DailyNoteEventCopyWithImpl<DailyNoteEvent>(this as DailyNoteEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyNoteEvent&&(identical(other.now, now) || other.now == now));
}


@override
int get hashCode => Object.hash(runtimeType,now);

@override
String toString() {
  return 'DailyNoteEvent(now: $now)';
}


}

/// @nodoc
abstract mixin class $DailyNoteEventCopyWith<$Res>  {
  factory $DailyNoteEventCopyWith(DailyNoteEvent value, $Res Function(DailyNoteEvent) _then) = _$DailyNoteEventCopyWithImpl;
@useResult
$Res call({
 DateTime now
});




}
/// @nodoc
class _$DailyNoteEventCopyWithImpl<$Res>
    implements $DailyNoteEventCopyWith<$Res> {
  _$DailyNoteEventCopyWithImpl(this._self, this._then);

  final DailyNoteEvent _self;
  final $Res Function(DailyNoteEvent) _then;

/// Create a copy of DailyNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? now = null,}) {
  return _then(_self.copyWith(
now: null == now ? _self.now : now // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}

}


/// Adds pattern-matching-related methods to [DailyNoteEvent].
extension DailyNoteEventPatterns on DailyNoteEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DailyNoteRequestedEvent value)?  requested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DailyNoteRequestedEvent() when requested != null:
return requested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DailyNoteRequestedEvent value)  requested,}){
final _that = this;
switch (_that) {
case DailyNoteRequestedEvent():
return requested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DailyNoteRequestedEvent value)?  requested,}){
final _that = this;
switch (_that) {
case DailyNoteRequestedEvent() when requested != null:
return requested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( DateTime now)?  requested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DailyNoteRequestedEvent() when requested != null:
return requested(_that.now);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( DateTime now)  requested,}) {final _that = this;
switch (_that) {
case DailyNoteRequestedEvent():
return requested(_that.now);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( DateTime now)?  requested,}) {final _that = this;
switch (_that) {
case DailyNoteRequestedEvent() when requested != null:
return requested(_that.now);case _:
  return null;

}
}

}

/// @nodoc


class DailyNoteRequestedEvent implements DailyNoteEvent {
  const DailyNoteRequestedEvent(this.now);
  

@override final  DateTime now;

/// Create a copy of DailyNoteEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DailyNoteRequestedEventCopyWith<DailyNoteRequestedEvent> get copyWith => _$DailyNoteRequestedEventCopyWithImpl<DailyNoteRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyNoteRequestedEvent&&(identical(other.now, now) || other.now == now));
}


@override
int get hashCode => Object.hash(runtimeType,now);

@override
String toString() {
  return 'DailyNoteEvent.requested(now: $now)';
}


}

/// @nodoc
abstract mixin class $DailyNoteRequestedEventCopyWith<$Res> implements $DailyNoteEventCopyWith<$Res> {
  factory $DailyNoteRequestedEventCopyWith(DailyNoteRequestedEvent value, $Res Function(DailyNoteRequestedEvent) _then) = _$DailyNoteRequestedEventCopyWithImpl;
@override @useResult
$Res call({
 DateTime now
});




}
/// @nodoc
class _$DailyNoteRequestedEventCopyWithImpl<$Res>
    implements $DailyNoteRequestedEventCopyWith<$Res> {
  _$DailyNoteRequestedEventCopyWithImpl(this._self, this._then);

  final DailyNoteRequestedEvent _self;
  final $Res Function(DailyNoteRequestedEvent) _then;

/// Create a copy of DailyNoteEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? now = null,}) {
  return _then(DailyNoteRequestedEvent(
null == now ? _self.now : now // ignore: cast_nullable_to_non_nullable
as DateTime,
  ));
}


}

/// @nodoc
mixin _$DailyNoteState {

 LoadStatus get status;/// Today's note, once [status] is ready.
 String? get noteId;/// Why it could not be opened, once [status] is failed.
 ApplicationError? get error;
/// Create a copy of DailyNoteState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DailyNoteStateCopyWith<DailyNoteState> get copyWith => _$DailyNoteStateCopyWithImpl<DailyNoteState>(this as DailyNoteState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DailyNoteState&&(identical(other.status, status) || other.status == status)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,noteId,error);

@override
String toString() {
  return 'DailyNoteState(status: $status, noteId: $noteId, error: $error)';
}


}

/// @nodoc
abstract mixin class $DailyNoteStateCopyWith<$Res>  {
  factory $DailyNoteStateCopyWith(DailyNoteState value, $Res Function(DailyNoteState) _then) = _$DailyNoteStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, String? noteId, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$DailyNoteStateCopyWithImpl<$Res>
    implements $DailyNoteStateCopyWith<$Res> {
  _$DailyNoteStateCopyWithImpl(this._self, this._then);

  final DailyNoteState _self;
  final $Res Function(DailyNoteState) _then;

/// Create a copy of DailyNoteState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? noteId = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of DailyNoteState
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


/// Adds pattern-matching-related methods to [DailyNoteState].
extension DailyNoteStatePatterns on DailyNoteState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DailyNoteState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DailyNoteState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DailyNoteState value)  $default,){
final _that = this;
switch (_that) {
case _DailyNoteState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DailyNoteState value)?  $default,){
final _that = this;
switch (_that) {
case _DailyNoteState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  String? noteId,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DailyNoteState() when $default != null:
return $default(_that.status,_that.noteId,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  String? noteId,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _DailyNoteState():
return $default(_that.status,_that.noteId,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  String? noteId,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _DailyNoteState() when $default != null:
return $default(_that.status,_that.noteId,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _DailyNoteState implements DailyNoteState {
  const _DailyNoteState({this.status = LoadStatus.loading, this.noteId, this.error});
  

@override@JsonKey() final  LoadStatus status;
/// Today's note, once [status] is ready.
@override final  String? noteId;
/// Why it could not be opened, once [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of DailyNoteState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DailyNoteStateCopyWith<_DailyNoteState> get copyWith => __$DailyNoteStateCopyWithImpl<_DailyNoteState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DailyNoteState&&(identical(other.status, status) || other.status == status)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,noteId,error);

@override
String toString() {
  return 'DailyNoteState(status: $status, noteId: $noteId, error: $error)';
}


}

/// @nodoc
abstract mixin class _$DailyNoteStateCopyWith<$Res> implements $DailyNoteStateCopyWith<$Res> {
  factory _$DailyNoteStateCopyWith(_DailyNoteState value, $Res Function(_DailyNoteState) _then) = __$DailyNoteStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, String? noteId, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$DailyNoteStateCopyWithImpl<$Res>
    implements _$DailyNoteStateCopyWith<$Res> {
  __$DailyNoteStateCopyWithImpl(this._self, this._then);

  final _DailyNoteState _self;
  final $Res Function(_DailyNoteState) _then;

/// Create a copy of DailyNoteState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? noteId = freezed,Object? error = freezed,}) {
  return _then(_DailyNoteState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,noteId: freezed == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of DailyNoteState
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
