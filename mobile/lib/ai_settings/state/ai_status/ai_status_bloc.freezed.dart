// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai_status_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AiStatusEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiStatusEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AiStatusEvent()';
}


}

/// @nodoc
class $AiStatusEventCopyWith<$Res>  {
$AiStatusEventCopyWith(AiStatusEvent _, $Res Function(AiStatusEvent) __);
}


/// Adds pattern-matching-related methods to [AiStatusEvent].
extension AiStatusEventPatterns on AiStatusEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AiStatusStartedEvent value)?  started,TResult Function( AiStatusSettingsChangedEvent value)?  settingsChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AiStatusStartedEvent() when started != null:
return started(_that);case AiStatusSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AiStatusStartedEvent value)  started,required TResult Function( AiStatusSettingsChangedEvent value)  settingsChanged,}){
final _that = this;
switch (_that) {
case AiStatusStartedEvent():
return started(_that);case AiStatusSettingsChangedEvent():
return settingsChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AiStatusStartedEvent value)?  started,TResult? Function( AiStatusSettingsChangedEvent value)?  settingsChanged,}){
final _that = this;
switch (_that) {
case AiStatusStartedEvent() when started != null:
return started(_that);case AiStatusSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( AiSettings settings)?  settingsChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AiStatusStartedEvent() when started != null:
return started();case AiStatusSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that.settings);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( AiSettings settings)  settingsChanged,}) {final _that = this;
switch (_that) {
case AiStatusStartedEvent():
return started();case AiStatusSettingsChangedEvent():
return settingsChanged(_that.settings);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( AiSettings settings)?  settingsChanged,}) {final _that = this;
switch (_that) {
case AiStatusStartedEvent() when started != null:
return started();case AiStatusSettingsChangedEvent() when settingsChanged != null:
return settingsChanged(_that.settings);case _:
  return null;

}
}

}

/// @nodoc


class AiStatusStartedEvent implements AiStatusEvent {
  const AiStatusStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiStatusStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AiStatusEvent.started()';
}


}




/// @nodoc


class AiStatusSettingsChangedEvent implements AiStatusEvent {
  const AiStatusSettingsChangedEvent(this.settings);
  

 final  AiSettings settings;

/// Create a copy of AiStatusEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiStatusSettingsChangedEventCopyWith<AiStatusSettingsChangedEvent> get copyWith => _$AiStatusSettingsChangedEventCopyWithImpl<AiStatusSettingsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiStatusSettingsChangedEvent&&(identical(other.settings, settings) || other.settings == settings));
}


@override
int get hashCode => Object.hash(runtimeType,settings);

@override
String toString() {
  return 'AiStatusEvent.settingsChanged(settings: $settings)';
}


}

/// @nodoc
abstract mixin class $AiStatusSettingsChangedEventCopyWith<$Res> implements $AiStatusEventCopyWith<$Res> {
  factory $AiStatusSettingsChangedEventCopyWith(AiStatusSettingsChangedEvent value, $Res Function(AiStatusSettingsChangedEvent) _then) = _$AiStatusSettingsChangedEventCopyWithImpl;
@useResult
$Res call({
 AiSettings settings
});


$AiSettingsCopyWith<$Res> get settings;

}
/// @nodoc
class _$AiStatusSettingsChangedEventCopyWithImpl<$Res>
    implements $AiStatusSettingsChangedEventCopyWith<$Res> {
  _$AiStatusSettingsChangedEventCopyWithImpl(this._self, this._then);

  final AiStatusSettingsChangedEvent _self;
  final $Res Function(AiStatusSettingsChangedEvent) _then;

/// Create a copy of AiStatusEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? settings = null,}) {
  return _then(AiStatusSettingsChangedEvent(
null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as AiSettings,
  ));
}

/// Create a copy of AiStatusEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<$Res> get settings {
  
  return $AiSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}
}

/// @nodoc
mixin _$AiStatusState {

/// Whether the settings have been read.
 bool get loaded; AiSettings get settings;
/// Create a copy of AiStatusState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiStatusStateCopyWith<AiStatusState> get copyWith => _$AiStatusStateCopyWithImpl<AiStatusState>(this as AiStatusState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiStatusState&&(identical(other.loaded, loaded) || other.loaded == loaded)&&(identical(other.settings, settings) || other.settings == settings));
}


