// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'installed_theme.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$InstalledTheme {

/// The theme file's name without `.json`: `nord`.
 String get id;/// The name people see: `Nord`.
 String get name;/// Who made the theme; empty when the file does not say.
 String get author;/// The theme's licence; empty when the file does not say.
 String get license;/// Whether the theme is for a dark interface.
 bool get dark;/// Where the theme came from.
 InstalledThemeOrigin get origin;/// The theme's colours.
 ThemeTokens get tokens;/// Where the theme was taken from, when it was.
 String? get source;
/// Create a copy of InstalledTheme
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InstalledThemeCopyWith<InstalledTheme> get copyWith => _$InstalledThemeCopyWithImpl<InstalledTheme>(this as InstalledTheme, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InstalledTheme&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.author, author) || other.author == author)&&(identical(other.license, license) || other.license == license)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.origin, origin) || other.origin == origin)&&(identical(other.tokens, tokens) || other.tokens == tokens)&&(identical(other.source, source) || other.source == source));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,author,license,dark,origin,tokens,source);

@override
String toString() {
  return 'InstalledTheme(id: $id, name: $name, author: $author, license: $license, dark: $dark, origin: $origin, tokens: $tokens, source: $source)';
}


}

/// @nodoc
abstract mixin class $InstalledThemeCopyWith<$Res>  {
  factory $InstalledThemeCopyWith(InstalledTheme value, $Res Function(InstalledTheme) _then) = _$InstalledThemeCopyWithImpl;
@useResult
$Res call({
 String id, String name, String author, String license, bool dark, InstalledThemeOrigin origin, ThemeTokens tokens, String? source
});




}
/// @nodoc
class _$InstalledThemeCopyWithImpl<$Res>
    implements $InstalledThemeCopyWith<$Res> {
  _$InstalledThemeCopyWithImpl(this._self, this._then);

  final InstalledTheme _self;
  final $Res Function(InstalledTheme) _then;

/// Create a copy of InstalledTheme
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? author = null,Object? license = null,Object? dark = null,Object? origin = null,Object? tokens = null,Object? source = freezed,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,license: null == license ? _self.license : license // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,origin: null == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as InstalledThemeOrigin,tokens: null == tokens ? _self.tokens : tokens // ignore: cast_nullable_to_non_nullable
as ThemeTokens,source: freezed == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [InstalledTheme].
extension InstalledThemePatterns on InstalledTheme {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InstalledTheme value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InstalledTheme() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InstalledTheme value)  $default,){
final _that = this;
switch (_that) {
case _InstalledTheme():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InstalledTheme value)?  $default,){
final _that = this;
switch (_that) {
case _InstalledTheme() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name,  String author,  String license,  bool dark,  InstalledThemeOrigin origin,  ThemeTokens tokens,  String? source)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InstalledTheme() when $default != null:
return $default(_that.id,_that.name,_that.author,_that.license,_that.dark,_that.origin,_that.tokens,_that.source);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name,  String author,  String license,  bool dark,  InstalledThemeOrigin origin,  ThemeTokens tokens,  String? source)  $default,) {final _that = this;
switch (_that) {
case _InstalledTheme():
return $default(_that.id,_that.name,_that.author,_that.license,_that.dark,_that.origin,_that.tokens,_that.source);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name,  String author,  String license,  bool dark,  InstalledThemeOrigin origin,  ThemeTokens tokens,  String? source)?  $default,) {final _that = this;
switch (_that) {
case _InstalledTheme() when $default != null:
return $default(_that.id,_that.name,_that.author,_that.license,_that.dark,_that.origin,_that.tokens,_that.source);case _:
  return null;

}
}

}

/// @nodoc


class _InstalledTheme implements InstalledTheme {
  const _InstalledTheme({required this.id, required this.name, required this.author, required this.license, required this.dark, required this.origin, required this.tokens, this.source});
  

/// The theme file's name without `.json`: `nord`.
@override final  String id;
/// The name people see: `Nord`.
@override final  String name;
/// Who made the theme; empty when the file does not say.
@override final  String author;
/// The theme's licence; empty when the file does not say.
@override final  String license;
/// Whether the theme is for a dark interface.
@override final  bool dark;
/// Where the theme came from.
@override final  InstalledThemeOrigin origin;
/// The theme's colours.
@override final  ThemeTokens tokens;
/// Where the theme was taken from, when it was.
@override final  String? source;

/// Create a copy of InstalledTheme
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InstalledThemeCopyWith<_InstalledTheme> get copyWith => __$InstalledThemeCopyWithImpl<_InstalledTheme>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InstalledTheme&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.author, author) || other.author == author)&&(identical(other.license, license) || other.license == license)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.origin, origin) || other.origin == origin)&&(identical(other.tokens, tokens) || other.tokens == tokens)&&(identical(other.source, source) || other.source == source));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,author,license,dark,origin,tokens,source);

@override
String toString() {
  return 'InstalledTheme(id: $id, name: $name, author: $author, license: $license, dark: $dark, origin: $origin, tokens: $tokens, source: $source)';
}


}

/// @nodoc
abstract mixin class _$InstalledThemeCopyWith<$Res> implements $InstalledThemeCopyWith<$Res> {
  factory _$InstalledThemeCopyWith(_InstalledTheme value, $Res Function(_InstalledTheme) _then) = __$InstalledThemeCopyWithImpl;
@override @useResult
$Res call({
 String id, String name, String author, String license, bool dark, InstalledThemeOrigin origin, ThemeTokens tokens, String? source
});




}
/// @nodoc
class __$InstalledThemeCopyWithImpl<$Res>
    implements _$InstalledThemeCopyWith<$Res> {
  __$InstalledThemeCopyWithImpl(this._self, this._then);

  final _InstalledTheme _self;
  final $Res Function(_InstalledTheme) _then;

/// Create a copy of InstalledTheme
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? author = null,Object? license = null,Object? dark = null,Object? origin = null,Object? tokens = null,Object? source = freezed,}) {
  return _then(_InstalledTheme(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,license: null == license ? _self.license : license // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,origin: null == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as InstalledThemeOrigin,tokens: null == tokens ? _self.tokens : tokens // ignore: cast_nullable_to_non_nullable
as ThemeTokens,source: freezed == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
