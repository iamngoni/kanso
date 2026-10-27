// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'push_registration_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PushRegistrationEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushRegistrationEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PushRegistrationEvent()';
}


}

/// @nodoc
class $PushRegistrationEventCopyWith<$Res>  {
$PushRegistrationEventCopyWith(PushRegistrationEvent _, $Res Function(PushRegistrationEvent) __);
}


/// Adds pattern-matching-related methods to [PushRegistrationEvent].
extension PushRegistrationEventPatterns on PushRegistrationEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PushRegistrationStartedEvent value)?  started,TResult Function( PushRegistrationTokenReceivedEvent value)?  tokenReceived,TResult Function( PushRegistrationResumedEvent value)?  resumed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PushRegistrationStartedEvent() when started != null:
return started(_that);case PushRegistrationTokenReceivedEvent() when tokenReceived != null:
return tokenReceived(_that);case PushRegistrationResumedEvent() when resumed != null:
return resumed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PushRegistrationStartedEvent value)  started,required TResult Function( PushRegistrationTokenReceivedEvent value)  tokenReceived,required TResult Function( PushRegistrationResumedEvent value)  resumed,}){
final _that = this;
switch (_that) {
case PushRegistrationStartedEvent():
return started(_that);case PushRegistrationTokenReceivedEvent():
return tokenReceived(_that);case PushRegistrationResumedEvent():
return resumed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PushRegistrationStartedEvent value)?  started,TResult? Function( PushRegistrationTokenReceivedEvent value)?  tokenReceived,TResult? Function( PushRegistrationResumedEvent value)?  resumed,}){
final _that = this;
switch (_that) {
case PushRegistrationStartedEvent() when started != null:
return started(_that);case PushRegistrationTokenReceivedEvent() when tokenReceived != null:
return tokenReceived(_that);case PushRegistrationResumedEvent() when resumed != null:
return resumed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( PushToken token)?  tokenReceived,TResult Function()?  resumed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PushRegistrationStartedEvent() when started != null:
return started();case PushRegistrationTokenReceivedEvent() when tokenReceived != null:
return tokenReceived(_that.token);case PushRegistrationResumedEvent() when resumed != null:
return resumed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( PushToken token)  tokenReceived,required TResult Function()  resumed,}) {final _that = this;
switch (_that) {
case PushRegistrationStartedEvent():
return started();case PushRegistrationTokenReceivedEvent():
return tokenReceived(_that.token);case PushRegistrationResumedEvent():
return resumed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( PushToken token)?  tokenReceived,TResult? Function()?  resumed,}) {final _that = this;
switch (_that) {
case PushRegistrationStartedEvent() when started != null:
return started();case PushRegistrationTokenReceivedEvent() when tokenReceived != null:
return tokenReceived(_that.token);case PushRegistrationResumedEvent() when resumed != null:
return resumed();case _:
  return null;

}
}

}

/// @nodoc


class PushRegistrationStartedEvent implements PushRegistrationEvent {
  const PushRegistrationStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushRegistrationStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PushRegistrationEvent.started()';
}


}




/// @nodoc


class PushRegistrationTokenReceivedEvent implements PushRegistrationEvent {
  const PushRegistrationTokenReceivedEvent(this.token);
  

 final  PushToken token;

/// Create a copy of PushRegistrationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PushRegistrationTokenReceivedEventCopyWith<PushRegistrationTokenReceivedEvent> get copyWith => _$PushRegistrationTokenReceivedEventCopyWithImpl<PushRegistrationTokenReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushRegistrationTokenReceivedEvent&&(identical(other.token, token) || other.token == token));
}


@override
int get hashCode => Object.hash(runtimeType,token);

@override
String toString() {
  return 'PushRegistrationEvent.tokenReceived(token: $token)';
}


}

/// @nodoc
abstract mixin class $PushRegistrationTokenReceivedEventCopyWith<$Res> implements $PushRegistrationEventCopyWith<$Res> {
  factory $PushRegistrationTokenReceivedEventCopyWith(PushRegistrationTokenReceivedEvent value, $Res Function(PushRegistrationTokenReceivedEvent) _then) = _$PushRegistrationTokenReceivedEventCopyWithImpl;
@useResult
$Res call({
 PushToken token
});


$PushTokenCopyWith<$Res> get token;

}
/// @nodoc
class _$PushRegistrationTokenReceivedEventCopyWithImpl<$Res>
    implements $PushRegistrationTokenReceivedEventCopyWith<$Res> {
  _$PushRegistrationTokenReceivedEventCopyWithImpl(this._self, this._then);

  final PushRegistrationTokenReceivedEvent _self;
  final $Res Function(PushRegistrationTokenReceivedEvent) _then;

/// Create a copy of PushRegistrationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? token = null,}) {
  return _then(PushRegistrationTokenReceivedEvent(
null == token ? _self.token : token // ignore: cast_nullable_to_non_nullable
as PushToken,
  ));
}

