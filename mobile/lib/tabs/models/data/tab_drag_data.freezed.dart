// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'tab_drag_data.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TabDragData {

/// Where in the row the tab was lifted from.
 int get from;/// The title it shows, for the lifted copy.
 String get title;
/// Create a copy of TabDragData
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TabDragDataCopyWith<TabDragData> get copyWith => _$TabDragDataCopyWithImpl<TabDragData>(this as TabDragData, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TabDragData&&(identical(other.from, from) || other.from == from)&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,from,title);

@override
String toString() {
  return 'TabDragData(from: $from, title: $title)';
}


}

/// @nodoc
abstract mixin class $TabDragDataCopyWith<$Res>  {
  factory $TabDragDataCopyWith(TabDragData value, $Res Function(TabDragData) _then) = _$TabDragDataCopyWithImpl;
@useResult
$Res call({
 int from, String title
});




}
/// @nodoc
class _$TabDragDataCopyWithImpl<$Res>
    implements $TabDragDataCopyWith<$Res> {
  _$TabDragDataCopyWithImpl(this._self, this._then);

  final TabDragData _self;
  final $Res Function(TabDragData) _then;

/// Create a copy of TabDragData
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? from = null,Object? title = null,}) {
  return _then(_self.copyWith(
from: null == from ? _self.from : from // ignore: cast_nullable_to_non_nullable
as int,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [TabDragData].
extension TabDragDataPatterns on TabDragData {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TabDragData value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TabDragData() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TabDragData value)  $default,){
final _that = this;
switch (_that) {
case _TabDragData():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TabDragData value)?  $default,){
final _that = this;
switch (_that) {
case _TabDragData() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int from,  String title)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TabDragData() when $default != null:
return $default(_that.from,_that.title);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int from,  String title)  $default,) {final _that = this;
switch (_that) {
case _TabDragData():
return $default(_that.from,_that.title);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int from,  String title)?  $default,) {final _that = this;
switch (_that) {
case _TabDragData() when $default != null:
return $default(_that.from,_that.title);case _:
  return null;

}
}

}

/// @nodoc


class _TabDragData implements TabDragData {
  const _TabDragData({required this.from, required this.title});
  

/// Where in the row the tab was lifted from.
@override final  int from;
/// The title it shows, for the lifted copy.
@override final  String title;

/// Create a copy of TabDragData
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TabDragDataCopyWith<_TabDragData> get copyWith => __$TabDragDataCopyWithImpl<_TabDragData>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TabDragData&&(identical(other.from, from) || other.from == from)&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,from,title);

@override
String toString() {
  return 'TabDragData(from: $from, title: $title)';
}


}

/// @nodoc
abstract mixin class _$TabDragDataCopyWith<$Res> implements $TabDragDataCopyWith<$Res> {
  factory _$TabDragDataCopyWith(_TabDragData value, $Res Function(_TabDragData) _then) = __$TabDragDataCopyWithImpl;
@override @useResult
$Res call({
 int from, String title
});




}
/// @nodoc
class __$TabDragDataCopyWithImpl<$Res>
    implements _$TabDragDataCopyWith<$Res> {
  __$TabDragDataCopyWithImpl(this._self, this._then);

  final _TabDragData _self;
  final $Res Function(_TabDragData) _then;

/// Create a copy of TabDragData
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? from = null,Object? title = null,}) {
  return _then(_TabDragData(
from: null == from ? _self.from : from // ignore: cast_nullable_to_non_nullable
as int,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
