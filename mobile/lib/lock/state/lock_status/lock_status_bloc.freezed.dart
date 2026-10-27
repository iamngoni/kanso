// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lock_status_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LockStatusEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockStatusEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockStatusEvent()';
}


}

/// @nodoc
class $LockStatusEventCopyWith<$Res>  {
$LockStatusEventCopyWith(LockStatusEvent _, $Res Function(LockStatusEvent) __);
}


/// Adds pattern-matching-related methods to [LockStatusEvent].
extension LockStatusEventPatterns on LockStatusEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockStatusStartedEvent value)?  started,TResult Function( LockStatusRefreshedEvent value)?  refreshed,TResult Function( LockStatusLockedNowEvent value)?  lockedNow,TResult Function( LockStatusLifecycleChangedEvent value)?  lifecycleChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockStatusStartedEvent() when started != null:
return started(_that);case LockStatusRefreshedEvent() when refreshed != null:
return refreshed(_that);case LockStatusLockedNowEvent() when lockedNow != null:
return lockedNow(_that);case LockStatusLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockStatusStartedEvent value)  started,required TResult Function( LockStatusRefreshedEvent value)  refreshed,required TResult Function( LockStatusLockedNowEvent value)  lockedNow,required TResult Function( LockStatusLifecycleChangedEvent value)  lifecycleChanged,}){
final _that = this;
switch (_that) {
case LockStatusStartedEvent():
return started(_that);case LockStatusRefreshedEvent():
return refreshed(_that);case LockStatusLockedNowEvent():
return lockedNow(_that);case LockStatusLifecycleChangedEvent():
return lifecycleChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockStatusStartedEvent value)?  started,TResult? Function( LockStatusRefreshedEvent value)?  refreshed,TResult? Function( LockStatusLockedNowEvent value)?  lockedNow,TResult? Function( LockStatusLifecycleChangedEvent value)?  lifecycleChanged,}){
final _that = this;
switch (_that) {
case LockStatusStartedEvent() when started != null:
return started(_that);case LockStatusRefreshedEvent() when refreshed != null:
return refreshed(_that);case LockStatusLockedNowEvent() when lockedNow != null:
return lockedNow(_that);case LockStatusLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  refreshed,TResult Function()?  lockedNow,TResult Function( AppLifecycleState lifecycle)?  lifecycleChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockStatusStartedEvent() when started != null:
return started();case LockStatusRefreshedEvent() when refreshed != null:
return refreshed();case LockStatusLockedNowEvent() when lockedNow != null:
return lockedNow();case LockStatusLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that.lifecycle);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  refreshed,required TResult Function()  lockedNow,required TResult Function( AppLifecycleState lifecycle)  lifecycleChanged,}) {final _that = this;
switch (_that) {
case LockStatusStartedEvent():
return started();case LockStatusRefreshedEvent():
return refreshed();case LockStatusLockedNowEvent():
return lockedNow();case LockStatusLifecycleChangedEvent():
return lifecycleChanged(_that.lifecycle);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  refreshed,TResult? Function()?  lockedNow,TResult? Function( AppLifecycleState lifecycle)?  lifecycleChanged,}) {final _that = this;
switch (_that) {
case LockStatusStartedEvent() when started != null:
return started();case LockStatusRefreshedEvent() when refreshed != null:
return refreshed();case LockStatusLockedNowEvent() when lockedNow != null:
return lockedNow();case LockStatusLifecycleChangedEvent() when lifecycleChanged != null:
return lifecycleChanged(_that.lifecycle);case _:
  return null;

}
}

}

/// @nodoc


class LockStatusStartedEvent implements LockStatusEvent {
  const LockStatusStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockStatusStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockStatusEvent.started()';
}


}




/// @nodoc


class LockStatusRefreshedEvent implements LockStatusEvent {
  const LockStatusRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockStatusRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockStatusEvent.refreshed()';
}


}




/// @nodoc


class LockStatusLockedNowEvent implements LockStatusEvent {
  const LockStatusLockedNowEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockStatusLockedNowEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockStatusEvent.lockedNow()';
}


}




/// @nodoc


class LockStatusLifecycleChangedEvent implements LockStatusEvent {
  const LockStatusLifecycleChangedEvent(this.lifecycle);
  

