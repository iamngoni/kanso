// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai_provider_info.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AiProviderInfo {

/// The provider's stable name, what settings and keys are kept under:
/// `anthropic`, `ollama`.
 String get key;/// What the picker calls it.
 String get label;/// One plain line under [label] in the picker.
 String get description;/// The address used when none is typed.
 String get defaultBaseUrl;/// The model used when none is typed.
 String get defaultModel;/// Whether the provider cannot answer without a key.
 bool get needsKey;/// Whether the provider takes a key at all (a local model takes one only
/// if its server asks).
 bool get acceptsKey;/// Whether the provider is reached through a region (Bedrock).
 bool get usesRegion;/// The region used when none is typed.
 String get defaultRegion;
/// Create a copy of AiProviderInfo
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiProviderInfoCopyWith<AiProviderInfo> get copyWith => _$AiProviderInfoCopyWithImpl<AiProviderInfo>(this as AiProviderInfo, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiProviderInfo&&(identical(other.key, key) || other.key == key)&&(identical(other.label, label) || other.label == label)&&(identical(other.description, description) || other.description == description)&&(identical(other.defaultBaseUrl, defaultBaseUrl) || other.defaultBaseUrl == defaultBaseUrl)&&(identical(other.defaultModel, defaultModel) || other.defaultModel == defaultModel)&&(identical(other.needsKey, needsKey) || other.needsKey == needsKey)&&(identical(other.acceptsKey, acceptsKey) || other.acceptsKey == acceptsKey)&&(identical(other.usesRegion, usesRegion) || other.usesRegion == usesRegion)&&(identical(other.defaultRegion, defaultRegion) || other.defaultRegion == defaultRegion));
}


@override
int get hashCode => Object.hash(runtimeType,key,label,description,defaultBaseUrl,defaultModel,needsKey,acceptsKey,usesRegion,defaultRegion);

@override
String toString() {
  return 'AiProviderInfo(key: $key, label: $label, description: $description, defaultBaseUrl: $defaultBaseUrl, defaultModel: $defaultModel, needsKey: $needsKey, acceptsKey: $acceptsKey, usesRegion: $usesRegion, defaultRegion: $defaultRegion)';
}


}

/// @nodoc
abstract mixin class $AiProviderInfoCopyWith<$Res>  {
  factory $AiProviderInfoCopyWith(AiProviderInfo value, $Res Function(AiProviderInfo) _then) = _$AiProviderInfoCopyWithImpl;
@useResult
$Res call({
 String key, String label, String description, String defaultBaseUrl, String defaultModel, bool needsKey, bool acceptsKey, bool usesRegion, String defaultRegion
});




}
/// @nodoc
class _$AiProviderInfoCopyWithImpl<$Res>
    implements $AiProviderInfoCopyWith<$Res> {
  _$AiProviderInfoCopyWithImpl(this._self, this._then);

  final AiProviderInfo _self;
  final $Res Function(AiProviderInfo) _then;

/// Create a copy of AiProviderInfo
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? key = null,Object? label = null,Object? description = null,Object? defaultBaseUrl = null,Object? defaultModel = null,Object? needsKey = null,Object? acceptsKey = null,Object? usesRegion = null,Object? defaultRegion = null,}) {
  return _then(_self.copyWith(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,description: null == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as String,defaultBaseUrl: null == defaultBaseUrl ? _self.defaultBaseUrl : defaultBaseUrl // ignore: cast_nullable_to_non_nullable
as String,defaultModel: null == defaultModel ? _self.defaultModel : defaultModel // ignore: cast_nullable_to_non_nullable
as String,needsKey: null == needsKey ? _self.needsKey : needsKey // ignore: cast_nullable_to_non_nullable
as bool,acceptsKey: null == acceptsKey ? _self.acceptsKey : acceptsKey // ignore: cast_nullable_to_non_nullable
as bool,usesRegion: null == usesRegion ? _self.usesRegion : usesRegion // ignore: cast_nullable_to_non_nullable
as bool,defaultRegion: null == defaultRegion ? _self.defaultRegion : defaultRegion // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AiProviderInfo].
extension AiProviderInfoPatterns on AiProviderInfo {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AiProviderInfo value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AiProviderInfo() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AiProviderInfo value)  $default,){
final _that = this;
switch (_that) {
case _AiProviderInfo():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AiProviderInfo value)?  $default,){
final _that = this;
switch (_that) {
case _AiProviderInfo() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String key,  String label,  String description,  String defaultBaseUrl,  String defaultModel,  bool needsKey,  bool acceptsKey,  bool usesRegion,  String defaultRegion)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AiProviderInfo() when $default != null:
return $default(_that.key,_that.label,_that.description,_that.defaultBaseUrl,_that.defaultModel,_that.needsKey,_that.acceptsKey,_that.usesRegion,_that.defaultRegion);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String key,  String label,  String description,  String defaultBaseUrl,  String defaultModel,  bool needsKey,  bool acceptsKey,  bool usesRegion,  String defaultRegion)  $default,) {final _that = this;
switch (_that) {
case _AiProviderInfo():
return $default(_that.key,_that.label,_that.description,_that.defaultBaseUrl,_that.defaultModel,_that.needsKey,_that.acceptsKey,_that.usesRegion,_that.defaultRegion);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String key,  String label,  String description,  String defaultBaseUrl,  String defaultModel,  bool needsKey,  bool acceptsKey,  bool usesRegion,  String defaultRegion)?  $default,) {final _that = this;
switch (_that) {
case _AiProviderInfo() when $default != null:
return $default(_that.key,_that.label,_that.description,_that.defaultBaseUrl,_that.defaultModel,_that.needsKey,_that.acceptsKey,_that.usesRegion,_that.defaultRegion);case _:
  return null;

}
}

}

