// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voice_settings_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$VoiceSettingsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceSettingsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'VoiceSettingsEvent()';
}


}

/// @nodoc
class $VoiceSettingsEventCopyWith<$Res>  {
$VoiceSettingsEventCopyWith(VoiceSettingsEvent _, $Res Function(VoiceSettingsEvent) __);
}


/// Adds pattern-matching-related methods to [VoiceSettingsEvent].
extension VoiceSettingsEventPatterns on VoiceSettingsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( VoiceSettingsStartedEvent value)?  started,TResult Function( VoiceSettingsSpokenCommandsChangedEvent value)?  spokenCommandsChanged,TResult Function( VoiceSettingsSystemSettingsRequestedEvent value)?  systemSettingsRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case VoiceSettingsStartedEvent() when started != null:
return started(_that);case VoiceSettingsSpokenCommandsChangedEvent() when spokenCommandsChanged != null:
return spokenCommandsChanged(_that);case VoiceSettingsSystemSettingsRequestedEvent() when systemSettingsRequested != null:
return systemSettingsRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( VoiceSettingsStartedEvent value)  started,required TResult Function( VoiceSettingsSpokenCommandsChangedEvent value)  spokenCommandsChanged,required TResult Function( VoiceSettingsSystemSettingsRequestedEvent value)  systemSettingsRequested,}){
final _that = this;
switch (_that) {
case VoiceSettingsStartedEvent():
return started(_that);case VoiceSettingsSpokenCommandsChangedEvent():
return spokenCommandsChanged(_that);case VoiceSettingsSystemSettingsRequestedEvent():
return systemSettingsRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( VoiceSettingsStartedEvent value)?  started,TResult? Function( VoiceSettingsSpokenCommandsChangedEvent value)?  spokenCommandsChanged,TResult? Function( VoiceSettingsSystemSettingsRequestedEvent value)?  systemSettingsRequested,}){
final _that = this;
switch (_that) {
case VoiceSettingsStartedEvent() when started != null:
return started(_that);case VoiceSettingsSpokenCommandsChangedEvent() when spokenCommandsChanged != null:
return spokenCommandsChanged(_that);case VoiceSettingsSystemSettingsRequestedEvent() when systemSettingsRequested != null:
return systemSettingsRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool on)?  spokenCommandsChanged,TResult Function()?  systemSettingsRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case VoiceSettingsStartedEvent() when started != null:
return started();case VoiceSettingsSpokenCommandsChangedEvent() when spokenCommandsChanged != null:
return spokenCommandsChanged(_that.on);case VoiceSettingsSystemSettingsRequestedEvent() when systemSettingsRequested != null:
return systemSettingsRequested();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool on)  spokenCommandsChanged,required TResult Function()  systemSettingsRequested,}) {final _that = this;
switch (_that) {
case VoiceSettingsStartedEvent():
return started();case VoiceSettingsSpokenCommandsChangedEvent():
return spokenCommandsChanged(_that.on);case VoiceSettingsSystemSettingsRequestedEvent():
return systemSettingsRequested();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool on)?  spokenCommandsChanged,TResult? Function()?  systemSettingsRequested,}) {final _that = this;
switch (_that) {
case VoiceSettingsStartedEvent() when started != null:
return started();case VoiceSettingsSpokenCommandsChangedEvent() when spokenCommandsChanged != null:
return spokenCommandsChanged(_that.on);case VoiceSettingsSystemSettingsRequestedEvent() when systemSettingsRequested != null:
return systemSettingsRequested();case _:
  return null;

}
}

}

/// @nodoc


class VoiceSettingsStartedEvent implements VoiceSettingsEvent {
  const VoiceSettingsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceSettingsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'VoiceSettingsEvent.started()';
}


}




/// @nodoc


class VoiceSettingsSpokenCommandsChangedEvent implements VoiceSettingsEvent {
  const VoiceSettingsSpokenCommandsChangedEvent({required this.on});
  

 final  bool on;

/// Create a copy of VoiceSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceSettingsSpokenCommandsChangedEventCopyWith<VoiceSettingsSpokenCommandsChangedEvent> get copyWith => _$VoiceSettingsSpokenCommandsChangedEventCopyWithImpl<VoiceSettingsSpokenCommandsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceSettingsSpokenCommandsChangedEvent&&(identical(other.on, on) || other.on == on));
}


@override
int get hashCode => Object.hash(runtimeType,on);

@override
String toString() {
  return 'VoiceSettingsEvent.spokenCommandsChanged(on: $on)';
}


}

/// @nodoc
abstract mixin class $VoiceSettingsSpokenCommandsChangedEventCopyWith<$Res> implements $VoiceSettingsEventCopyWith<$Res> {
  factory $VoiceSettingsSpokenCommandsChangedEventCopyWith(VoiceSettingsSpokenCommandsChangedEvent value, $Res Function(VoiceSettingsSpokenCommandsChangedEvent) _then) = _$VoiceSettingsSpokenCommandsChangedEventCopyWithImpl;
@useResult
$Res call({
 bool on
});




}
/// @nodoc
class _$VoiceSettingsSpokenCommandsChangedEventCopyWithImpl<$Res>
    implements $VoiceSettingsSpokenCommandsChangedEventCopyWith<$Res> {
  _$VoiceSettingsSpokenCommandsChangedEventCopyWithImpl(this._self, this._then);

  final VoiceSettingsSpokenCommandsChangedEvent _self;
  final $Res Function(VoiceSettingsSpokenCommandsChangedEvent) _then;

/// Create a copy of VoiceSettingsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? on = null,}) {
  return _then(VoiceSettingsSpokenCommandsChangedEvent(
on: null == on ? _self.on : on // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class VoiceSettingsSystemSettingsRequestedEvent implements VoiceSettingsEvent {
  const VoiceSettingsSystemSettingsRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceSettingsSystemSettingsRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'VoiceSettingsEvent.systemSettingsRequested()';
}


}




/// @nodoc
mixin _$VoiceSettingsState {

 LoadStatus get status; VoiceSettings get settings;/// What the recognizer can do; null until asked.
 DictationCapability? get capability;
/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceSettingsStateCopyWith<VoiceSettingsState> get copyWith => _$VoiceSettingsStateCopyWithImpl<VoiceSettingsState>(this as VoiceSettingsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.settings, settings) || other.settings == settings)&&(identical(other.capability, capability) || other.capability == capability));
}