 final  AppLifecycleState lifecycle;

/// Create a copy of LockStatusEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockStatusLifecycleChangedEventCopyWith<LockStatusLifecycleChangedEvent> get copyWith => _$LockStatusLifecycleChangedEventCopyWithImpl<LockStatusLifecycleChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockStatusLifecycleChangedEvent&&(identical(other.lifecycle, lifecycle) || other.lifecycle == lifecycle));
}


@override
int get hashCode => Object.hash(runtimeType,lifecycle);

@override
String toString() {
  return 'LockStatusEvent.lifecycleChanged(lifecycle: $lifecycle)';
}


}

/// @nodoc
abstract mixin class $LockStatusLifecycleChangedEventCopyWith<$Res> implements $LockStatusEventCopyWith<$Res> {
  factory $LockStatusLifecycleChangedEventCopyWith(LockStatusLifecycleChangedEvent value, $Res Function(LockStatusLifecycleChangedEvent) _then) = _$LockStatusLifecycleChangedEventCopyWithImpl;
@useResult
$Res call({
 AppLifecycleState lifecycle
});




}
/// @nodoc
class _$LockStatusLifecycleChangedEventCopyWithImpl<$Res>
    implements $LockStatusLifecycleChangedEventCopyWith<$Res> {
  _$LockStatusLifecycleChangedEventCopyWithImpl(this._self, this._then);

  final LockStatusLifecycleChangedEvent _self;
  final $Res Function(LockStatusLifecycleChangedEvent) _then;

/// Create a copy of LockStatusEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? lifecycle = null,}) {
  return _then(LockStatusLifecycleChangedEvent(
null == lifecycle ? _self.lifecycle : lifecycle // ignore: cast_nullable_to_non_nullable
as AppLifecycleState,
  ));
}


}

/// @nodoc
mixin _$LockStatusState {

/// Whether the status was read.
 bool get loaded;/// Whether any note or notebook is locked, here or on another device.
 bool get hasLockPassword;/// Whether the lock password has opened the locked notes.
 bool get unlocked;/// Whether biometrics are turned on for the locked notes on this phone.
 bool get biometricsOn;/// What biometric unlock the phone offers.
 BiometricKind get kind;
/// Create a copy of LockStatusState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockStatusStateCopyWith<LockStatusState> get copyWith => _$LockStatusStateCopyWithImpl<LockStatusState>(this as LockStatusState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockStatusState&&(identical(other.loaded, loaded) || other.loaded == loaded)&&(identical(other.hasLockPassword, hasLockPassword) || other.hasLockPassword == hasLockPassword)&&(identical(other.unlocked, unlocked) || other.unlocked == unlocked)&&(identical(other.biometricsOn, biometricsOn) || other.biometricsOn == biometricsOn)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,loaded,hasLockPassword,unlocked,biometricsOn,kind);

@override
String toString() {
  return 'LockStatusState(loaded: $loaded, hasLockPassword: $hasLockPassword, unlocked: $unlocked, biometricsOn: $biometricsOn, kind: $kind)';
}


}

/// @nodoc
abstract mixin class $LockStatusStateCopyWith<$Res>  {
  factory $LockStatusStateCopyWith(LockStatusState value, $Res Function(LockStatusState) _then) = _$LockStatusStateCopyWithImpl;
@useResult
$Res call({
 bool loaded, bool hasLockPassword, bool unlocked, bool biometricsOn, BiometricKind kind
});




}
/// @nodoc
class _$LockStatusStateCopyWithImpl<$Res>
    implements $LockStatusStateCopyWith<$Res> {
  _$LockStatusStateCopyWithImpl(this._self, this._then);

  final LockStatusState _self;
  final $Res Function(LockStatusState) _then;

/// Create a copy of LockStatusState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? loaded = null,Object? hasLockPassword = null,Object? unlocked = null,Object? biometricsOn = null,Object? kind = null,}) {
  return _then(_self.copyWith(
loaded: null == loaded ? _self.loaded : loaded // ignore: cast_nullable_to_non_nullable
as bool,hasLockPassword: null == hasLockPassword ? _self.hasLockPassword : hasLockPassword // ignore: cast_nullable_to_non_nullable
as bool,unlocked: null == unlocked ? _self.unlocked : unlocked // ignore: cast_nullable_to_non_nullable
as bool,biometricsOn: null == biometricsOn ? _self.biometricsOn : biometricsOn // ignore: cast_nullable_to_non_nullable
as bool,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BiometricKind,
  ));
}

}


