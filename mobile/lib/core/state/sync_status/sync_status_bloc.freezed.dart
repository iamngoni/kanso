// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'sync_status_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SyncStatusEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncStatusEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncStatusEvent()';
}


}

/// @nodoc
class $SyncStatusEventCopyWith<$Res>  {
$SyncStatusEventCopyWith(SyncStatusEvent _, $Res Function(SyncStatusEvent) __);
}


/// Adds pattern-matching-related methods to [SyncStatusEvent].
extension SyncStatusEventPatterns on SyncStatusEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SyncStatusStartedEvent value)?  started,TResult Function( SyncStatusSyncRequestedEvent value)?  syncRequested,TResult Function( SyncStatusForegroundChangedEvent value)?  foregroundChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SyncStatusStartedEvent() when started != null:
return started(_that);case SyncStatusSyncRequestedEvent() when syncRequested != null:
return syncRequested(_that);case SyncStatusForegroundChangedEvent() when foregroundChanged != null:
return foregroundChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SyncStatusStartedEvent value)  started,required TResult Function( SyncStatusSyncRequestedEvent value)  syncRequested,required TResult Function( SyncStatusForegroundChangedEvent value)  foregroundChanged,}){
final _that = this;
switch (_that) {
case SyncStatusStartedEvent():
return started(_that);case SyncStatusSyncRequestedEvent():
return syncRequested(_that);case SyncStatusForegroundChangedEvent():
return foregroundChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SyncStatusStartedEvent value)?  started,TResult? Function( SyncStatusSyncRequestedEvent value)?  syncRequested,TResult? Function( SyncStatusForegroundChangedEvent value)?  foregroundChanged,}){
final _that = this;
switch (_that) {
case SyncStatusStartedEvent() when started != null:
return started(_that);case SyncStatusSyncRequestedEvent() when syncRequested != null:
return syncRequested(_that);case SyncStatusForegroundChangedEvent() when foregroundChanged != null:
return foregroundChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  syncRequested,TResult Function( bool foreground)?  foregroundChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SyncStatusStartedEvent() when started != null:
return started();case SyncStatusSyncRequestedEvent() when syncRequested != null:
return syncRequested();case SyncStatusForegroundChangedEvent() when foregroundChanged != null:
return foregroundChanged(_that.foreground);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  syncRequested,required TResult Function( bool foreground)  foregroundChanged,}) {final _that = this;
switch (_that) {
case SyncStatusStartedEvent():
return started();case SyncStatusSyncRequestedEvent():
return syncRequested();case SyncStatusForegroundChangedEvent():
return foregroundChanged(_that.foreground);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  syncRequested,TResult? Function( bool foreground)?  foregroundChanged,}) {final _that = this;
switch (_that) {
case SyncStatusStartedEvent() when started != null:
return started();case SyncStatusSyncRequestedEvent() when syncRequested != null:
return syncRequested();case SyncStatusForegroundChangedEvent() when foregroundChanged != null:
return foregroundChanged(_that.foreground);case _:
  return null;

}
}

}

/// @nodoc


class SyncStatusStartedEvent implements SyncStatusEvent {
  const SyncStatusStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncStatusStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncStatusEvent.started()';
}


}




/// @nodoc


class SyncStatusSyncRequestedEvent implements SyncStatusEvent {
  const SyncStatusSyncRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncStatusSyncRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncStatusEvent.syncRequested()';
}


}




/// @nodoc


class SyncStatusForegroundChangedEvent implements SyncStatusEvent {
  const SyncStatusForegroundChangedEvent({required this.foreground});
  

 final  bool foreground;

/// Create a copy of SyncStatusEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncStatusForegroundChangedEventCopyWith<SyncStatusForegroundChangedEvent> get copyWith => _$SyncStatusForegroundChangedEventCopyWithImpl<SyncStatusForegroundChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncStatusForegroundChangedEvent&&(identical(other.foreground, foreground) || other.foreground == foreground));
}


@override
int get hashCode => Object.hash(runtimeType,foreground);

@override
String toString() {
  return 'SyncStatusEvent.foregroundChanged(foreground: $foreground)';
}


}

