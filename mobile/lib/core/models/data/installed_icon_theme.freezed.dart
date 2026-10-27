// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'installed_icon_theme.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$InstalledIconTheme {

/// The icon theme's id.
 String get id;/// The name people see.
 String get name;/// Who made the icon theme; empty when it does not say.
 String get author;/// The icon theme's licence; empty when it does not say.
 String get license;/// Where the icon theme came from.
 InstalledThemeOrigin get origin;
/// Create a copy of InstalledIconTheme
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InstalledIconThemeCopyWith<InstalledIconTheme> get copyWith => _$InstalledIconThemeCopyWithImpl<InstalledIconTheme>(this as InstalledIconTheme, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InstalledIconTheme&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.author, author) || other.author == author)&&(identical(other.license, license) || other.license == license)&&(identical(other.origin, origin) || other.origin == origin));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,author,license,origin);

@override
String toString() {
  return 'InstalledIconTheme(id: $id, name: $name, author: $author, license: $license, origin: $origin)';
}


}

/// @nodoc
abstract mixin class $InstalledIconThemeCopyWith<$Res>  {
  factory $InstalledIconThemeCopyWith(InstalledIconTheme value, $Res Function(InstalledIconTheme) _then) = _$InstalledIconThemeCopyWithImpl;
@useResult
$Res call({
 String id, String name, String author, String license, InstalledThemeOrigin origin
});




}
/// @nodoc
class _$InstalledIconThemeCopyWithImpl<$Res>
    implements $InstalledIconThemeCopyWith<$Res> {
  _$InstalledIconThemeCopyWithImpl(this._self, this._then);

  final InstalledIconTheme _self;
  final $Res Function(InstalledIconTheme) _then;

/// Create a copy of InstalledIconTheme
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? author = null,Object? license = null,Object? origin = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,license: null == license ? _self.license : license // ignore: cast_nullable_to_non_nullable
as String,origin: null == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as InstalledThemeOrigin,
  ));
}

}


/// Adds pattern-matching-related methods to [InstalledIconTheme].
extension InstalledIconThemePatterns on InstalledIconTheme {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InstalledIconTheme value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InstalledIconTheme() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InstalledIconTheme value)  $default,){
final _that = this;
switch (_that) {
case _InstalledIconTheme():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InstalledIconTheme value)?  $default,){
final _that = this;
switch (_that) {
case _InstalledIconTheme() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name,  String author,  String license,  InstalledThemeOrigin origin)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InstalledIconTheme() when $default != null:
return $default(_that.id,_that.name,_that.author,_that.license,_that.origin);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name,  String author,  String license,  InstalledThemeOrigin origin)  $default,) {final _that = this;
switch (_that) {
case _InstalledIconTheme():
return $default(_that.id,_that.name,_that.author,_that.license,_that.origin);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name,  String author,  String license,  InstalledThemeOrigin origin)?  $default,) {final _that = this;
switch (_that) {
case _InstalledIconTheme() when $default != null:
return $default(_that.id,_that.name,_that.author,_that.license,_that.origin);case _:
  return null;

}
}

}

/// @nodoc


class _InstalledIconTheme implements InstalledIconTheme {
  const _InstalledIconTheme({required this.id, required this.name, required this.author, required this.license, required this.origin});
  

/// The icon theme's id.
@override final  String id;
/// The name people see.
@override final  String name;
/// Who made the icon theme; empty when it does not say.
@override final  String author;
/// The icon theme's licence; empty when it does not say.
@override final  String license;
/// Where the icon theme came from.
@override final  InstalledThemeOrigin origin;

/// Create a copy of InstalledIconTheme
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InstalledIconThemeCopyWith<_InstalledIconTheme> get copyWith => __$InstalledIconThemeCopyWithImpl<_InstalledIconTheme>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InstalledIconTheme&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.author, author) || other.author == author)&&(identical(other.license, license) || other.license == license)&&(identical(other.origin, origin) || other.origin == origin));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,author,license,origin);

@override
String toString() {
  return 'InstalledIconTheme(id: $id, name: $name, author: $author, license: $license, origin: $origin)';
}


}

/// @nodoc
abstract mixin class _$InstalledIconThemeCopyWith<$Res> implements $InstalledIconThemeCopyWith<$Res> {
  factory _$InstalledIconThemeCopyWith(_InstalledIconTheme value, $Res Function(_InstalledIconTheme) _then) = __$InstalledIconThemeCopyWithImpl;
@override @useResult
$Res call({
 String id, String name, String author, String license, InstalledThemeOrigin origin
});




}
/// @nodoc
class __$InstalledIconThemeCopyWithImpl<$Res>
    implements _$InstalledIconThemeCopyWith<$Res> {
  __$InstalledIconThemeCopyWithImpl(this._self, this._then);

  final _InstalledIconTheme _self;
  final $Res Function(_InstalledIconTheme) _then;

/// Create a copy of InstalledIconTheme
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? author = null,Object? license = null,Object? origin = null,}) {
  return _then(_InstalledIconTheme(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,license: null == license ? _self.license : license // ignore: cast_nullable_to_non_nullable
as String,origin: null == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as InstalledThemeOrigin,
  ));
}


}

// dart format on
