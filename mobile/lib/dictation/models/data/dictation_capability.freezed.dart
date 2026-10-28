// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dictation_capability.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DictationCapability {

/// Whether speech can be recognized at all.
 bool get available;/// Whether it is recognized on the phone itself. Always so on iOS when
/// [available]; on Android false where the system's recognizer, asked to
/// work offline, stands in for one on the device.
 bool get onDevice; DictationPermission get permission;/// The name of the language asked about, in the phone's language.
 String get language;/// Why not, while [available] is false.
 DictationUnavailableReason? get reason;
/// Create a copy of DictationCapability
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationCapabilityCopyWith<DictationCapability> get copyWith => _$DictationCapabilityCopyWithImpl<DictationCapability>(this as DictationCapability, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationCapability&&(identical(other.available, available) || other.available == available)&&(identical(other.onDevice, onDevice) || other.onDevice == onDevice)&&(identical(other.permission, permission) || other.permission == permission)&&(identical(other.language, language) || other.language == language)&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,available,onDevice,permission,language,reason);

@override
String toString() {
  return 'DictationCapability(available: $available, onDevice: $onDevice, permission: $permission, language: $language, reason: $reason)';
}


}

/// @nodoc
abstract mixin class $DictationCapabilityCopyWith<$Res>  {
  factory $DictationCapabilityCopyWith(DictationCapability value, $Res Function(DictationCapability) _then) = _$DictationCapabilityCopyWithImpl;
@useResult
$Res call({
 bool available, bool onDevice, DictationPermission permission, String language, DictationUnavailableReason? reason
});




}
/// @nodoc
class _$DictationCapabilityCopyWithImpl<$Res>
    implements $DictationCapabilityCopyWith<$Res> {
  _$DictationCapabilityCopyWithImpl(this._self, this._then);

  final DictationCapability _self;
  final $Res Function(DictationCapability) _then;

/// Create a copy of DictationCapability
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? available = null,Object? onDevice = null,Object? permission = null,Object? language = null,Object? reason = freezed,}) {
  return _then(_self.copyWith(
available: null == available ? _self.available : available // ignore: cast_nullable_to_non_nullable
as bool,onDevice: null == onDevice ? _self.onDevice : onDevice // ignore: cast_nullable_to_non_nullable
as bool,permission: null == permission ? _self.permission : permission // ignore: cast_nullable_to_non_nullable
as DictationPermission,language: null == language ? _self.language : language // ignore: cast_nullable_to_non_nullable
as String,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as DictationUnavailableReason?,
  ));
}

}


/// Adds pattern-matching-related methods to [DictationCapability].
extension DictationCapabilityPatterns on DictationCapability {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DictationCapability value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DictationCapability() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DictationCapability value)  $default,){
final _that = this;
switch (_that) {
case _DictationCapability():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DictationCapability value)?  $default,){
final _that = this;
switch (_that) {
case _DictationCapability() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool available,  bool onDevice,  DictationPermission permission,  String language,  DictationUnavailableReason? reason)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DictationCapability() when $default != null:
return $default(_that.available,_that.onDevice,_that.permission,_that.language,_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool available,  bool onDevice,  DictationPermission permission,  String language,  DictationUnavailableReason? reason)  $default,) {final _that = this;
switch (_that) {
case _DictationCapability():
return $default(_that.available,_that.onDevice,_that.permission,_that.language,_that.reason);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool available,  bool onDevice,  DictationPermission permission,  String language,  DictationUnavailableReason? reason)?  $default,) {final _that = this;
switch (_that) {
case _DictationCapability() when $default != null:
return $default(_that.available,_that.onDevice,_that.permission,_that.language,_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class _DictationCapability implements DictationCapability {
  const _DictationCapability({required this.available, required this.onDevice, required this.permission, this.language = '', this.reason});
  

/// Whether speech can be recognized at all.
@override final  bool available;
/// Whether it is recognized on the phone itself. Always so on iOS when
/// [available]; on Android false where the system's recognizer, asked to
/// work offline, stands in for one on the device.
@override final  bool onDevice;
@override final  DictationPermission permission;
/// The name of the language asked about, in the phone's language.
@override@JsonKey() final  String language;
/// Why not, while [available] is false.
@override final  DictationUnavailableReason? reason;

/// Create a copy of DictationCapability
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DictationCapabilityCopyWith<_DictationCapability> get copyWith => __$DictationCapabilityCopyWithImpl<_DictationCapability>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DictationCapability&&(identical(other.available, available) || other.available == available)&&(identical(other.onDevice, onDevice) || other.onDevice == onDevice)&&(identical(other.permission, permission) || other.permission == permission)&&(identical(other.language, language) || other.language == language)&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,available,onDevice,permission,language,reason);

@override
String toString() {
  return 'DictationCapability(available: $available, onDevice: $onDevice, permission: $permission, language: $language, reason: $reason)';
}


}

/// @nodoc
abstract mixin class _$DictationCapabilityCopyWith<$Res> implements $DictationCapabilityCopyWith<$Res> {
  factory _$DictationCapabilityCopyWith(_DictationCapability value, $Res Function(_DictationCapability) _then) = __$DictationCapabilityCopyWithImpl;
@override @useResult
$Res call({
 bool available, bool onDevice, DictationPermission permission, String language, DictationUnavailableReason? reason
});




}
/// @nodoc
class __$DictationCapabilityCopyWithImpl<$Res>
    implements _$DictationCapabilityCopyWith<$Res> {
  __$DictationCapabilityCopyWithImpl(this._self, this._then);

  final _DictationCapability _self;
  final $Res Function(_DictationCapability) _then;

/// Create a copy of DictationCapability
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? available = null,Object? onDevice = null,Object? permission = null,Object? language = null,Object? reason = freezed,}) {
  return _then(_DictationCapability(
available: null == available ? _self.available : available // ignore: cast_nullable_to_non_nullable
as bool,onDevice: null == onDevice ? _self.onDevice : onDevice // ignore: cast_nullable_to_non_nullable
as bool,permission: null == permission ? _self.permission : permission // ignore: cast_nullable_to_non_nullable
as DictationPermission,language: null == language ? _self.language : language // ignore: cast_nullable_to_non_nullable
as String,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as DictationUnavailableReason?,
  ));
}


}

// dart format on
