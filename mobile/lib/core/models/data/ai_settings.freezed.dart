// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai_settings.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AiSettings {

/// Whether the AI features are on. They are off until the person turns
/// them on.
 bool get enabled;/// The provider's key (see `AiProviderInfo.key`).
 String get provider;/// The address typed, empty to use the provider's default.
 String get baseUrl;/// The model typed, empty to use the provider's default.
 String get model;/// The region typed, for the providers that use one.
 String get region;/// Whether a key is saved for [provider]. The key itself is never read
/// back.
 bool get hasKey;/// The server a question goes to, as a name: `api.anthropic.com`.
 String get host;/// Whether [host] is somewhere other than this device or its own
/// network (not a loopback address).
 bool get isRemote;/// The plain line about where a question goes and what is never sent,
/// in the words of the device ("this phone", "this tablet").
 String get privacy;
/// Create a copy of AiSettings
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiSettingsCopyWith<AiSettings> get copyWith => _$AiSettingsCopyWithImpl<AiSettings>(this as AiSettings, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiSettings&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.baseUrl, baseUrl) || other.baseUrl == baseUrl)&&(identical(other.model, model) || other.model == model)&&(identical(other.region, region) || other.region == region)&&(identical(other.hasKey, hasKey) || other.hasKey == hasKey)&&(identical(other.host, host) || other.host == host)&&(identical(other.isRemote, isRemote) || other.isRemote == isRemote)&&(identical(other.privacy, privacy) || other.privacy == privacy));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,provider,baseUrl,model,region,hasKey,host,isRemote,privacy);

@override
String toString() {
  return 'AiSettings(enabled: $enabled, provider: $provider, baseUrl: $baseUrl, model: $model, region: $region, hasKey: $hasKey, host: $host, isRemote: $isRemote, privacy: $privacy)';
}


}

/// @nodoc
abstract mixin class $AiSettingsCopyWith<$Res>  {
  factory $AiSettingsCopyWith(AiSettings value, $Res Function(AiSettings) _then) = _$AiSettingsCopyWithImpl;
@useResult
$Res call({
 bool enabled, String provider, String baseUrl, String model, String region, bool hasKey, String host, bool isRemote, String privacy
});




}
/// @nodoc
class _$AiSettingsCopyWithImpl<$Res>
    implements $AiSettingsCopyWith<$Res> {
  _$AiSettingsCopyWithImpl(this._self, this._then);

  final AiSettings _self;
  final $Res Function(AiSettings) _then;

/// Create a copy of AiSettings
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? enabled = null,Object? provider = null,Object? baseUrl = null,Object? model = null,Object? region = null,Object? hasKey = null,Object? host = null,Object? isRemote = null,Object? privacy = null,}) {
  return _then(_self.copyWith(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as String,baseUrl: null == baseUrl ? _self.baseUrl : baseUrl // ignore: cast_nullable_to_non_nullable
as String,model: null == model ? _self.model : model // ignore: cast_nullable_to_non_nullable
as String,region: null == region ? _self.region : region // ignore: cast_nullable_to_non_nullable
as String,hasKey: null == hasKey ? _self.hasKey : hasKey // ignore: cast_nullable_to_non_nullable
as bool,host: null == host ? _self.host : host // ignore: cast_nullable_to_non_nullable
as String,isRemote: null == isRemote ? _self.isRemote : isRemote // ignore: cast_nullable_to_non_nullable
as bool,privacy: null == privacy ? _self.privacy : privacy // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AiSettings].
extension AiSettingsPatterns on AiSettings {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AiSettings value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AiSettings() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AiSettings value)  $default,){
final _that = this;
switch (_that) {
case _AiSettings():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AiSettings value)?  $default,){
final _that = this;
switch (_that) {
case _AiSettings() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool enabled,  String provider,  String baseUrl,  String model,  String region,  bool hasKey,  String host,  bool isRemote,  String privacy)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AiSettings() when $default != null:
return $default(_that.enabled,_that.provider,_that.baseUrl,_that.model,_that.region,_that.hasKey,_that.host,_that.isRemote,_that.privacy);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool enabled,  String provider,  String baseUrl,  String model,  String region,  bool hasKey,  String host,  bool isRemote,  String privacy)  $default,) {final _that = this;
switch (_that) {
case _AiSettings():
return $default(_that.enabled,_that.provider,_that.baseUrl,_that.model,_that.region,_that.hasKey,_that.host,_that.isRemote,_that.privacy);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool enabled,  String provider,  String baseUrl,  String model,  String region,  bool hasKey,  String host,  bool isRemote,  String privacy)?  $default,) {final _that = this;
switch (_that) {
case _AiSettings() when $default != null:
return $default(_that.enabled,_that.provider,_that.baseUrl,_that.model,_that.region,_that.hasKey,_that.host,_that.isRemote,_that.privacy);case _:
  return null;

}
}

}

