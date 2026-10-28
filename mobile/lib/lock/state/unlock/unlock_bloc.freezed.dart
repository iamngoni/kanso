// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'unlock_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$UnlockEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'UnlockEvent()';
}


}

/// @nodoc
class $UnlockEventCopyWith<$Res>  {
$UnlockEventCopyWith(UnlockEvent _, $Res Function(UnlockEvent) __);
}


/// Adds pattern-matching-related methods to [UnlockEvent].
extension UnlockEventPatterns on UnlockEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( UnlockGateOpenedEvent value)?  gateOpened,TResult Function( UnlockBiometricRequestedEvent value)?  biometricRequested,TResult Function( UnlockPasswordSubmittedEvent value)?  passwordSubmitted,required TResult orElse(),}){
final _that = this;
switch (_that) {
case UnlockGateOpenedEvent() when gateOpened != null:
return gateOpened(_that);case UnlockBiometricRequestedEvent() when biometricRequested != null:
return biometricRequested(_that);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( UnlockGateOpenedEvent value)  gateOpened,required TResult Function( UnlockBiometricRequestedEvent value)  biometricRequested,required TResult Function( UnlockPasswordSubmittedEvent value)  passwordSubmitted,}){
final _that = this;
switch (_that) {
case UnlockGateOpenedEvent():
return gateOpened(_that);case UnlockBiometricRequestedEvent():
return biometricRequested(_that);case UnlockPasswordSubmittedEvent():
return passwordSubmitted(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( UnlockGateOpenedEvent value)?  gateOpened,TResult? Function( UnlockBiometricRequestedEvent value)?  biometricRequested,TResult? Function( UnlockPasswordSubmittedEvent value)?  passwordSubmitted,}){
final _that = this;
switch (_that) {
case UnlockGateOpenedEvent() when gateOpened != null:
return gateOpened(_that);case UnlockBiometricRequestedEvent() when biometricRequested != null:
return biometricRequested(_that);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( LockTarget target,  String reason)?  gateOpened,TResult Function( String reason)?  biometricRequested,TResult Function( LockTarget target,  String password)?  passwordSubmitted,required TResult orElse(),}) {final _that = this;
switch (_that) {
case UnlockGateOpenedEvent() when gateOpened != null:
return gateOpened(_that.target,_that.reason);case UnlockBiometricRequestedEvent() when biometricRequested != null:
return biometricRequested(_that.reason);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.target,_that.password);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( LockTarget target,  String reason)  gateOpened,required TResult Function( String reason)  biometricRequested,required TResult Function( LockTarget target,  String password)  passwordSubmitted,}) {final _that = this;
switch (_that) {
case UnlockGateOpenedEvent():
return gateOpened(_that.target,_that.reason);case UnlockBiometricRequestedEvent():
return biometricRequested(_that.reason);case UnlockPasswordSubmittedEvent():
return passwordSubmitted(_that.target,_that.password);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( LockTarget target,  String reason)?  gateOpened,TResult? Function( String reason)?  biometricRequested,TResult? Function( LockTarget target,  String password)?  passwordSubmitted,}) {final _that = this;
switch (_that) {
case UnlockGateOpenedEvent() when gateOpened != null:
return gateOpened(_that.target,_that.reason);case UnlockBiometricRequestedEvent() when biometricRequested != null:
return biometricRequested(_that.reason);case UnlockPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.target,_that.password);case _:
  return null;

}
}

}

/// @nodoc


class UnlockGateOpenedEvent implements UnlockEvent {
  const UnlockGateOpenedEvent(this.target, this.reason);
  

 final  LockTarget target;
 final  String reason;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockGateOpenedEventCopyWith<UnlockGateOpenedEvent> get copyWith => _$UnlockGateOpenedEventCopyWithImpl<UnlockGateOpenedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockGateOpenedEvent&&(identical(other.target, target) || other.target == target)&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,target,reason);

