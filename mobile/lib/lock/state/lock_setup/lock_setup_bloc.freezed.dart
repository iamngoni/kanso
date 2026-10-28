// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lock_setup_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LockSetupEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockSetupEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockSetupEvent()';
}


}

/// @nodoc
class $LockSetupEventCopyWith<$Res>  {
$LockSetupEventCopyWith(LockSetupEvent _, $Res Function(LockSetupEvent) __);
}


/// Adds pattern-matching-related methods to [LockSetupEvent].
extension LockSetupEventPatterns on LockSetupEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockSetupStartedEvent value)?  started,TResult Function( LockSetupSubmittedEvent value)?  submitted,TResult Function( LockSetupConfirmedEvent value)?  confirmed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockSetupStartedEvent() when started != null:
return started(_that);case LockSetupSubmittedEvent() when submitted != null:
return submitted(_that);case LockSetupConfirmedEvent() when confirmed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockSetupStartedEvent value)  started,required TResult Function( LockSetupSubmittedEvent value)  submitted,required TResult Function( LockSetupConfirmedEvent value)  confirmed,}){
final _that = this;
switch (_that) {
case LockSetupStartedEvent():
return started(_that);case LockSetupSubmittedEvent():
return submitted(_that);case LockSetupConfirmedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockSetupStartedEvent value)?  started,TResult? Function( LockSetupSubmittedEvent value)?  submitted,TResult? Function( LockSetupConfirmedEvent value)?  confirmed,}){
final _that = this;
switch (_that) {
case LockSetupStartedEvent() when started != null:
return started(_that);case LockSetupSubmittedEvent() when submitted != null:
return submitted(_that);case LockSetupConfirmedEvent() when confirmed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( LockTarget target)?  started,TResult Function( String password,  String again)?  submitted,TResult Function()?  confirmed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockSetupStartedEvent() when started != null:
return started(_that.target);case LockSetupSubmittedEvent() when submitted != null:
return submitted(_that.password,_that.again);case LockSetupConfirmedEvent() when confirmed != null:
return confirmed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( LockTarget target)  started,required TResult Function( String password,  String again)  submitted,required TResult Function()  confirmed,}) {final _that = this;
switch (_that) {
case LockSetupStartedEvent():
return started(_that.target);case LockSetupSubmittedEvent():
return submitted(_that.password,_that.again);case LockSetupConfirmedEvent():
return confirmed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( LockTarget target)?  started,TResult? Function( String password,  String again)?  submitted,TResult? Function()?  confirmed,}) {final _that = this;
switch (_that) {
case LockSetupStartedEvent() when started != null:
return started(_that.target);case LockSetupSubmittedEvent() when submitted != null:
return submitted(_that.password,_that.again);case LockSetupConfirmedEvent() when confirmed != null:
return confirmed();case _:
  return null;

}
}

}

/// @nodoc


class LockSetupStartedEvent implements LockSetupEvent {
  const LockSetupStartedEvent(this.target);
  

 final  LockTarget target;

/// Create a copy of LockSetupEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockSetupStartedEventCopyWith<LockSetupStartedEvent> get copyWith => _$LockSetupStartedEventCopyWithImpl<LockSetupStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockSetupStartedEvent&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,target);

@override
String toString() {
  return 'LockSetupEvent.started(target: $target)';
}


}

/// @nodoc
abstract mixin class $LockSetupStartedEventCopyWith<$Res> implements $LockSetupEventCopyWith<$Res> {
  factory $LockSetupStartedEventCopyWith(LockSetupStartedEvent value, $Res Function(LockSetupStartedEvent) _then) = _$LockSetupStartedEventCopyWithImpl;
@useResult
$Res call({
 LockTarget target
});


$LockTargetCopyWith<$Res> get target;

}
/// @nodoc
class _$LockSetupStartedEventCopyWithImpl<$Res>
    implements $LockSetupStartedEventCopyWith<$Res> {
  _$LockSetupStartedEventCopyWithImpl(this._self, this._then);

  final LockSetupStartedEvent _self;
  final $Res Function(LockSetupStartedEvent) _then;

/// Create a copy of LockSetupEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? target = null,}) {
  return _then(LockSetupStartedEvent(
null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget,
  ));
}

/// Create a copy of LockSetupEvent
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


class LockSetupSubmittedEvent implements LockSetupEvent {
  const LockSetupSubmittedEvent(this.password, this.again);
  

 final  String password;
 final  String again;

/// Create a copy of LockSetupEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockSetupSubmittedEventCopyWith<LockSetupSubmittedEvent> get copyWith => _$LockSetupSubmittedEventCopyWithImpl<LockSetupSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockSetupSubmittedEvent&&(identical(other.password, password) || other.password == password)&&(identical(other.again, again) || other.again == again));
}


@override
int get hashCode => Object.hash(runtimeType,password,again);

@override
String toString() {
  return 'LockSetupEvent.submitted(password: $password, again: $again)';
}


}