/// Create a copy of PushRegistrationEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PushTokenCopyWith<$Res> get token {
  
  return $PushTokenCopyWith<$Res>(_self.token, (value) {
    return _then(_self.copyWith(token: value));
  });
}
}

/// @nodoc


class PushRegistrationResumedEvent implements PushRegistrationEvent {
  const PushRegistrationResumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushRegistrationResumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PushRegistrationEvent.resumed()';
}


}




/// @nodoc
mixin _$PushRegistrationState {

/// How far the registration has got.
 PushRegistrationPhase get phase;/// Why the last step failed; null when it worked. Nothing shows it: a
/// push that can't be registered is retried, not reported.
 ApplicationError? get error;
/// Create a copy of PushRegistrationState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PushRegistrationStateCopyWith<PushRegistrationState> get copyWith => _$PushRegistrationStateCopyWithImpl<PushRegistrationState>(this as PushRegistrationState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushRegistrationState&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,phase,error);

@override
String toString() {
  return 'PushRegistrationState(phase: $phase, error: $error)';
}


}

/// @nodoc
abstract mixin class $PushRegistrationStateCopyWith<$Res>  {
  factory $PushRegistrationStateCopyWith(PushRegistrationState value, $Res Function(PushRegistrationState) _then) = _$PushRegistrationStateCopyWithImpl;
@useResult
$Res call({
 PushRegistrationPhase phase, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$PushRegistrationStateCopyWithImpl<$Res>
    implements $PushRegistrationStateCopyWith<$Res> {
  _$PushRegistrationStateCopyWithImpl(this._self, this._then);

  final PushRegistrationState _self;
  final $Res Function(PushRegistrationState) _then;

/// Create a copy of PushRegistrationState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? phase = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as PushRegistrationPhase,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of PushRegistrationState
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


/// Adds pattern-matching-related methods to [PushRegistrationState].
extension PushRegistrationStatePatterns on PushRegistrationState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PushRegistrationState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PushRegistrationState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PushRegistrationState value)  $default,){
final _that = this;
switch (_that) {
case _PushRegistrationState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PushRegistrationState value)?  $default,){
final _that = this;
switch (_that) {
case _PushRegistrationState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( PushRegistrationPhase phase,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PushRegistrationState() when $default != null:
return $default(_that.phase,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( PushRegistrationPhase phase,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _PushRegistrationState():
return $default(_that.phase,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( PushRegistrationPhase phase,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _PushRegistrationState() when $default != null:
return $default(_that.phase,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _PushRegistrationState implements PushRegistrationState {
  const _PushRegistrationState({this.phase = PushRegistrationPhase.starting, this.error});
  

/// How far the registration has got.
@override@JsonKey() final  PushRegistrationPhase phase;
/// Why the last step failed; null when it worked. Nothing shows it: a
/// push that can't be registered is retried, not reported.
@override final  ApplicationError? error;

/// Create a copy of PushRegistrationState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PushRegistrationStateCopyWith<_PushRegistrationState> get copyWith => __$PushRegistrationStateCopyWithImpl<_PushRegistrationState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PushRegistrationState&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,phase,error);

@override
String toString() {
  return 'PushRegistrationState(phase: $phase, error: $error)';
}


}

/// @nodoc
abstract mixin class _$PushRegistrationStateCopyWith<$Res> implements $PushRegistrationStateCopyWith<$Res> {
  factory _$PushRegistrationStateCopyWith(_PushRegistrationState value, $Res Function(_PushRegistrationState) _then) = __$PushRegistrationStateCopyWithImpl;
@override @useResult
$Res call({
 PushRegistrationPhase phase, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$PushRegistrationStateCopyWithImpl<$Res>
    implements _$PushRegistrationStateCopyWith<$Res> {
  __$PushRegistrationStateCopyWithImpl(this._self, this._then);

  final _PushRegistrationState _self;
  final $Res Function(_PushRegistrationState) _then;

/// Create a copy of PushRegistrationState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? phase = null,Object? error = freezed,}) {
  return _then(_PushRegistrationState(
phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as PushRegistrationPhase,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of PushRegistrationState
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