/// @nodoc


class _AiProviderInfo implements AiProviderInfo {
  const _AiProviderInfo({required this.key, required this.label, required this.description, required this.defaultBaseUrl, required this.defaultModel, required this.needsKey, required this.acceptsKey, this.usesRegion = false, this.defaultRegion = ''});
  

/// The provider's stable name, what settings and keys are kept under:
/// `anthropic`, `ollama`.
@override final  String key;
/// What the picker calls it.
@override final  String label;
/// One plain line under [label] in the picker.
@override final  String description;
/// The address used when none is typed.
@override final  String defaultBaseUrl;
/// The model used when none is typed.
@override final  String defaultModel;
/// Whether the provider cannot answer without a key.
@override final  bool needsKey;
/// Whether the provider takes a key at all (a local model takes one only
/// if its server asks).
@override final  bool acceptsKey;
/// Whether the provider is reached through a region (Bedrock).
@override@JsonKey() final  bool usesRegion;
/// The region used when none is typed.
@override@JsonKey() final  String defaultRegion;

/// Create a copy of AiProviderInfo
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AiProviderInfoCopyWith<_AiProviderInfo> get copyWith => __$AiProviderInfoCopyWithImpl<_AiProviderInfo>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AiProviderInfo&&(identical(other.key, key) || other.key == key)&&(identical(other.label, label) || other.label == label)&&(identical(other.description, description) || other.description == description)&&(identical(other.defaultBaseUrl, defaultBaseUrl) || other.defaultBaseUrl == defaultBaseUrl)&&(identical(other.defaultModel, defaultModel) || other.defaultModel == defaultModel)&&(identical(other.needsKey, needsKey) || other.needsKey == needsKey)&&(identical(other.acceptsKey, acceptsKey) || other.acceptsKey == acceptsKey)&&(identical(other.usesRegion, usesRegion) || other.usesRegion == usesRegion)&&(identical(other.defaultRegion, defaultRegion) || other.defaultRegion == defaultRegion));
}


@override
int get hashCode => Object.hash(runtimeType,key,label,description,defaultBaseUrl,defaultModel,needsKey,acceptsKey,usesRegion,defaultRegion);

@override
String toString() {
  return 'AiProviderInfo(key: $key, label: $label, description: $description, defaultBaseUrl: $defaultBaseUrl, defaultModel: $defaultModel, needsKey: $needsKey, acceptsKey: $acceptsKey, usesRegion: $usesRegion, defaultRegion: $defaultRegion)';
}


}

/// @nodoc
abstract mixin class _$AiProviderInfoCopyWith<$Res> implements $AiProviderInfoCopyWith<$Res> {
  factory _$AiProviderInfoCopyWith(_AiProviderInfo value, $Res Function(_AiProviderInfo) _then) = __$AiProviderInfoCopyWithImpl;
@override @useResult
$Res call({
 String key, String label, String description, String defaultBaseUrl, String defaultModel, bool needsKey, bool acceptsKey, bool usesRegion, String defaultRegion
});




}
/// @nodoc
class __$AiProviderInfoCopyWithImpl<$Res>
    implements _$AiProviderInfoCopyWith<$Res> {
  __$AiProviderInfoCopyWithImpl(this._self, this._then);

  final _AiProviderInfo _self;
  final $Res Function(_AiProviderInfo) _then;

/// Create a copy of AiProviderInfo
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? key = null,Object? label = null,Object? description = null,Object? defaultBaseUrl = null,Object? defaultModel = null,Object? needsKey = null,Object? acceptsKey = null,Object? usesRegion = null,Object? defaultRegion = null,}) {
  return _then(_AiProviderInfo(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,description: null == description ? _self.description : description // ignore: cast_nullable_to_non_nullable
as String,defaultBaseUrl: null == defaultBaseUrl ? _self.defaultBaseUrl : defaultBaseUrl // ignore: cast_nullable_to_non_nullable
as String,defaultModel: null == defaultModel ? _self.defaultModel : defaultModel // ignore: cast_nullable_to_non_nullable
as String,needsKey: null == needsKey ? _self.needsKey : needsKey // ignore: cast_nullable_to_non_nullable
as bool,acceptsKey: null == acceptsKey ? _self.acceptsKey : acceptsKey // ignore: cast_nullable_to_non_nullable
as bool,usesRegion: null == usesRegion ? _self.usesRegion : usesRegion // ignore: cast_nullable_to_non_nullable
as bool,defaultRegion: null == defaultRegion ? _self.defaultRegion : defaultRegion // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
