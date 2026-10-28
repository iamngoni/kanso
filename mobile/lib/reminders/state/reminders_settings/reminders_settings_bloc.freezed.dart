// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'reminders_settings_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RemindersSettingsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RemindersSettingsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RemindersSettingsEvent()';
}


}

/// @nodoc
class $RemindersSettingsEventCopyWith<$Res>  {
$RemindersSettingsEventCopyWith(RemindersSettingsEvent _, $Res Function(RemindersSettingsEvent) __);
}


/// Adds pattern-matching-related methods to [RemindersSettingsEvent].
extension RemindersSettingsEventPatterns on RemindersSettingsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RemindersSettingsStartedEvent value)?  started,TResult Function( RemindersSettingsResumedEvent value)?  resumed,TResult Function( RemindersSettingsToggledEvent value)?  toggled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RemindersSettingsStartedEvent() when started != null:
return started(_that);case RemindersSettingsResumedEvent() when resumed != null:
return resumed(_that);case RemindersSettingsToggledEvent() when toggled != null:
return toggled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RemindersSettingsStartedEvent value)  started,required TResult Function( RemindersSettingsResumedEvent value)  resumed,required TResult Function( RemindersSettingsToggledEvent value)  toggled,}){
final _that = this;
switch (_that) {
case RemindersSettingsStartedEvent():
return started(_that);case RemindersSettingsResumedEvent():
return resumed(_that);case RemindersSettingsToggledEvent():
return toggled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RemindersSettingsStartedEvent value)?  started,TResult? Function( RemindersSettingsResumedEvent value)?  resumed,TResult? Function( RemindersSettingsToggledEvent value)?  toggled,}){
final _that = this;
switch (_that) {
case RemindersSettingsStartedEvent() when started != null:
return started(_that);case RemindersSettingsResumedEvent() when resumed != null:
return resumed(_that);case RemindersSettingsToggledEvent() when toggled != null:
return toggled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  resumed,TResult Function( bool enabled)?  toggled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RemindersSettingsStartedEvent() when started != null:
return started();case RemindersSettingsResumedEvent() when resumed != null:
return resumed();case RemindersSettingsToggledEvent() when toggled != null:
return toggled(_that.enabled);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  resumed,required TResult Function( bool enabled)  toggled,}) {final _that = this;
switch (_that) {
case RemindersSettingsStartedEvent():
return started();case RemindersSettingsResumedEvent():
return resumed();case RemindersSettingsToggledEvent():
return toggled(_that.enabled);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  resumed,TResult? Function( bool enabled)?  toggled,}) {final _that = this;
switch (_that) {
case RemindersSettingsStartedEvent() when started != null:
return started();case RemindersSettingsResumedEvent() when resumed != null:
return resumed();case RemindersSettingsToggledEvent() when toggled != null:
return toggled(_that.enabled);case _:
  return null;

}
}

}

/// @nodoc


class RemindersSettingsStartedEvent implements RemindersSettingsEvent {
  const RemindersSettingsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RemindersSettingsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RemindersSettingsEvent.started()';
}


}




/// @nodoc


class RemindersSettingsResumedEvent implements RemindersSettingsEvent {
  const RemindersSettingsResumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RemindersSettingsResumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RemindersSettingsEvent.resumed()';
}


}




/// @nodoc