@override
String toString() {
  return 'UnlockEvent.gateOpened(target: $target, reason: $reason)';
}


}

/// @nodoc
abstract mixin class $UnlockGateOpenedEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockGateOpenedEventCopyWith(UnlockGateOpenedEvent value, $Res Function(UnlockGateOpenedEvent) _then) = _$UnlockGateOpenedEventCopyWithImpl;
@useResult
$Res call({
 LockTarget target, String reason
});


$LockTargetCopyWith<$Res> get target;

}
/// @nodoc
class _$UnlockGateOpenedEventCopyWithImpl<$Res>
    implements $UnlockGateOpenedEventCopyWith<$Res> {
  _$UnlockGateOpenedEventCopyWithImpl(this._self, this._then);

  final UnlockGateOpenedEvent _self;
  final $Res Function(UnlockGateOpenedEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? target = null,Object? reason = null,}) {
  return _then(UnlockGateOpenedEvent(
null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget,null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of UnlockEvent
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


class UnlockBiometricRequestedEvent implements UnlockEvent {
  const UnlockBiometricRequestedEvent(this.reason);
  

 final  String reason;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockBiometricRequestedEventCopyWith<UnlockBiometricRequestedEvent> get copyWith => _$UnlockBiometricRequestedEventCopyWithImpl<UnlockBiometricRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockBiometricRequestedEvent&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'UnlockEvent.biometricRequested(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $UnlockBiometricRequestedEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockBiometricRequestedEventCopyWith(UnlockBiometricRequestedEvent value, $Res Function(UnlockBiometricRequestedEvent) _then) = _$UnlockBiometricRequestedEventCopyWithImpl;
@useResult
$Res call({
 String reason
});




}
/// @nodoc
class _$UnlockBiometricRequestedEventCopyWithImpl<$Res>
    implements $UnlockBiometricRequestedEventCopyWith<$Res> {
  _$UnlockBiometricRequestedEventCopyWithImpl(this._self, this._then);

  final UnlockBiometricRequestedEvent _self;
  final $Res Function(UnlockBiometricRequestedEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(UnlockBiometricRequestedEvent(
null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class UnlockPasswordSubmittedEvent implements UnlockEvent {
  const UnlockPasswordSubmittedEvent(this.target, this.password);
  

 final  LockTarget target;
 final  String password;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockPasswordSubmittedEventCopyWith<UnlockPasswordSubmittedEvent> get copyWith => _$UnlockPasswordSubmittedEventCopyWithImpl<UnlockPasswordSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockPasswordSubmittedEvent&&(identical(other.target, target) || other.target == target)&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,target,password);

@override
String toString() {
  return 'UnlockEvent.passwordSubmitted(target: $target, password: $password)';
}


}

/// @nodoc
abstract mixin class $UnlockPasswordSubmittedEventCopyWith<$Res> implements $UnlockEventCopyWith<$Res> {
  factory $UnlockPasswordSubmittedEventCopyWith(UnlockPasswordSubmittedEvent value, $Res Function(UnlockPasswordSubmittedEvent) _then) = _$UnlockPasswordSubmittedEventCopyWithImpl;
@useResult
$Res call({
 LockTarget target, String password
});


$LockTargetCopyWith<$Res> get target;

}
/// @nodoc
class _$UnlockPasswordSubmittedEventCopyWithImpl<$Res>
    implements $UnlockPasswordSubmittedEventCopyWith<$Res> {
  _$UnlockPasswordSubmittedEventCopyWithImpl(this._self, this._then);

  final UnlockPasswordSubmittedEvent _self;
  final $Res Function(UnlockPasswordSubmittedEvent) _then;

/// Create a copy of UnlockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? target = null,Object? password = null,}) {
  return _then(UnlockPasswordSubmittedEvent(
null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget,null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of UnlockEvent
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
mixin _$UnlockState {

 UnlockStatus get status;/// Whether to offer turning on a biometric now; only with
/// [UnlockStatus.opened].
 bool get offerBiometrics;/// Why it failed, with [UnlockStatus.failed].
 ApplicationError? get error;
/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnlockStateCopyWith<UnlockState> get copyWith => _$UnlockStateCopyWithImpl<UnlockState>(this as UnlockState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnlockState&&(identical(other.status, status) || other.status == status)&&(identical(other.offerBiometrics, offerBiometrics) || other.offerBiometrics == offerBiometrics)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,offerBiometrics,error);

@override
String toString() {
  return 'UnlockState(status: $status, offerBiometrics: $offerBiometrics, error: $error)';
}


}

/// @nodoc
abstract mixin class $UnlockStateCopyWith<$Res>  {
  factory $UnlockStateCopyWith(UnlockState value, $Res Function(UnlockState) _then) = _$UnlockStateCopyWithImpl;
@useResult
$Res call({
 UnlockStatus status, bool offerBiometrics, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$UnlockStateCopyWithImpl<$Res>
    implements $UnlockStateCopyWith<$Res> {
  _$UnlockStateCopyWithImpl(this._self, this._then);

  final UnlockState _self;
  final $Res Function(UnlockState) _then;

/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? offerBiometrics = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as UnlockStatus,offerBiometrics: null == offerBiometrics ? _self.offerBiometrics : offerBiometrics // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of UnlockState
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


/// Adds pattern-matching-related methods to [UnlockState].
extension UnlockStatePatterns on UnlockState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _UnlockState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _UnlockState value)  $default,){
final _that = this;
switch (_that) {
case _UnlockState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _UnlockState value)?  $default,){
final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( UnlockStatus status,  bool offerBiometrics,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
return $default(_that.status,_that.offerBiometrics,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( UnlockStatus status,  bool offerBiometrics,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _UnlockState():
return $default(_that.status,_that.offerBiometrics,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( UnlockStatus status,  bool offerBiometrics,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _UnlockState() when $default != null:
return $default(_that.status,_that.offerBiometrics,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _UnlockState implements UnlockState {
  const _UnlockState({this.status = UnlockStatus.idle, this.offerBiometrics = false, this.error});
  

@override@JsonKey() final  UnlockStatus status;
/// Whether to offer turning on a biometric now; only with
/// [UnlockStatus.opened].
@override@JsonKey() final  bool offerBiometrics;
/// Why it failed, with [UnlockStatus.failed].
@override final  ApplicationError? error;

/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$UnlockStateCopyWith<_UnlockState> get copyWith => __$UnlockStateCopyWithImpl<_UnlockState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _UnlockState&&(identical(other.status, status) || other.status == status)&&(identical(other.offerBiometrics, offerBiometrics) || other.offerBiometrics == offerBiometrics)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,offerBiometrics,error);

@override
String toString() {
  return 'UnlockState(status: $status, offerBiometrics: $offerBiometrics, error: $error)';
}


}

/// @nodoc
abstract mixin class _$UnlockStateCopyWith<$Res> implements $UnlockStateCopyWith<$Res> {
  factory _$UnlockStateCopyWith(_UnlockState value, $Res Function(_UnlockState) _then) = __$UnlockStateCopyWithImpl;
@override @useResult
$Res call({
 UnlockStatus status, bool offerBiometrics, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$UnlockStateCopyWithImpl<$Res>
    implements _$UnlockStateCopyWith<$Res> {
  __$UnlockStateCopyWithImpl(this._self, this._then);

  final _UnlockState _self;
  final $Res Function(_UnlockState) _then;

/// Create a copy of UnlockState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? offerBiometrics = null,Object? error = freezed,}) {
  return _then(_UnlockState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as UnlockStatus,offerBiometrics: null == offerBiometrics ? _self.offerBiometrics : offerBiometrics // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of UnlockState
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
