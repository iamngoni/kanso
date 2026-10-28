// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai_settings_change.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AiSettingsChange {

 bool get enabled; String get provider; String get baseUrl; String get model; String get region;
/// Create a copy of AiSettingsChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsChangeCopyWith<AiSettingsChange> get copyWith => _$AiSettingsChangeCopyWithImpl<AiSettingsChange>(this as AiSettingsChange, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettingsChange&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.baseUrl, baseUrl) || other.baseUrl == baseUrl)&&(identical(other.model, model) || other.model == model)&&(identical(other.region, region) || other.region == region));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,provider,baseUrl,model,region);

@override
String toString() {
  return 'AiSettingsChange(enabled: $enabled, provider: $provider, baseUrl: $baseUrl, model: $model, region: $region)';
}


}

/// @nodoc
abstract mixin class $AiSettingsChangeCopyWith<$Res>  {
  factory $AiSettingsChangeCopyWith(AiSettingsChange value, $Res Function(AiSettingsChange) _then) = _$AiSettingsChangeCopyWithImpl;
@useResult
$Res call({
 bool enabled, String provider, String baseUrl, String model, String region
});




}
/// @nodoc
class _$AiSettingsChangeCopyWithImpl<$Res>
    implements $AiSettingsChangeCopyWith<$Res> {
  _$AiSettingsChangeCopyWithImpl(this._self, this._then);

  final AiSettingsChange _self;
  final $Res Function(AiSettingsChange) _then;

/// Create a copy of AiSettingsChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? enabled = null,Object? provider = null,Object? baseUrl = null,Object? model = null,Object? region = null,}) {
  return _then(_self.copyWith(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as String,baseUrl: null == baseUrl ? _self.baseUrl : baseUrl // ignore: cast_nullable_to_non_nullable
as String,model: null == model ? _self.model : model // ignore: cast_nullable_to_non_nullable
as String,region: null == region ? _self.region : region // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AiSettingsChange].
extension AiSettingsChangePatterns on AiSettingsChange {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AiSettingsChange value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AiSettingsChange() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AiSettingsChange value)  $default,){
final _that = this;
switch (_that) {
case _AiSettingsChange():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AiSettingsChange value)?  $default,){
final _that = this;
switch (_that) {
case _AiSettingsChange() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool enabled,  String provider,  String baseUrl,  String model,  String region)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AiSettingsChange() when $default != null:
return $default(_that.enabled,_that.provider,_that.baseUrl,_that.model,_that.region);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool enabled,  String provider,  String baseUrl,  String model,  String region)  $default,) {final _that = this;
switch (_that) {
case _AiSettingsChange():
return $default(_that.enabled,_that.provider,_that.baseUrl,_that.model,_that.region);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool enabled,  String provider,  String baseUrl,  String model,  String region)?  $default,) {final _that = this;
switch (_that) {
case _AiSettingsChange() when $default != null:
return $default(_that.enabled,_that.provider,_that.baseUrl,_that.model,_that.region);case _:
  return null;

}
}

}

/// @nodoc


class _AiSettingsChange implements AiSettingsChange {
  const _AiSettingsChange({required this.enabled, required this.provider, this.baseUrl = '', this.model = '', this.region = ''});
  

@override final  bool enabled;
@override final  String provider;
@override@JsonKey() final  String baseUrl;
@override@JsonKey() final  String model;
@override@JsonKey() final  String region;

/// Create a copy of AiSettingsChange
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AiSettingsChangeCopyWith<_AiSettingsChange> get copyWith => __$AiSettingsChangeCopyWithImpl<_AiSettingsChange>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AiSettingsChange&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.baseUrl, baseUrl) || other.baseUrl == baseUrl)&&(identical(other.model, model) || other.model == model)&&(identical(other.region, region) || other.region == region));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,provider,baseUrl,model,region);

@override
String toString() {
  return 'AiSettingsChange(enabled: $enabled, provider: $provider, baseUrl: $baseUrl, model: $model, region: $region)';
}


}

/// @nodoc
abstract mixin class _$AiSettingsChangeCopyWith<$Res> implements $AiSettingsChangeCopyWith<$Res> {
  factory _$AiSettingsChangeCopyWith(_AiSettingsChange value, $Res Function(_AiSettingsChange) _then) = __$AiSettingsChangeCopyWithImpl;
@override @useResult
$Res call({
 bool enabled, String provider, String baseUrl, String model, String region
});




}
/// @nodoc
class __$AiSettingsChangeCopyWithImpl<$Res>
    implements _$AiSettingsChangeCopyWith<$Res> {
  __$AiSettingsChangeCopyWithImpl(this._self, this._then);

  final _AiSettingsChange _self;
  final $Res Function(_AiSettingsChange) _then;

/// Create a copy of AiSettingsChange
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? enabled = null,Object? provider = null,Object? baseUrl = null,Object? model = null,Object? region = null,}) {
  return _then(_AiSettingsChange(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as String,baseUrl: null == baseUrl ? _self.baseUrl : baseUrl // ignore: cast_nullable_to_non_nullable
as String,model: null == model ? _self.model : model // ignore: cast_nullable_to_non_nullable
as String,region: null == region ? _self.region : region // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
