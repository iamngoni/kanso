// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'graph_viewport.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$GraphViewport {

 Offset get center; double get zoom;
/// Create a copy of GraphViewport
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GraphViewportCopyWith<GraphViewport> get copyWith => _$GraphViewportCopyWithImpl<GraphViewport>(this as GraphViewport, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GraphViewport&&(identical(other.center, center) || other.center == center)&&(identical(other.zoom, zoom) || other.zoom == zoom));
}


@override
int get hashCode => Object.hash(runtimeType,center,zoom);

@override
String toString() {
  return 'GraphViewport(center: $center, zoom: $zoom)';
}


}

/// @nodoc
abstract mixin class $GraphViewportCopyWith<$Res>  {
  factory $GraphViewportCopyWith(GraphViewport value, $Res Function(GraphViewport) _then) = _$GraphViewportCopyWithImpl;
@useResult
$Res call({
 Offset center, double zoom
});




}
/// @nodoc
class _$GraphViewportCopyWithImpl<$Res>
    implements $GraphViewportCopyWith<$Res> {
  _$GraphViewportCopyWithImpl(this._self, this._then);

  final GraphViewport _self;
  final $Res Function(GraphViewport) _then;

/// Create a copy of GraphViewport
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? center = null,Object? zoom = null,}) {
  return _then(_self.copyWith(
center: null == center ? _self.center : center // ignore: cast_nullable_to_non_nullable
as Offset,zoom: null == zoom ? _self.zoom : zoom // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [GraphViewport].
extension GraphViewportPatterns on GraphViewport {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _GraphViewport value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _GraphViewport() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _GraphViewport value)  $default,){
final _that = this;
switch (_that) {
case _GraphViewport():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _GraphViewport value)?  $default,){
final _that = this;
switch (_that) {
case _GraphViewport() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( Offset center,  double zoom)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _GraphViewport() when $default != null:
return $default(_that.center,_that.zoom);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( Offset center,  double zoom)  $default,) {final _that = this;
switch (_that) {
case _GraphViewport():
return $default(_that.center,_that.zoom);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( Offset center,  double zoom)?  $default,) {final _that = this;
switch (_that) {
case _GraphViewport() when $default != null:
return $default(_that.center,_that.zoom);case _:
  return null;

}
}

}

/// @nodoc


class _GraphViewport extends GraphViewport {
  const _GraphViewport({this.center = Offset.zero, this.zoom = 1.0}): super._();
  

@override@JsonKey() final  Offset center;
@override@JsonKey() final  double zoom;

/// Create a copy of GraphViewport
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$GraphViewportCopyWith<_GraphViewport> get copyWith => __$GraphViewportCopyWithImpl<_GraphViewport>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _GraphViewport&&(identical(other.center, center) || other.center == center)&&(identical(other.zoom, zoom) || other.zoom == zoom));
}


@override
int get hashCode => Object.hash(runtimeType,center,zoom);

@override
String toString() {
  return 'GraphViewport(center: $center, zoom: $zoom)';
}


}

/// @nodoc
abstract mixin class _$GraphViewportCopyWith<$Res> implements $GraphViewportCopyWith<$Res> {
  factory _$GraphViewportCopyWith(_GraphViewport value, $Res Function(_GraphViewport) _then) = __$GraphViewportCopyWithImpl;
@override @useResult
$Res call({
 Offset center, double zoom
});




}
/// @nodoc
class __$GraphViewportCopyWithImpl<$Res>
    implements _$GraphViewportCopyWith<$Res> {
  __$GraphViewportCopyWithImpl(this._self, this._then);

  final _GraphViewport _self;
  final $Res Function(_GraphViewport) _then;

/// Create a copy of GraphViewport
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? center = null,Object? zoom = null,}) {
  return _then(_GraphViewport(
center: null == center ? _self.center : center // ignore: cast_nullable_to_non_nullable
as Offset,zoom: null == zoom ? _self.zoom : zoom // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

// dart format on
