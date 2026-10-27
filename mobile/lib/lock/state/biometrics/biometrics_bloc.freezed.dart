// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'biometrics_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$BiometricsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BiometricsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BiometricsEvent()';
}


}

/// @nodoc
class $BiometricsEventCopyWith<$Res>  {
$BiometricsEventCopyWith(BiometricsEvent _, $Res Function(BiometricsEvent) __);
}


/// Adds pattern-matching-related methods to [BiometricsEvent].
extension BiometricsEventPatterns on BiometricsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BiometricsTurnedOnEvent value)?  turnedOn,TResult Function( BiometricsTurnedOffEvent value)?  turnedOff,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BiometricsTurnedOnEvent() when turnedOn != null:
return turnedOn(_that);case BiometricsTurnedOffEvent() when turnedOff != null:
return turnedOff(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BiometricsTurnedOnEvent value)  turnedOn,required TResult Function( BiometricsTurnedOffEvent value)  turnedOff,}){
final _that = this;
switch (_that) {
case BiometricsTurnedOnEvent():
return turnedOn(_that);case BiometricsTurnedOffEvent():
return turnedOff(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BiometricsTurnedOnEvent value)?  turnedOn,TResult? Function( BiometricsTurnedOffEvent value)?  turnedOff,}){
final _that = this;
switch (_that) {
case BiometricsTurnedOnEvent() when turnedOn != null:
return turnedOn(_that);case BiometricsTurnedOffEvent() when turnedOff != null:
return turnedOff(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String reason)?  turnedOn,TResult Function()?  turnedOff,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BiometricsTurnedOnEvent() when turnedOn != null:
return turnedOn(_that.reason);case BiometricsTurnedOffEvent() when turnedOff != null:
return turnedOff();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String reason)  turnedOn,required TResult Function()  turnedOff,}) {final _that = this;
switch (_that) {
case BiometricsTurnedOnEvent():
return turnedOn(_that.reason);case BiometricsTurnedOffEvent():
return turnedOff();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String reason)?  turnedOn,TResult? Function()?  turnedOff,}) {final _that = this;
switch (_that) {
case BiometricsTurnedOnEvent() when turnedOn != null:
return turnedOn(_that.reason);case BiometricsTurnedOffEvent() when turnedOff != null:
return turnedOff();case _:
  return null;

}
}

}

/// @nodoc


class BiometricsTurnedOnEvent implements BiometricsEvent {
  const BiometricsTurnedOnEvent(this.reason);
  

 final  String reason;

/// Create a copy of BiometricsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BiometricsTurnedOnEventCopyWith<BiometricsTurnedOnEvent> get copyWith => _$BiometricsTurnedOnEventCopyWithImpl<BiometricsTurnedOnEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BiometricsTurnedOnEvent&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'BiometricsEvent.turnedOn(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $BiometricsTurnedOnEventCopyWith<$Res> implements $BiometricsEventCopyWith<$Res> {
  factory $BiometricsTurnedOnEventCopyWith(BiometricsTurnedOnEvent value, $Res Function(BiometricsTurnedOnEvent) _then) = _$BiometricsTurnedOnEventCopyWithImpl;
@useResult
$Res call({
 String reason
});




}
/// @nodoc
class _$BiometricsTurnedOnEventCopyWithImpl<$Res>
    implements $BiometricsTurnedOnEventCopyWith<$Res> {
  _$BiometricsTurnedOnEventCopyWithImpl(this._self, this._then);

  final BiometricsTurnedOnEvent _self;
  final $Res Function(BiometricsTurnedOnEvent) _then;

/// Create a copy of BiometricsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(BiometricsTurnedOnEvent(
null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class BiometricsTurnedOffEvent implements BiometricsEvent {
  const BiometricsTurnedOffEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BiometricsTurnedOffEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BiometricsEvent.turnedOff()';
}


}




/// @nodoc
mixin _$BiometricsState {

 BiometricsStatus get status; ApplicationError? get error;
/// Create a copy of BiometricsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BiometricsStateCopyWith<BiometricsState> get copyWith => _$BiometricsStateCopyWithImpl<BiometricsState>(this as BiometricsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BiometricsState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,error);

@override
String toString() {
  return 'BiometricsState(status: $status, error: $error)';
}


}

/// @nodoc
abstract mixin class $BiometricsStateCopyWith<$Res>  {
  factory $BiometricsStateCopyWith(BiometricsState value, $Res Function(BiometricsState) _then) = _$BiometricsStateCopyWithImpl;
@useResult
$Res call({
 BiometricsStatus status, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$BiometricsStateCopyWithImpl<$Res>
    implements $BiometricsStateCopyWith<$Res> {
  _$BiometricsStateCopyWithImpl(this._self, this._then);

  final BiometricsState _self;
  final $Res Function(BiometricsState) _then;

/// Create a copy of BiometricsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as BiometricsStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of BiometricsState
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


/// Adds pattern-matching-related methods to [BiometricsState].
extension BiometricsStatePatterns on BiometricsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _BiometricsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _BiometricsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _BiometricsState value)  $default,){
final _that = this;
switch (_that) {
case _BiometricsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _BiometricsState value)?  $default,){
final _that = this;
switch (_that) {
case _BiometricsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( BiometricsStatus status,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _BiometricsState() when $default != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( BiometricsStatus status,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _BiometricsState():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( BiometricsStatus status,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _BiometricsState() when $default != null:
return $default(_that.status,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _BiometricsState implements BiometricsState {
  const _BiometricsState({this.status = BiometricsStatus.idle, this.error});
  

@override@JsonKey() final  BiometricsStatus status;
@override final  ApplicationError? error;

/// Create a copy of BiometricsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$BiometricsStateCopyWith<_BiometricsState> get copyWith => __$BiometricsStateCopyWithImpl<_BiometricsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _BiometricsState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,error);

@override
String toString() {
  return 'BiometricsState(status: $status, error: $error)';
}


}

/// @nodoc
abstract mixin class _$BiometricsStateCopyWith<$Res> implements $BiometricsStateCopyWith<$Res> {
  factory _$BiometricsStateCopyWith(_BiometricsState value, $Res Function(_BiometricsState) _then) = __$BiometricsStateCopyWithImpl;
@override @useResult
$Res call({
 BiometricsStatus status, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$BiometricsStateCopyWithImpl<$Res>
    implements _$BiometricsStateCopyWith<$Res> {
  __$BiometricsStateCopyWithImpl(this._self, this._then);

  final _BiometricsState _self;
  final $Res Function(_BiometricsState) _then;

/// Create a copy of BiometricsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? error = freezed,}) {
  return _then(_BiometricsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as BiometricsStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of BiometricsState
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
