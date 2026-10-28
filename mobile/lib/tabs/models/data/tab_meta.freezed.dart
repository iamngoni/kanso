// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'tab_meta.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TabMeta {

 String get id; String get title; bool get locked;
/// Create a copy of TabMeta
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabMetaCopyWith<TabMeta> get copyWith => _$TabMetaCopyWithImpl<TabMeta>(this as TabMeta, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabMeta&&(identical(other.id, id) || other.id == id)&&(identical(other.title, title) || other.title == title)&&(identical(other.locked, locked) || other.locked == locked));
}


@override
int get hashCode => Object.hash(runtimeType,id,title,locked);

@override
String toString() {
  return 'TabMeta(id: $id, title: $title, locked: $locked)';
}


}

/// @nodoc
abstract mixin class $TabMetaCopyWith<$Res>  {
  factory $TabMetaCopyWith(TabMeta value, $Res Function(TabMeta) _then) = _$TabMetaCopyWithImpl;
@useResult
$Res call({
 String id, String title, bool locked
});




}
/// @nodoc
class _$TabMetaCopyWithImpl<$Res>
    implements $TabMetaCopyWith<$Res> {
  _$TabMetaCopyWithImpl(this._self, this._then);

  final TabMeta _self;
  final $Res Function(TabMeta) _then;

/// Create a copy of TabMeta
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? title = null,Object? locked = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [TabMeta].
extension TabMetaPatterns on TabMeta {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TabMeta value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TabMeta() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TabMeta value)  $default,){
final _that = this;
switch (_that) {
case _TabMeta():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TabMeta value)?  $default,){
final _that = this;
switch (_that) {
case _TabMeta() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String title,  bool locked)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TabMeta() when $default != null:
return $default(_that.id,_that.title,_that.locked);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String title,  bool locked)  $default,) {final _that = this;
switch (_that) {
case _TabMeta():
return $default(_that.id,_that.title,_that.locked);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String title,  bool locked)?  $default,) {final _that = this;
switch (_that) {
case _TabMeta() when $default != null:
return $default(_that.id,_that.title,_that.locked);case _:
  return null;

}
}

}

/// @nodoc


class _TabMeta extends TabMeta {
  const _TabMeta({required this.id, required this.title, this.locked = false}): super._();
  

@override final  String id;
@override final  String title;
@override@JsonKey() final  bool locked;

/// Create a copy of TabMeta
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TabMetaCopyWith<_TabMeta> get copyWith => __$TabMetaCopyWithImpl<_TabMeta>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TabMeta&&(identical(other.id, id) || other.id == id)&&(identical(other.title, title) || other.title == title)&&(identical(other.locked, locked) || other.locked == locked));
}


@override
int get hashCode => Object.hash(runtimeType,id,title,locked);

@override
String toString() {
  return 'TabMeta(id: $id, title: $title, locked: $locked)';
}


}

/// @nodoc
abstract mixin class _$TabMetaCopyWith<$Res> implements $TabMetaCopyWith<$Res> {
  factory _$TabMetaCopyWith(_TabMeta value, $Res Function(_TabMeta) _then) = __$TabMetaCopyWithImpl;
@override @useResult
$Res call({
 String id, String title, bool locked
});




}
/// @nodoc
class __$TabMetaCopyWithImpl<$Res>
    implements _$TabMetaCopyWith<$Res> {
  __$TabMetaCopyWithImpl(this._self, this._then);

  final _TabMeta _self;
  final $Res Function(_TabMeta) _then;

/// Create a copy of TabMeta
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? title = null,Object? locked = null,}) {
  return _then(_TabMeta(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