/// @nodoc
abstract mixin class $SyncStatusForegroundChangedEventCopyWith<$Res> implements $SyncStatusEventCopyWith<$Res> {
  factory $SyncStatusForegroundChangedEventCopyWith(SyncStatusForegroundChangedEvent value, $Res Function(SyncStatusForegroundChangedEvent) _then) = _$SyncStatusForegroundChangedEventCopyWithImpl;
@useResult
$Res call({
 bool foreground
});




}
/// @nodoc
class _$SyncStatusForegroundChangedEventCopyWithImpl<$Res>
    implements $SyncStatusForegroundChangedEventCopyWith<$Res> {
  _$SyncStatusForegroundChangedEventCopyWithImpl(this._self, this._then);

  final SyncStatusForegroundChangedEvent _self;
  final $Res Function(SyncStatusForegroundChangedEvent) _then;

/// Create a copy of SyncStatusEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? foreground = null,}) {
  return _then(SyncStatusForegroundChangedEvent(
foreground: null == foreground ? _self.foreground : foreground // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$SyncStatusState {

/// The status of the account and its sync; null until it is first read.
 SyncStatus? get status;/// Whether a sync the person asked for is running.
 bool get syncing;/// Why the last sync the person asked for failed.
 ApplicationError? get error;
/// Create a copy of SyncStatusState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncStatusStateCopyWith<SyncStatusState> get copyWith => _$SyncStatusStateCopyWithImpl<SyncStatusState>(this as SyncStatusState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncStatusState&&(identical(other.status, status) || other.status == status)&&(identical(other.syncing, syncing) || other.syncing == syncing)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,syncing,error);

@override
String toString() {
  return 'SyncStatusState(status: $status, syncing: $syncing, error: $error)';
}


}

/// @nodoc
abstract mixin class $SyncStatusStateCopyWith<$Res>  {
  factory $SyncStatusStateCopyWith(SyncStatusState value, $Res Function(SyncStatusState) _then) = _$SyncStatusStateCopyWithImpl;
@useResult
$Res call({
 SyncStatus? status, bool syncing, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$SyncStatusStateCopyWithImpl<$Res>
    implements $SyncStatusStateCopyWith<$Res> {
  _$SyncStatusStateCopyWithImpl(this._self, this._then);

  final SyncStatusState _self;
  final $Res Function(SyncStatusState) _then;

/// Create a copy of SyncStatusState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = freezed,Object? syncing = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: freezed == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as SyncStatus?,syncing: null == syncing ? _self.syncing : syncing // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of SyncStatusState
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


/// Adds pattern-matching-related methods to [SyncStatusState].
extension SyncStatusStatePatterns on SyncStatusState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SyncStatusState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SyncStatusState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SyncStatusState value)  $default,){
final _that = this;
switch (_that) {
case _SyncStatusState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SyncStatusState value)?  $default,){
final _that = this;
switch (_that) {
case _SyncStatusState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( SyncStatus? status,  bool syncing,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SyncStatusState() when $default != null:
return $default(_that.status,_that.syncing,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( SyncStatus? status,  bool syncing,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _SyncStatusState():
return $default(_that.status,_that.syncing,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( SyncStatus? status,  bool syncing,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _SyncStatusState() when $default != null:
return $default(_that.status,_that.syncing,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _SyncStatusState extends SyncStatusState {
  const _SyncStatusState({this.status, this.syncing = false, this.error}): super._();
  

/// The status of the account and its sync; null until it is first read.
@override final  SyncStatus? status;
/// Whether a sync the person asked for is running.
@override@JsonKey() final  bool syncing;
/// Why the last sync the person asked for failed.
@override final  ApplicationError? error;

/// Create a copy of SyncStatusState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SyncStatusStateCopyWith<_SyncStatusState> get copyWith => __$SyncStatusStateCopyWithImpl<_SyncStatusState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SyncStatusState&&(identical(other.status, status) || other.status == status)&&(identical(other.syncing, syncing) || other.syncing == syncing)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,syncing,error);

@override
String toString() {
  return 'SyncStatusState(status: $status, syncing: $syncing, error: $error)';
}


}

/// @nodoc
abstract mixin class _$SyncStatusStateCopyWith<$Res> implements $SyncStatusStateCopyWith<$Res> {
  factory _$SyncStatusStateCopyWith(_SyncStatusState value, $Res Function(_SyncStatusState) _then) = __$SyncStatusStateCopyWithImpl;
@override @useResult
$Res call({
 SyncStatus? status, bool syncing, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$SyncStatusStateCopyWithImpl<$Res>
    implements _$SyncStatusStateCopyWith<$Res> {
  __$SyncStatusStateCopyWithImpl(this._self, this._then);

  final _SyncStatusState _self;
  final $Res Function(_SyncStatusState) _then;

/// Create a copy of SyncStatusState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = freezed,Object? syncing = null,Object? error = freezed,}) {
  return _then(_SyncStatusState(
status: freezed == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as SyncStatus?,syncing: null == syncing ? _self.syncing : syncing // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of SyncStatusState
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
