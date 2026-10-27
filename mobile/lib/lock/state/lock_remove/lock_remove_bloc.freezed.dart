// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lock_remove_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LockRemoveEvent {

 LockTarget get target;
/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockRemoveEventCopyWith<LockRemoveEvent> get copyWith => _$LockRemoveEventCopyWithImpl<LockRemoveEvent>(this as LockRemoveEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveEvent&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,target);

@override
String toString() {
  return 'LockRemoveEvent(target: $target)';
}


}

/// @nodoc
abstract mixin class $LockRemoveEventCopyWith<$Res>  {
  factory $LockRemoveEventCopyWith(LockRemoveEvent value, $Res Function(LockRemoveEvent) _then) = _$LockRemoveEventCopyWithImpl;
@useResult
$Res call({
 LockTarget target
});


$LockTargetCopyWith<$Res> get target;

}
/// @nodoc
class _$LockRemoveEventCopyWithImpl<$Res>
    implements $LockRemoveEventCopyWith<$Res> {
  _$LockRemoveEventCopyWithImpl(this._self, this._then);

  final LockRemoveEvent _self;
  final $Res Function(LockRemoveEvent) _then;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? target = null,}) {
  return _then(_self.copyWith(
target: null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget,
  ));
}
/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$LockTargetCopyWith<$Res> get target {
  
  return $LockTargetCopyWith<$Res>(_self.target, (value) {
    return _then(_self.copyWith(target: value));
  });
}
}


/// Adds pattern-matching-related methods to [LockRemoveEvent].
extension LockRemoveEventPatterns on LockRemoveEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockRemoveConfirmedEvent value)?  confirmed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockRemoveConfirmedEvent value)  confirmed,}){
final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent():
return confirmed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockRemoveConfirmedEvent value)?  confirmed,}){
final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( LockTarget target)?  confirmed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that.target);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( LockTarget target)  confirmed,}) {final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent():
return confirmed(_that.target);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( LockTarget target)?  confirmed,}) {final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that.target);case _:
  return null;

}
}

}

/// @nodoc


class LockRemoveConfirmedEvent implements LockRemoveEvent {
  const LockRemoveConfirmedEvent(this.target);
  

@override final  LockTarget target;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockRemoveConfirmedEventCopyWith<LockRemoveConfirmedEvent> get copyWith => _$LockRemoveConfirmedEventCopyWithImpl<LockRemoveConfirmedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveConfirmedEvent&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,target);

@override
String toString() {
  return 'LockRemoveEvent.confirmed(target: $target)';
}


}

/// @nodoc
abstract mixin class $LockRemoveConfirmedEventCopyWith<$Res> implements $LockRemoveEventCopyWith<$Res> {
  factory $LockRemoveConfirmedEventCopyWith(LockRemoveConfirmedEvent value, $Res Function(LockRemoveConfirmedEvent) _then) = _$LockRemoveConfirmedEventCopyWithImpl;
@override @useResult
$Res call({
 LockTarget target
});


@override $LockTargetCopyWith<$Res> get target;

}
/// @nodoc
class _$LockRemoveConfirmedEventCopyWithImpl<$Res>
    implements $LockRemoveConfirmedEventCopyWith<$Res> {
  _$LockRemoveConfirmedEventCopyWithImpl(this._self, this._then);

  final LockRemoveConfirmedEvent _self;
  final $Res Function(LockRemoveConfirmedEvent) _then;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? target = null,}) {
  return _then(LockRemoveConfirmedEvent(
null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget,
  ));
}

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$LockTargetCopyWith<$Res> get target {
  
  return $LockTargetCopyWith<$Res>(_self.target, (value) {
    return _then(_self.copyWith(target: value));
  });
}
}

/// @nodoc
mixin _$LockRemoveState {

 RemoveStatus get status;/// Why it failed, with [RemoveStatus.failed]. An invalid request carries
/// the library's own sentence, which the sheet shows as it is.
 ApplicationError? get error;
/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockRemoveStateCopyWith<LockRemoveState> get copyWith => _$LockRemoveStateCopyWithImpl<LockRemoveState>(this as LockRemoveState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,error);

@override
String toString() {
  return 'LockRemoveState(status: $status, error: $error)';
}


}

/// @nodoc
abstract mixin class $LockRemoveStateCopyWith<$Res>  {
  factory $LockRemoveStateCopyWith(LockRemoveState value, $Res Function(LockRemoveState) _then) = _$LockRemoveStateCopyWithImpl;
@useResult
$Res call({
 RemoveStatus status, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$LockRemoveStateCopyWithImpl<$Res>
    implements $LockRemoveStateCopyWith<$Res> {
  _$LockRemoveStateCopyWithImpl(this._self, this._then);

  final LockRemoveState _self;
  final $Res Function(LockRemoveState) _then;

/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as RemoveStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of LockRemoveState
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


/// Adds pattern-matching-related methods to [LockRemoveState].
extension LockRemoveStatePatterns on LockRemoveState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LockRemoveState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LockRemoveState value)  $default,){
final _that = this;
switch (_that) {
case _LockRemoveState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LockRemoveState value)?  $default,){
final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( RemoveStatus status,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
return $default(_that.status,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( RemoveStatus status,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _LockRemoveState():
return $default(_that.status,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( RemoveStatus status,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
return $default(_that.status,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _LockRemoveState implements LockRemoveState {
  const _LockRemoveState({this.status = RemoveStatus.idle, this.error});
  

@override@JsonKey() final  RemoveStatus status;
/// Why it failed, with [RemoveStatus.failed]. An invalid request carries
/// the library's own sentence, which the sheet shows as it is.
@override final  ApplicationError? error;

/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LockRemoveStateCopyWith<_LockRemoveState> get copyWith => __$LockRemoveStateCopyWithImpl<_LockRemoveState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LockRemoveState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,error);

@override
String toString() {
  return 'LockRemoveState(status: $status, error: $error)';
}


}

/// @nodoc
abstract mixin class _$LockRemoveStateCopyWith<$Res> implements $LockRemoveStateCopyWith<$Res> {
  factory _$LockRemoveStateCopyWith(_LockRemoveState value, $Res Function(_LockRemoveState) _then) = __$LockRemoveStateCopyWithImpl;
@override @useResult
$Res call({
 RemoveStatus status, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$LockRemoveStateCopyWithImpl<$Res>
    implements _$LockRemoveStateCopyWith<$Res> {
  __$LockRemoveStateCopyWithImpl(this._self, this._then);

  final _LockRemoveState _self;
  final $Res Function(_LockRemoveState) _then;

/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? error = freezed,}) {
  return _then(_LockRemoveState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as RemoveStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of LockRemoveState
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