/// @nodoc


class _AiSettings implements AiSettings {
  const _AiSettings({this.enabled = false, this.provider = '', this.baseUrl = '', this.model = '', this.region = '', this.hasKey = false, this.host = '', this.isRemote = false, this.privacy = ''});
  

/// Whether the AI features are on. They are off until the person turns
/// them on.
@override@JsonKey() final  bool enabled;
/// The provider's key (see `AiProviderInfo.key`).
@override@JsonKey() final  String provider;
/// The address typed, empty to use the provider's default.
@override@JsonKey() final  String baseUrl;
/// The model typed, empty to use the provider's default.
@override@JsonKey() final  String model;
/// The region typed, for the providers that use one.
@override@JsonKey() final  String region;
/// Whether a key is saved for [provider]. The key itself is never read
/// back.
@override@JsonKey() final  bool hasKey;
/// The server a question goes to, as a name: `api.anthropic.com`.
@override@JsonKey() final  String host;
/// Whether [host] is somewhere other than this device or its own
/// network (not a loopback address).
@override@JsonKey() final  bool isRemote;
/// The plain line about where a question goes and what is never sent,
/// in the words of the device ("this phone", "this tablet").
@override@JsonKey() final  String privacy;

/// Create a copy of AiSettings
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AiSettingsCopyWith<_AiSettings> get copyWith => __$AiSettingsCopyWithImpl<_AiSettings>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AiSettings&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.baseUrl, baseUrl) || other.baseUrl == baseUrl)&&(identical(other.model, model) || other.model == model)&&(identical(other.region, region) || other.region == region)&&(identical(other.hasKey, hasKey) || other.hasKey == hasKey)&&(identical(other.host, host) || other.host == host)&&(identical(other.isRemote, isRemote) || other.isRemote == isRemote)&&(identical(other.privacy, privacy) || other.privacy == privacy));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,provider,baseUrl,model,region,hasKey,host,isRemote,privacy);

@override
String toString() {
  return 'AiSettings(enabled: $enabled, provider: $provider, baseUrl: $baseUrl, model: $model, region: $region, hasKey: $hasKey, host: $host, isRemote: $isRemote, privacy: $privacy)';
}


}

/// @nodoc
abstract mixin class _$AiSettingsCopyWith<$Res> implements $AiSettingsCopyWith<$Res> {
  factory _$AiSettingsCopyWith(_AiSettings value, $Res Function(_AiSettings) _then) = __$AiSettingsCopyWithImpl;
@override @useResult
$Res call({
 bool enabled, String provider, String baseUrl, String model, String region, bool hasKey, String host, bool isRemote, String privacy
});




}
/// @nodoc
class __$AiSettingsCopyWithImpl<$Res>
    implements _$AiSettingsCopyWith<$Res> {
  __$AiSettingsCopyWithImpl(this._self, this._then);

  final _AiSettings _self;
  final $Res Function(_AiSettings) _then;

/// Create a copy of AiSettings
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? enabled = null,Object? provider = null,Object? baseUrl = null,Object? model = null,Object? region = null,Object? hasKey = null,Object? host = null,Object? isRemote = null,Object? privacy = null,}) {
  return _then(_AiSettings(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as String,baseUrl: null == baseUrl ? _self.baseUrl : baseUrl // ignore: cast_nullable_to_non_nullable
as String,model: null == model ? _self.model : model // ignore: cast_nullable_to_non_nullable
as String,region: null == region ? _self.region : region // ignore: cast_nullable_to_non_nullable
as String,hasKey: null == hasKey ? _self.hasKey : hasKey // ignore: cast_nullable_to_non_nullable
as bool,host: null == host ? _self.host : host // ignore: cast_nullable_to_non_nullable
as String,isRemote: null == isRemote ? _self.isRemote : isRemote // ignore: cast_nullable_to_non_nullable
as bool,privacy: null == privacy ? _self.privacy : privacy // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