@override
int get hashCode => Object.hash(runtimeType,loaded,settings);

@override
String toString() {
  return 'AiStatusState(loaded: $loaded, settings: $settings)';
}


}

/// @nodoc
abstract mixin class $AiStatusStateCopyWith<$Res>  {
  factory $AiStatusStateCopyWith(AiStatusState value, $Res Function(AiStatusState) _then) = _$AiStatusStateCopyWithImpl;
@useResult
$Res call({
 bool loaded, AiSettings settings
});


$AiSettingsCopyWith<$Res> get settings;

}
/// @nodoc
class _$AiStatusStateCopyWithImpl<$Res>
    implements $AiStatusStateCopyWith<$Res> {
  _$AiStatusStateCopyWithImpl(this._self, this._then);

  final AiStatusState _self;
  final $Res Function(AiStatusState) _then;

/// Create a copy of AiStatusState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? loaded = null,Object? settings = null,}) {
  return _then(_self.copyWith(
loaded: null == loaded ? _self.loaded : loaded // ignore: cast_nullable_to_non_nullable
as bool,settings: null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as AiSettings,
  ));
}
/// Create a copy of AiStatusState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<$Res> get settings {
  
  return $AiSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}
}


/// Adds pattern-matching-related methods to [AiStatusState].
extension AiStatusStatePatterns on AiStatusState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AiStatusState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AiStatusState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AiStatusState value)  $default,){
final _that = this;
switch (_that) {
case _AiStatusState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AiStatusState value)?  $default,){
final _that = this;
switch (_that) {
case _AiStatusState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool loaded,  AiSettings settings)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AiStatusState() when $default != null:
return $default(_that.loaded,_that.settings);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool loaded,  AiSettings settings)  $default,) {final _that = this;
switch (_that) {
case _AiStatusState():
return $default(_that.loaded,_that.settings);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool loaded,  AiSettings settings)?  $default,) {final _that = this;
switch (_that) {
case _AiStatusState() when $default != null:
return $default(_that.loaded,_that.settings);case _:
  return null;

}
}

}

/// @nodoc


class _AiStatusState extends AiStatusState {
  const _AiStatusState({this.loaded = false, this.settings = const AiSettings()}): super._();
  

/// Whether the settings have been read.
@override@JsonKey() final  bool loaded;
@override@JsonKey() final  AiSettings settings;

/// Create a copy of AiStatusState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AiStatusStateCopyWith<_AiStatusState> get copyWith => __$AiStatusStateCopyWithImpl<_AiStatusState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AiStatusState&&(identical(other.loaded, loaded) || other.loaded == loaded)&&(identical(other.settings, settings) || other.settings == settings));
}


@override
int get hashCode => Object.hash(runtimeType,loaded,settings);

@override
String toString() {
  return 'AiStatusState(loaded: $loaded, settings: $settings)';
}


}

/// @nodoc
abstract mixin class _$AiStatusStateCopyWith<$Res> implements $AiStatusStateCopyWith<$Res> {
  factory _$AiStatusStateCopyWith(_AiStatusState value, $Res Function(_AiStatusState) _then) = __$AiStatusStateCopyWithImpl;
@override @useResult
$Res call({
 bool loaded, AiSettings settings
});


@override $AiSettingsCopyWith<$Res> get settings;

}
/// @nodoc
class __$AiStatusStateCopyWithImpl<$Res>
    implements _$AiStatusStateCopyWith<$Res> {
  __$AiStatusStateCopyWithImpl(this._self, this._then);

  final _AiStatusState _self;
  final $Res Function(_AiStatusState) _then;

/// Create a copy of AiStatusState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? loaded = null,Object? settings = null,}) {
  return _then(_AiStatusState(
loaded: null == loaded ? _self.loaded : loaded // ignore: cast_nullable_to_non_nullable
as bool,settings: null == settings ? _self.settings : settings // ignore: cast_nullable_to_non_nullable
as AiSettings,
  ));
}

/// Create a copy of AiStatusState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<$Res> get settings {
  
  return $AiSettingsCopyWith<$Res>(_self.settings, (value) {
    return _then(_self.copyWith(settings: value));
  });
}
}

// dart format on
