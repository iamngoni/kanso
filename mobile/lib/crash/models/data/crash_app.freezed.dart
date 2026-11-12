// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_app.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$CrashApp {

/// `0.9.0`.
 String get version;/// `57`.
 String get build;/// Shorebird's patch number; null without Shorebird or before the first
/// patch. Always written, `null` included: the server requires the key
/// (the build.yaml default, `include_if_null: false`, would drop it and
/// the server would answer 400 for `app.patch`).
@JsonKey(includeIfNull: true) int? get patch;
/// Create a copy of CrashApp
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashAppCopyWith<CrashApp> get copyWith => _$CrashAppCopyWithImpl<CrashApp>(this as CrashApp, _$identity);

  /// Serializes this CrashApp to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashApp&&(identical(other.version, version) || other.version == version)&&(identical(other.build, build) || other.build == build)&&(identical(other.patch, patch) || other.patch == patch));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,version,build,patch);

@override
String toString() {
  return 'CrashApp(version: $version, build: $build, patch: $patch)';
}


}

/// @nodoc
abstract mixin class $CrashAppCopyWith<$Res>  {
  factory $CrashAppCopyWith(CrashApp value, $Res Function(CrashApp) _then) = _$CrashAppCopyWithImpl;
@useResult
$Res call({
 String version, String build,@JsonKey(includeIfNull: true) int? patch
});




}
/// @nodoc
class _$CrashAppCopyWithImpl<$Res>
    implements $CrashAppCopyWith<$Res> {
  _$CrashAppCopyWithImpl(this._self, this._then);

  final CrashApp _self;
  final $Res Function(CrashApp) _then;

/// Create a copy of CrashApp
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? version = null,Object? build = null,Object? patch = freezed,}) {
  return _then(_self.copyWith(
version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,build: null == build ? _self.build : build // ignore: cast_nullable_to_non_nullable
as String,patch: freezed == patch ? _self.patch : patch // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}

}


/// Adds pattern-matching-related methods to [CrashApp].
extension CrashAppPatterns on CrashApp {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashApp value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashApp() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashApp value)  $default,){
final _that = this;
switch (_that) {
case _CrashApp():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashApp value)?  $default,){
final _that = this;
switch (_that) {
case _CrashApp() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String version,  String build, @JsonKey(includeIfNull: true)  int? patch)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CrashApp() when $default != null:
return $default(_that.version,_that.build,_that.patch);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String version,  String build, @JsonKey(includeIfNull: true)  int? patch)  $default,) {final _that = this;
switch (_that) {
case _CrashApp():
return $default(_that.version,_that.build,_that.patch);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String version,  String build, @JsonKey(includeIfNull: true)  int? patch)?  $default,) {final _that = this;
switch (_that) {
case _CrashApp() when $default != null:
return $default(_that.version,_that.build,_that.patch);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _CrashApp implements CrashApp {
  const _CrashApp({required this.version, required this.build, @JsonKey(includeIfNull: true) required this.patch});
  factory _CrashApp.fromJson(Map<String, dynamic> json) => _$CrashAppFromJson(json);

/// `0.9.0`.
@override final  String version;
/// `57`.
@override final  String build;
/// Shorebird's patch number; null without Shorebird or before the first
/// patch. Always written, `null` included: the server requires the key
/// (the build.yaml default, `include_if_null: false`, would drop it and
/// the server would answer 400 for `app.patch`).
@override@JsonKey(includeIfNull: true) final  int? patch;

/// Create a copy of CrashApp
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashAppCopyWith<_CrashApp> get copyWith => __$CrashAppCopyWithImpl<_CrashApp>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CrashAppToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashApp&&(identical(other.version, version) || other.version == version)&&(identical(other.build, build) || other.build == build)&&(identical(other.patch, patch) || other.patch == patch));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,version,build,patch);

@override
String toString() {
  return 'CrashApp(version: $version, build: $build, patch: $patch)';
}


}

/// @nodoc
abstract mixin class _$CrashAppCopyWith<$Res> implements $CrashAppCopyWith<$Res> {
  factory _$CrashAppCopyWith(_CrashApp value, $Res Function(_CrashApp) _then) = __$CrashAppCopyWithImpl;
@override @useResult
$Res call({
 String version, String build,@JsonKey(includeIfNull: true) int? patch
});




}
/// @nodoc
class __$CrashAppCopyWithImpl<$Res>
    implements _$CrashAppCopyWith<$Res> {
  __$CrashAppCopyWithImpl(this._self, this._then);

  final _CrashApp _self;
  final $Res Function(_CrashApp) _then;

/// Create a copy of CrashApp
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? version = null,Object? build = null,Object? patch = freezed,}) {
  return _then(_CrashApp(
version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,build: null == build ? _self.build : build // ignore: cast_nullable_to_non_nullable
as String,patch: freezed == patch ? _self.patch : patch // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}

// dart format on
