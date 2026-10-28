// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voice_settings.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$VoiceSettings {

/// Whether "comma", "new paragraph" and "delete that" are acted on
/// rather than written. On by default, as on the desktop.
 bool get spokenCommands;
/// Create a copy of VoiceSettings
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceSettingsCopyWith<VoiceSettings> get copyWith => _$VoiceSettingsCopyWithImpl<VoiceSettings>(this as VoiceSettings, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceSettings&&(identical(other.spokenCommands, spokenCommands) || other.spokenCommands == spokenCommands));
}


@override
int get hashCode => Object.hash(runtimeType,spokenCommands);

@override
String toString() {
  return 'VoiceSettings(spokenCommands: $spokenCommands)';
}


}

/// @nodoc
abstract mixin class $VoiceSettingsCopyWith<$Res>  {
  factory $VoiceSettingsCopyWith(VoiceSettings value, $Res Function(VoiceSettings) _then) = _$VoiceSettingsCopyWithImpl;
@useResult
$Res call({
 bool spokenCommands
});




}
/// @nodoc
class _$VoiceSettingsCopyWithImpl<$Res>
    implements $VoiceSettingsCopyWith<$Res> {
  _$VoiceSettingsCopyWithImpl(this._self, this._then);

  final VoiceSettings _self;
  final $Res Function(VoiceSettings) _then;

/// Create a copy of VoiceSettings
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? spokenCommands = null,}) {
  return _then(_self.copyWith(
spokenCommands: null == spokenCommands ? _self.spokenCommands : spokenCommands // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [VoiceSettings].
extension VoiceSettingsPatterns on VoiceSettings {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VoiceSettings value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VoiceSettings() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VoiceSettings value)  $default,){
final _that = this;
switch (_that) {
case _VoiceSettings():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VoiceSettings value)?  $default,){
final _that = this;
switch (_that) {
case _VoiceSettings() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool spokenCommands)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VoiceSettings() when $default != null:
return $default(_that.spokenCommands);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool spokenCommands)  $default,) {final _that = this;
switch (_that) {
case _VoiceSettings():
return $default(_that.spokenCommands);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool spokenCommands)?  $default,) {final _that = this;
switch (_that) {
case _VoiceSettings() when $default != null:
return $default(_that.spokenCommands);case _:
  return null;

}
}

}

/// @nodoc


class _VoiceSettings implements VoiceSettings {
  const _VoiceSettings({this.spokenCommands = true});
  

/// Whether "comma", "new paragraph" and "delete that" are acted on
/// rather than written. On by default, as on the desktop.
@override@JsonKey() final  bool spokenCommands;

/// Create a copy of VoiceSettings
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VoiceSettingsCopyWith<_VoiceSettings> get copyWith => __$VoiceSettingsCopyWithImpl<_VoiceSettings>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VoiceSettings&&(identical(other.spokenCommands, spokenCommands) || other.spokenCommands == spokenCommands));
}


@override
int get hashCode => Object.hash(runtimeType,spokenCommands);

@override
String toString() {
  return 'VoiceSettings(spokenCommands: $spokenCommands)';
}


}

/// @nodoc
abstract mixin class _$VoiceSettingsCopyWith<$Res> implements $VoiceSettingsCopyWith<$Res> {
  factory _$VoiceSettingsCopyWith(_VoiceSettings value, $Res Function(_VoiceSettings) _then) = __$VoiceSettingsCopyWithImpl;
@override @useResult
$Res call({
 bool spokenCommands
});




}
/// @nodoc
class __$VoiceSettingsCopyWithImpl<$Res>
    implements _$VoiceSettingsCopyWith<$Res> {
  __$VoiceSettingsCopyWithImpl(this._self, this._then);

  final _VoiceSettings _self;
  final $Res Function(_VoiceSettings) _then;

/// Create a copy of VoiceSettings
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? spokenCommands = null,}) {
  return _then(_VoiceSettings(
spokenCommands: null == spokenCommands ? _self.spokenCommands : spokenCommands // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
