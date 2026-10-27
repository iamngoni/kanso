// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'kanso_icon_set.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$KansoIconSet {

/// The theme's folder name: `lucide`, `phosphor`.
 String get id;/// The name shown in Settings.
 String get name;/// The theme's SVG for each icon it draws, by Lucide name.
 Map<String, String> get svgs;
/// Create a copy of KansoIconSet
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoIconSetCopyWith<KansoIconSet> get copyWith => _$KansoIconSetCopyWithImpl<KansoIconSet>(this as KansoIconSet, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoIconSet&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other.svgs, svgs));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,const DeepCollectionEquality().hash(svgs));

@override
String toString() {
  return 'KansoIconSet(id: $id, name: $name, svgs: $svgs)';
}


}

/// @nodoc
abstract mixin class $KansoIconSetCopyWith<$Res>  {
  factory $KansoIconSetCopyWith(KansoIconSet value, $Res Function(KansoIconSet) _then) = _$KansoIconSetCopyWithImpl;
@useResult
$Res call({
 String id, String name, Map<String, String> svgs
});




}
/// @nodoc
class _$KansoIconSetCopyWithImpl<$Res>
    implements $KansoIconSetCopyWith<$Res> {
  _$KansoIconSetCopyWithImpl(this._self, this._then);

  final KansoIconSet _self;
  final $Res Function(KansoIconSet) _then;

/// Create a copy of KansoIconSet
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? svgs = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,svgs: null == svgs ? _self.svgs : svgs // ignore: cast_nullable_to_non_nullable
as Map<String, String>,
  ));
}

}


/// Adds pattern-matching-related methods to [KansoIconSet].
extension KansoIconSetPatterns on KansoIconSet {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KansoIconSet value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KansoIconSet() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KansoIconSet value)  $default,){
final _that = this;
switch (_that) {
case _KansoIconSet():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KansoIconSet value)?  $default,){
final _that = this;
switch (_that) {
case _KansoIconSet() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name,  Map<String, String> svgs)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KansoIconSet() when $default != null:
return $default(_that.id,_that.name,_that.svgs);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name,  Map<String, String> svgs)  $default,) {final _that = this;
switch (_that) {
case _KansoIconSet():
return $default(_that.id,_that.name,_that.svgs);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name,  Map<String, String> svgs)?  $default,) {final _that = this;
switch (_that) {
case _KansoIconSet() when $default != null:
return $default(_that.id,_that.name,_that.svgs);case _:
  return null;

}
}

}

/// @nodoc


class _KansoIconSet extends KansoIconSet {
  const _KansoIconSet({this.id = KansoIconSet.lucideId, this.name = KansoIconSet.lucideName, final  Map<String, String> svgs = const <String, String>{}}): _svgs = svgs,super._();
  

/// The theme's folder name: `lucide`, `phosphor`.
@override@JsonKey() final  String id;
/// The name shown in Settings.
@override@JsonKey() final  String name;
/// The theme's SVG for each icon it draws, by Lucide name.
 final  Map<String, String> _svgs;
/// The theme's SVG for each icon it draws, by Lucide name.
@override@JsonKey() Map<String, String> get svgs {
  if (_svgs is EqualUnmodifiableMapView) return _svgs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_svgs);
}


/// Create a copy of KansoIconSet
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KansoIconSetCopyWith<_KansoIconSet> get copyWith => __$KansoIconSetCopyWithImpl<_KansoIconSet>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KansoIconSet&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other._svgs, _svgs));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,const DeepCollectionEquality().hash(_svgs));

@override
String toString() {
  return 'KansoIconSet(id: $id, name: $name, svgs: $svgs)';
}


}

/// @nodoc
abstract mixin class _$KansoIconSetCopyWith<$Res> implements $KansoIconSetCopyWith<$Res> {
  factory _$KansoIconSetCopyWith(_KansoIconSet value, $Res Function(_KansoIconSet) _then) = __$KansoIconSetCopyWithImpl;
@override @useResult
$Res call({
 String id, String name, Map<String, String> svgs
});




}
/// @nodoc
class __$KansoIconSetCopyWithImpl<$Res>
    implements _$KansoIconSetCopyWith<$Res> {
  __$KansoIconSetCopyWithImpl(this._self, this._then);

  final _KansoIconSet _self;
  final $Res Function(_KansoIconSet) _then;

/// Create a copy of KansoIconSet
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? svgs = null,}) {
  return _then(_KansoIconSet(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,svgs: null == svgs ? _self._svgs : svgs // ignore: cast_nullable_to_non_nullable
as Map<String, String>,
  ));
}


}

// dart format on