class RemindersSettingsToggledEvent implements RemindersSettingsEvent {
  const RemindersSettingsToggledEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of RemindersSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RemindersSettingsToggledEventCopyWith<RemindersSettingsToggledEvent> get copyWith => _$RemindersSettingsToggledEventCopyWithImpl<RemindersSettingsToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RemindersSettingsToggledEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'RemindersSettingsEvent.toggled(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $RemindersSettingsToggledEventCopyWith<$Res> implements $RemindersSettingsEventCopyWith<$Res> {
  factory $RemindersSettingsToggledEventCopyWith(RemindersSettingsToggledEvent value, $Res Function(RemindersSettingsToggledEvent) _then) = _$RemindersSettingsToggledEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$RemindersSettingsToggledEventCopyWithImpl<$Res>
    implements $RemindersSettingsToggledEventCopyWith<$Res> {
  _$RemindersSettingsToggledEventCopyWithImpl(this._self, this._then);

  final RemindersSettingsToggledEvent _self;
  final $Res Function(RemindersSettingsToggledEvent) _then;

/// Create a copy of RemindersSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(RemindersSettingsToggledEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$RemindersSettingsState {

 LoadStatus get status;/// The switch.
 bool get enabled; ReminderPermission get permission; ApplicationError? get error;
/// Create a copy of RemindersSettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RemindersSettingsStateCopyWith<RemindersSettingsState> get copyWith => _$RemindersSettingsStateCopyWithImpl<RemindersSettingsState>(this as RemindersSettingsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RemindersSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.permission, permission) || other.permission == permission)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,enabled,permission,error);

@override
String toString() {
  return 'RemindersSettingsState(status: $status, enabled: $enabled, permission: $permission, error: $error)';
}


}

/// @nodoc
abstract mixin class $RemindersSettingsStateCopyWith<$Res>  {
  factory $RemindersSettingsStateCopyWith(RemindersSettingsState value, $Res Function(RemindersSettingsState) _then) = _$RemindersSettingsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, bool enabled, ReminderPermission permission, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$RemindersSettingsStateCopyWithImpl<$Res>
    implements $RemindersSettingsStateCopyWith<$Res> {
  _$RemindersSettingsStateCopyWithImpl(this._self, this._then);

  final RemindersSettingsState _self;
  final $Res Function(RemindersSettingsState) _then;

/// Create a copy of RemindersSettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? enabled = null,Object? permission = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,permission: null == permission ? _self.permission : permission // ignore: cast_nullable_to_non_nullable
as ReminderPermission,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of RemindersSettingsState
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


/// Adds pattern-matching-related methods to [RemindersSettingsState].
extension RemindersSettingsStatePatterns on RemindersSettingsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RemindersSettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RemindersSettingsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RemindersSettingsState value)  $default,){
final _that = this;
switch (_that) {
case _RemindersSettingsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RemindersSettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _RemindersSettingsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  bool enabled,  ReminderPermission permission,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RemindersSettingsState() when $default != null:
return $default(_that.status,_that.enabled,_that.permission,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  bool enabled,  ReminderPermission permission,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _RemindersSettingsState():
return $default(_that.status,_that.enabled,_that.permission,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  bool enabled,  ReminderPermission permission,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _RemindersSettingsState() when $default != null:
return $default(_that.status,_that.enabled,_that.permission,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _RemindersSettingsState extends RemindersSettingsState {
  const _RemindersSettingsState({this.status = LoadStatus.loading, this.enabled = true, this.permission = ReminderPermission.notDetermined, this.error}): super._();
  

@override@JsonKey() final  LoadStatus status;
/// The switch.
@override@JsonKey() final  bool enabled;
@override@JsonKey() final  ReminderPermission permission;
@override final  ApplicationError? error;

/// Create a copy of RemindersSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RemindersSettingsStateCopyWith<_RemindersSettingsState> get copyWith => __$RemindersSettingsStateCopyWithImpl<_RemindersSettingsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RemindersSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.permission, permission) || other.permission == permission)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,enabled,permission,error);

@override
String toString() {
  return 'RemindersSettingsState(status: $status, enabled: $enabled, permission: $permission, error: $error)';
}


}

/// @nodoc
abstract mixin class _$RemindersSettingsStateCopyWith<$Res> implements $RemindersSettingsStateCopyWith<$Res> {
  factory _$RemindersSettingsStateCopyWith(_RemindersSettingsState value, $Res Function(_RemindersSettingsState) _then) = __$RemindersSettingsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, bool enabled, ReminderPermission permission, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$RemindersSettingsStateCopyWithImpl<$Res>
    implements _$RemindersSettingsStateCopyWith<$Res> {
  __$RemindersSettingsStateCopyWithImpl(this._self, this._then);

  final _RemindersSettingsState _self;
  final $Res Function(_RemindersSettingsState) _then;

/// Create a copy of RemindersSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? enabled = null,Object? permission = null,Object? error = freezed,}) {
  return _then(_RemindersSettingsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,permission: null == permission ? _self.permission : permission // ignore: cast_nullable_to_non_nullable
as ReminderPermission,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of RemindersSettingsState
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
