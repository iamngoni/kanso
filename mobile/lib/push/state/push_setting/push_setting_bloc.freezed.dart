// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'push_setting_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PushSettingEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushSettingEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PushSettingEvent()';
}


}

/// @nodoc
class $PushSettingEventCopyWith<$Res>  {
$PushSettingEventCopyWith(PushSettingEvent _, $Res Function(PushSettingEvent) __);
}


/// Adds pattern-matching-related methods to [PushSettingEvent].
extension PushSettingEventPatterns on PushSettingEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PushSettingStartedEvent value)?  started,TResult Function( PushSettingToggledEvent value)?  toggled,TResult Function( PushSettingErrorDismissedEvent value)?  errorDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PushSettingStartedEvent() when started != null:
return started(_that);case PushSettingToggledEvent() when toggled != null:
return toggled(_that);case PushSettingErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PushSettingStartedEvent value)  started,required TResult Function( PushSettingToggledEvent value)  toggled,required TResult Function( PushSettingErrorDismissedEvent value)  errorDismissed,}){
final _that = this;
switch (_that) {
case PushSettingStartedEvent():
return started(_that);case PushSettingToggledEvent():
return toggled(_that);case PushSettingErrorDismissedEvent():
return errorDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PushSettingStartedEvent value)?  started,TResult? Function( PushSettingToggledEvent value)?  toggled,TResult? Function( PushSettingErrorDismissedEvent value)?  errorDismissed,}){
final _that = this;
switch (_that) {
case PushSettingStartedEvent() when started != null:
return started(_that);case PushSettingToggledEvent() when toggled != null:
return toggled(_that);case PushSettingErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool enabled)?  toggled,TResult Function()?  errorDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PushSettingStartedEvent() when started != null:
return started();case PushSettingToggledEvent() when toggled != null:
return toggled(_that.enabled);case PushSettingErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool enabled)  toggled,required TResult Function()  errorDismissed,}) {final _that = this;
switch (_that) {
case PushSettingStartedEvent():
return started();case PushSettingToggledEvent():
return toggled(_that.enabled);case PushSettingErrorDismissedEvent():
return errorDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool enabled)?  toggled,TResult? Function()?  errorDismissed,}) {final _that = this;
switch (_that) {
case PushSettingStartedEvent() when started != null:
return started();case PushSettingToggledEvent() when toggled != null:
return toggled(_that.enabled);case PushSettingErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
  return null;

}
}

}

/// @nodoc


class PushSettingStartedEvent implements PushSettingEvent {
  const PushSettingStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushSettingStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PushSettingEvent.started()';
}


}




/// @nodoc


class PushSettingToggledEvent implements PushSettingEvent {
  const PushSettingToggledEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of PushSettingEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PushSettingToggledEventCopyWith<PushSettingToggledEvent> get copyWith => _$PushSettingToggledEventCopyWithImpl<PushSettingToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushSettingToggledEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'PushSettingEvent.toggled(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $PushSettingToggledEventCopyWith<$Res> implements $PushSettingEventCopyWith<$Res> {
  factory $PushSettingToggledEventCopyWith(PushSettingToggledEvent value, $Res Function(PushSettingToggledEvent) _then) = _$PushSettingToggledEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$PushSettingToggledEventCopyWithImpl<$Res>
    implements $PushSettingToggledEventCopyWith<$Res> {
  _$PushSettingToggledEventCopyWithImpl(this._self, this._then);

  final PushSettingToggledEvent _self;
  final $Res Function(PushSettingToggledEvent) _then;

/// Create a copy of PushSettingEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(PushSettingToggledEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PushSettingErrorDismissedEvent implements PushSettingEvent {
  const PushSettingErrorDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushSettingErrorDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PushSettingEvent.errorDismissed()';
}


}