/// @nodoc
abstract mixin class $LockSetupSubmittedEventCopyWith<$Res> implements $LockSetupEventCopyWith<$Res> {
  factory $LockSetupSubmittedEventCopyWith(LockSetupSubmittedEvent value, $Res Function(LockSetupSubmittedEvent) _then) = _$LockSetupSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String password, String again
});




}
/// @nodoc
class _$LockSetupSubmittedEventCopyWithImpl<$Res>
    implements $LockSetupSubmittedEventCopyWith<$Res> {
  _$LockSetupSubmittedEventCopyWithImpl(this._self, this._then);

  final LockSetupSubmittedEvent _self;
  final $Res Function(LockSetupSubmittedEvent) _then;

/// Create a copy of LockSetupEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = null,Object? again = null,}) {
  return _then(LockSetupSubmittedEvent(
null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,null == again ? _self.again : again // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LockSetupConfirmedEvent implements LockSetupEvent {
  const LockSetupConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockSetupConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockSetupEvent.confirmed()';
}


}




/// @nodoc
mixin _$LockSetupState {

 LockTarget? get target; LockMode get mode; LockFormStatus get status;/// How many notes were sealed, once [status] is done.
 int get count;/// Why it failed, with [LockFormStatus.failed].
 ApplicationError? get error;
/// Create a copy of LockSetupState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockSetupStateCopyWith<LockSetupState> get copyWith => _$LockSetupStateCopyWithImpl<LockSetupState>(this as LockSetupState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockSetupState&&(identical(other.target, target) || other.target == target)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.status, status) || other.status == status)&&(identical(other.count, count) || other.count == count)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,target,mode,status,count,error);

@override
String toString() {
  return 'LockSetupState(target: $target, mode: $mode, status: $status, count: $count, error: $error)';
}


}

/// @nodoc
abstract mixin class $LockSetupStateCopyWith<$Res>  {
  factory $LockSetupStateCopyWith(LockSetupState value, $Res Function(LockSetupState) _then) = _$LockSetupStateCopyWithImpl;
@useResult
$Res call({
 LockTarget? target, LockMode mode, LockFormStatus status, int count, ApplicationError? error
});


$LockTargetCopyWith<$Res>? get target;$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$LockSetupStateCopyWithImpl<$Res>
    implements $LockSetupStateCopyWith<$Res> {
  _$LockSetupStateCopyWithImpl(this._self, this._then);

  final LockSetupState _self;
  final $Res Function(LockSetupState) _then;

/// Create a copy of LockSetupState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? target = freezed,Object? mode = null,Object? status = null,Object? count = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
target: freezed == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget?,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as LockMode,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LockFormStatus,count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of LockSetupState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$LockTargetCopyWith<$Res>? get target {
    if (_self.target == null) {
    return null;
  }

  return $LockTargetCopyWith<$Res>(_self.target!, (value) {
    return _then(_self.copyWith(target: value));
  });
}/// Create a copy of LockSetupState
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


/// Adds pattern-matching-related methods to [LockSetupState].
extension LockSetupStatePatterns on LockSetupState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LockSetupState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LockSetupState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LockSetupState value)  $default,){
final _that = this;
switch (_that) {
case _LockSetupState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LockSetupState value)?  $default,){
final _that = this;
switch (_that) {
case _LockSetupState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LockTarget? target,  LockMode mode,  LockFormStatus status,  int count,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LockSetupState() when $default != null:
return $default(_that.target,_that.mode,_that.status,_that.count,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LockTarget? target,  LockMode mode,  LockFormStatus status,  int count,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _LockSetupState():
return $default(_that.target,_that.mode,_that.status,_that.count,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LockTarget? target,  LockMode mode,  LockFormStatus status,  int count,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _LockSetupState() when $default != null:
return $default(_that.target,_that.mode,_that.status,_that.count,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _LockSetupState implements LockSetupState {
  const _LockSetupState({this.target, this.mode = LockMode.setPassword, this.status = LockFormStatus.loading, this.count = 0, this.error});
  

@override final  LockTarget? target;
@override@JsonKey() final  LockMode mode;
@override@JsonKey() final  LockFormStatus status;
/// How many notes were sealed, once [status] is done.
@override@JsonKey() final  int count;
/// Why it failed, with [LockFormStatus.failed].
@override final  ApplicationError? error;

/// Create a copy of LockSetupState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LockSetupStateCopyWith<_LockSetupState> get copyWith => __$LockSetupStateCopyWithImpl<_LockSetupState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LockSetupState&&(identical(other.target, target) || other.target == target)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.status, status) || other.status == status)&&(identical(other.count, count) || other.count == count)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,target,mode,status,count,error);

@override
String toString() {
  return 'LockSetupState(target: $target, mode: $mode, status: $status, count: $count, error: $error)';
}


}

/// @nodoc
abstract mixin class _$LockSetupStateCopyWith<$Res> implements $LockSetupStateCopyWith<$Res> {
  factory _$LockSetupStateCopyWith(_LockSetupState value, $Res Function(_LockSetupState) _then) = __$LockSetupStateCopyWithImpl;
@override @useResult
$Res call({
 LockTarget? target, LockMode mode, LockFormStatus status, int count, ApplicationError? error
});


@override $LockTargetCopyWith<$Res>? get target;@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$LockSetupStateCopyWithImpl<$Res>
    implements _$LockSetupStateCopyWith<$Res> {
  __$LockSetupStateCopyWithImpl(this._self, this._then);

  final _LockSetupState _self;
  final $Res Function(_LockSetupState) _then;

/// Create a copy of LockSetupState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? target = freezed,Object? mode = null,Object? status = null,Object? count = null,Object? error = freezed,}) {
  return _then(_LockSetupState(
target: freezed == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget?,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as LockMode,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LockFormStatus,count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of LockSetupState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$LockTargetCopyWith<$Res>? get target {
    if (_self.target == null) {
    return null;
  }

  return $LockTargetCopyWith<$Res>(_self.target!, (value) {
    return _then(_self.copyWith(target: value));
  });
}/// Create a copy of LockSetupState
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
