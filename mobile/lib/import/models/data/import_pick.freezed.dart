// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'import_pick.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ImportPick {

/// Where the copy is: a file, or a folder with its contents.
 String get path;/// What the person chose, by its own name.
 String get name;
/// Create a copy of ImportPick
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportPickCopyWith<ImportPick> get copyWith => _$ImportPickCopyWithImpl<ImportPick>(this as ImportPick, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportPick&&(identical(other.path, path) || other.path == path)&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,path,name);

@override
String toString() {
  return 'ImportPick(path: $path, name: $name)';
}


}

/// @nodoc
abstract mixin class $ImportPickCopyWith<$Res>  {
  factory $ImportPickCopyWith(ImportPick value, $Res Function(ImportPick) _then) = _$ImportPickCopyWithImpl;
@useResult
$Res call({
 String path, String name
});




}
/// @nodoc
class _$ImportPickCopyWithImpl<$Res>
    implements $ImportPickCopyWith<$Res> {
  _$ImportPickCopyWithImpl(this._self, this._then);

  final ImportPick _self;
  final $Res Function(ImportPick) _then;

/// Create a copy of ImportPick
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? path = null,Object? name = null,}) {
  return _then(_self.copyWith(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [ImportPick].
extension ImportPickPatterns on ImportPick {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ImportPick value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ImportPick() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ImportPick value)  $default,){
final _that = this;
switch (_that) {
case _ImportPick():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ImportPick value)?  $default,){
final _that = this;
switch (_that) {
case _ImportPick() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String path,  String name)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ImportPick() when $default != null:
return $default(_that.path,_that.name);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String path,  String name)  $default,) {final _that = this;
switch (_that) {
case _ImportPick():
return $default(_that.path,_that.name);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String path,  String name)?  $default,) {final _that = this;
switch (_that) {
case _ImportPick() when $default != null:
return $default(_that.path,_that.name);case _:
  return null;

}
}

}

/// @nodoc


class _ImportPick implements ImportPick {
  const _ImportPick({required this.path, required this.name});
  

/// Where the copy is: a file, or a folder with its contents.
@override final  String path;
/// What the person chose, by its own name.
@override final  String name;

/// Create a copy of ImportPick
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ImportPickCopyWith<_ImportPick> get copyWith => __$ImportPickCopyWithImpl<_ImportPick>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ImportPick&&(identical(other.path, path) || other.path == path)&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,path,name);

@override
String toString() {
  return 'ImportPick(path: $path, name: $name)';
}


}

/// @nodoc
abstract mixin class _$ImportPickCopyWith<$Res> implements $ImportPickCopyWith<$Res> {
  factory _$ImportPickCopyWith(_ImportPick value, $Res Function(_ImportPick) _then) = __$ImportPickCopyWithImpl;
@override @useResult
$Res call({
 String path, String name
});




}
/// @nodoc
class __$ImportPickCopyWithImpl<$Res>
    implements _$ImportPickCopyWith<$Res> {
  __$ImportPickCopyWithImpl(this._self, this._then);

  final _ImportPick _self;
  final $Res Function(_ImportPick) _then;

/// Create a copy of ImportPick
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? path = null,Object? name = null,}) {
  return _then(_ImportPick(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