/// Adds pattern-matching-related methods to [LockStatusState].
extension LockStatusStatePatterns on LockStatusState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LockStatusState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LockStatusState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LockStatusState value)  $default,){
final _that = this;
switch (_that) {
case _LockStatusState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LockStatusState value)?  $default,){
final _that = this;
switch (_that) {
case _LockStatusState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool loaded,  bool hasLockPassword,  bool unlocked,  bool biometricsOn,  BiometricKind kind)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LockStatusState() when $default != null:
return $default(_that.loaded,_that.hasLockPassword,_that.unlocked,_that.biometricsOn,_that.kind);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool loaded,  bool hasLockPassword,  bool unlocked,  bool biometricsOn,  BiometricKind kind)  $default,) {final _that = this;
switch (_that) {
case _LockStatusState():
return $default(_that.loaded,_that.hasLockPassword,_that.unlocked,_that.biometricsOn,_that.kind);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool loaded,  bool hasLockPassword,  bool unlocked,  bool biometricsOn,  BiometricKind kind)?  $default,) {final _that = this;
switch (_that) {
case _LockStatusState() when $default != null:
return $default(_that.loaded,_that.hasLockPassword,_that.unlocked,_that.biometricsOn,_that.kind);case _:
  return null;

}
}

}

/// @nodoc


class _LockStatusState extends LockStatusState {
  const _LockStatusState({this.loaded = false, this.hasLockPassword = false, this.unlocked = false, this.biometricsOn = false, this.kind = BiometricKind.none}): super._();
  

/// Whether the status was read.
@override@JsonKey() final  bool loaded;
/// Whether any note or notebook is locked, here or on another device.
@override@JsonKey() final  bool hasLockPassword;
/// Whether the lock password has opened the locked notes.
@override@JsonKey() final  bool unlocked;
/// Whether biometrics are turned on for the locked notes on this phone.
@override@JsonKey() final  bool biometricsOn;
/// What biometric unlock the phone offers.
@override@JsonKey() final  BiometricKind kind;

/// Create a copy of LockStatusState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LockStatusStateCopyWith<_LockStatusState> get copyWith => __$LockStatusStateCopyWithImpl<_LockStatusState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LockStatusState&&(identical(other.loaded, loaded) || other.loaded == loaded)&&(identical(other.hasLockPassword, hasLockPassword) || other.hasLockPassword == hasLockPassword)&&(identical(other.unlocked, unlocked) || other.unlocked == unlocked)&&(identical(other.biometricsOn, biometricsOn) || other.biometricsOn == biometricsOn)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,loaded,hasLockPassword,unlocked,biometricsOn,kind);

@override
String toString() {
  return 'LockStatusState(loaded: $loaded, hasLockPassword: $hasLockPassword, unlocked: $unlocked, biometricsOn: $biometricsOn, kind: $kind)';
}


}

/// @nodoc
abstract mixin class _$LockStatusStateCopyWith<$Res> implements $LockStatusStateCopyWith<$Res> {
  factory _$LockStatusStateCopyWith(_LockStatusState value, $Res Function(_LockStatusState) _then) = __$LockStatusStateCopyWithImpl;
@override @useResult
$Res call({
 bool loaded, bool hasLockPassword, bool unlocked, bool biometricsOn, BiometricKind kind
});




}
/// @nodoc
class __$LockStatusStateCopyWithImpl<$Res>
    implements _$LockStatusStateCopyWith<$Res> {
  __$LockStatusStateCopyWithImpl(this._self, this._then);

  final _LockStatusState _self;
  final $Res Function(_LockStatusState) _then;

/// Create a copy of LockStatusState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? loaded = null,Object? hasLockPassword = null,Object? unlocked = null,Object? biometricsOn = null,Object? kind = null,}) {
  return _then(_LockStatusState(
loaded: null == loaded ? _self.loaded : loaded // ignore: cast_nullable_to_non_nullable
as bool,hasLockPassword: null == hasLockPassword ? _self.hasLockPassword : hasLockPassword // ignore: cast_nullable_to_non_nullable
as bool,unlocked: null == unlocked ? _self.unlocked : unlocked // ignore: cast_nullable_to_non_nullable
as bool,biometricsOn: null == biometricsOn ? _self.biometricsOn : biometricsOn // ignore: cast_nullable_to_non_nullable
as bool,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BiometricKind,
  ));
}


}

// dart format on