/// @nodoc
mixin _$PushSettingState {

 PushSettingStatus get status;/// Whether this phone is to be woken. On until turned off.
 bool get enabled;/// Whether a change is being made.
 bool get saving;/// Why the last change could not be made.
 ApplicationError? get error;
/// Create a copy of PushSettingState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PushSettingStateCopyWith<PushSettingState> get copyWith => _$PushSettingStateCopyWithImpl<PushSettingState>(this as PushSettingState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PushSettingState&&(identical(other.status, status) || other.status == status)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.saving, saving) || other.saving == saving)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,enabled,saving,error);

@override
String toString() {
  return 'PushSettingState(status: $status, enabled: $enabled, saving: $saving, error: $error)';
}


}

/// @nodoc
abstract mixin class $PushSettingStateCopyWith<$Res>  {
  factory $PushSettingStateCopyWith(PushSettingState value, $Res Function(PushSettingState) _then) = _$PushSettingStateCopyWithImpl;
@useResult
$Res call({
 PushSettingStatus status, bool enabled, bool saving, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$PushSettingStateCopyWithImpl<$Res>
    implements $PushSettingStateCopyWith<$Res> {
  _$PushSettingStateCopyWithImpl(this._self, this._then);

  final PushSettingState _self;
  final $Res Function(PushSettingState) _then;

/// Create a copy of PushSettingState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? enabled = null,Object? saving = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as PushSettingStatus,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,saving: null == saving ? _self.saving : saving // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of PushSettingState
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


/// Adds pattern-matching-related methods to [PushSettingState].
extension PushSettingStatePatterns on PushSettingState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PushSettingState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PushSettingState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PushSettingState value)  $default,){
final _that = this;
switch (_that) {
case _PushSettingState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PushSettingState value)?  $default,){
final _that = this;
switch (_that) {
case _PushSettingState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( PushSettingStatus status,  bool enabled,  bool saving,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PushSettingState() when $default != null:
return $default(_that.status,_that.enabled,_that.saving,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( PushSettingStatus status,  bool enabled,  bool saving,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _PushSettingState():
return $default(_that.status,_that.enabled,_that.saving,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( PushSettingStatus status,  bool enabled,  bool saving,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _PushSettingState() when $default != null:
return $default(_that.status,_that.enabled,_that.saving,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _PushSettingState implements PushSettingState {
  const _PushSettingState({this.status = PushSettingStatus.loading, this.enabled = true, this.saving = false, this.error});
  

@override@JsonKey() final  PushSettingStatus status;
/// Whether this phone is to be woken. On until turned off.
@override@JsonKey() final  bool enabled;
/// Whether a change is being made.
@override@JsonKey() final  bool saving;
/// Why the last change could not be made.
@override final  ApplicationError? error;

/// Create a copy of PushSettingState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PushSettingStateCopyWith<_PushSettingState> get copyWith => __$PushSettingStateCopyWithImpl<_PushSettingState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PushSettingState&&(identical(other.status, status) || other.status == status)&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.saving, saving) || other.saving == saving)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,enabled,saving,error);

@override
String toString() {
  return 'PushSettingState(status: $status, enabled: $enabled, saving: $saving, error: $error)';
}


}

/// @nodoc
abstract mixin class _$PushSettingStateCopyWith<$Res> implements $PushSettingStateCopyWith<$Res> {
  factory _$PushSettingStateCopyWith(_PushSettingState value, $Res Function(_PushSettingState) _then) = __$PushSettingStateCopyWithImpl;
@override @useResult
$Res call({
 PushSettingStatus status, bool enabled, bool saving, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$PushSettingStateCopyWithImpl<$Res>
    implements _$PushSettingStateCopyWith<$Res> {
  __$PushSettingStateCopyWithImpl(this._self, this._then);

  final _PushSettingState _self;
  final $Res Function(_PushSettingState) _then;

/// Create a copy of PushSettingState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? enabled = null,Object? saving = null,Object? error = freezed,}) {
  return _then(_PushSettingState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as PushSettingStatus,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,saving: null == saving ? _self.saving : saving // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of PushSettingState
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