@override
int get hashCode => Object.hash(runtimeType,status,settings,capability);

@override
String toString() {
  return 'VoiceSettingsState(status: $status, settings: $settings, capability: $capability)';
}


}

/// @nodoc
abstract mixin class $VoiceSettingsStateCopyWith<$Res>  {
  factory $VoiceSettingsStateCopyWith(VoiceSettingsState value, $Res Function(VoiceSettingsState) _then) = _$VoiceSettingsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, VoiceSettings settings, DictationCapability? capability
});


$VoiceSettingsCopyWith<$Res> get settings;$DictationCapabilityCopyWith<$Res>? get capability;

}
/// @nodoc
class _$VoiceSettingsStateCopyWithImpl<$Res>
    implements $VoiceSettingsStateCopyWith<$Res> {
  _$VoiceSettingsStateCopyWithImpl(this._self, this._then);

  final VoiceSettingsState _self;
  final $Res Function(VoiceSettingsState) _then;

/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? settings = null,Object? capability = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,settings: null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as VoiceSettings,capability: freezed == capability ? _self.capability : capability // ignore: cast_nullable_to_non_nullable
as DictationCapability?,
  ));
}
/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$VoiceSettingsCopyWith<$Res> get settings {
  
  return $VoiceSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DictationCapabilityCopyWith<$Res>? get capability {
    if (_self.capability == null) {
    return null;
  }

  return $DictationCapabilityCopyWith<$Res>(_self.capability!, (value) {
    return _then(_self.copyWith(capability: value));
  });
}
}


/// Adds pattern-matching-related methods to [VoiceSettingsState].
extension VoiceSettingsStatePatterns on VoiceSettingsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VoiceSettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VoiceSettingsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VoiceSettingsState value)  $default,){
final _that = this;
switch (_that) {
case _VoiceSettingsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VoiceSettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _VoiceSettingsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  VoiceSettings settings,  DictationCapability? capability)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VoiceSettingsState() when $default != null:
return $default(_that.status,_that.settings,_that.capability);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  VoiceSettings settings,  DictationCapability? capability)  $default,) {final _that = this;
switch (_that) {
case _VoiceSettingsState():
return $default(_that.status,_that.settings,_that.capability);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  VoiceSettings settings,  DictationCapability? capability)?  $default,) {final _that = this;
switch (_that) {
case _VoiceSettingsState() when $default != null:
return $default(_that.status,_that.settings,_that.capability);case _:
  return null;

}
}

}

/// @nodoc


class _VoiceSettingsState implements VoiceSettingsState {
  const _VoiceSettingsState({this.status = LoadStatus.loading, this.settings = const VoiceSettings(), this.capability});
  

@override@JsonKey() final  LoadStatus status;
@override@JsonKey() final  VoiceSettings settings;
/// What the recognizer can do; null until asked.
@override final  DictationCapability? capability;

/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VoiceSettingsStateCopyWith<_VoiceSettingsState> get copyWith => __$VoiceSettingsStateCopyWithImpl<_VoiceSettingsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VoiceSettingsState&&(identical(other.status, status) || other.status == status)&&(identical(other.settings, settings) || other.settings == settings)&&(identical(other.capability, capability) || other.capability == capability));
}


@override
int get hashCode => Object.hash(runtimeType,status,settings,capability);

@override
String toString() {
  return 'VoiceSettingsState(status: $status, settings: $settings, capability: $capability)';
}


}

/// @nodoc
abstract mixin class _$VoiceSettingsStateCopyWith<$Res> implements $VoiceSettingsStateCopyWith<$Res> {
  factory _$VoiceSettingsStateCopyWith(_VoiceSettingsState value, $Res Function(_VoiceSettingsState) _then) = __$VoiceSettingsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, VoiceSettings settings, DictationCapability? capability
});


@override $VoiceSettingsCopyWith<$Res> get settings;@override $DictationCapabilityCopyWith<$Res>? get capability;

}
/// @nodoc
class __$VoiceSettingsStateCopyWithImpl<$Res>
    implements _$VoiceSettingsStateCopyWith<$Res> {
  __$VoiceSettingsStateCopyWithImpl(this._self, this._then);

  final _VoiceSettingsState _self;
  final $Res Function(_VoiceSettingsState) _then;

/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? settings = null,Object? capability = freezed,}) {
  return _then(_VoiceSettingsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,settings: null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as VoiceSettings,capability: freezed == capability ? _self.capability : capability // ignore: cast_nullable_to_non_nullable
as DictationCapability?,
  ));
}

/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$VoiceSettingsCopyWith<$Res> get settings {
  
  return $VoiceSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}/// Create a copy of VoiceSettingsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DictationCapabilityCopyWith<$Res>? get capability {
    if (_self.capability == null) {
    return null;
  }

  return $DictationCapabilityCopyWith<$Res>(_self.capability!, (value) {
    return _then(_self.copyWith(capability: value));
  });
}
}

// dart format on
