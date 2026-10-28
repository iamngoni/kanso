// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'open_tabs.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$OpenTabs {

/// The tabs, left to right.
 List<OpenTab> get open;/// The index of the tab showing; 0 while none is open.
 int get active;/// The tabs closed, the newest last; at most [mostClosed].
 List<ClosedTab> get closed;
/// Create a copy of OpenTabs
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OpenTabsCopyWith<OpenTabs> get copyWith => _$OpenTabsCopyWithImpl<OpenTabs>(this as OpenTabs, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenTabs&&const DeepCollectionEquality().equals(other.open, open)&&(identical(other.active, active) || other.active == active)&&const DeepCollectionEquality().equals(other.closed, closed));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(open),active,const DeepCollectionEquality().hash(closed));

@override
String toString() {
  return 'OpenTabs(open: $open, active: $active, closed: $closed)';
}


}

/// @nodoc
abstract mixin class $OpenTabsCopyWith<$Res>  {
  factory $OpenTabsCopyWith(OpenTabs value, $Res Function(OpenTabs) _then) = _$OpenTabsCopyWithImpl;
@useResult
$Res call({
 List<OpenTab> open, int active, List<ClosedTab> closed
});




}
/// @nodoc
class _$OpenTabsCopyWithImpl<$Res>
    implements $OpenTabsCopyWith<$Res> {
  _$OpenTabsCopyWithImpl(this._self, this._then);

  final OpenTabs _self;
  final $Res Function(OpenTabs) _then;

/// Create a copy of OpenTabs
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? open = null,Object? active = null,Object? closed = null,}) {
  return _then(_self.copyWith(
open: null == open ? _self.open : open // ignore: cast_nullable_to_non_nullable
as List<OpenTab>,active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as int,closed: null == closed ? _self.closed : closed // ignore: cast_nullable_to_non_nullable
as List<ClosedTab>,
  ));
}

}


/// Adds pattern-matching-related methods to [OpenTabs].
extension OpenTabsPatterns on OpenTabs {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _OpenTabs value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _OpenTabs() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _OpenTabs value)  $default,){
final _that = this;
switch (_that) {
case _OpenTabs():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _OpenTabs value)?  $default,){
final _that = this;
switch (_that) {
case _OpenTabs() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<OpenTab> open,  int active,  List<ClosedTab> closed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _OpenTabs() when $default != null:
return $default(_that.open,_that.active,_that.closed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<OpenTab> open,  int active,  List<ClosedTab> closed)  $default,) {final _that = this;
switch (_that) {
case _OpenTabs():
return $default(_that.open,_that.active,_that.closed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<OpenTab> open,  int active,  List<ClosedTab> closed)?  $default,) {final _that = this;
switch (_that) {
case _OpenTabs() when $default != null:
return $default(_that.open,_that.active,_that.closed);case _:
  return null;

}
}

}

/// @nodoc


class _OpenTabs extends OpenTabs {
  const _OpenTabs({final  List<OpenTab> open = const <OpenTab>[], this.active = 0, final  List<ClosedTab> closed = const <ClosedTab>[]}): _open = open,_closed = closed,super._();
  

/// The tabs, left to right.
 final  List<OpenTab> _open;
/// The tabs, left to right.
@override@JsonKey() List<OpenTab> get open {
  if (_open is EqualUnmodifiableListView) return _open;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_open);
}

/// The index of the tab showing; 0 while none is open.
@override@JsonKey() final  int active;
/// The tabs closed, the newest last; at most [mostClosed].
 final  List<ClosedTab> _closed;
/// The tabs closed, the newest last; at most [mostClosed].
@override@JsonKey() List<ClosedTab> get closed {
  if (_closed is EqualUnmodifiableListView) return _closed;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_closed);
}


/// Create a copy of OpenTabs
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OpenTabsCopyWith<_OpenTabs> get copyWith => __$OpenTabsCopyWithImpl<_OpenTabs>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _OpenTabs&&const DeepCollectionEquality().equals(other._open, _open)&&(identical(other.active, active) || other.active == active)&&const DeepCollectionEquality().equals(other._closed, _closed));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_open),active,const DeepCollectionEquality().hash(_closed));

@override
String toString() {
  return 'OpenTabs(open: $open, active: $active, closed: $closed)';
}


}

/// @nodoc
abstract mixin class _$OpenTabsCopyWith<$Res> implements $OpenTabsCopyWith<$Res> {
  factory _$OpenTabsCopyWith(_OpenTabs value, $Res Function(_OpenTabs) _then) = __$OpenTabsCopyWithImpl;
@override @useResult
$Res call({
 List<OpenTab> open, int active, List<ClosedTab> closed
});




}
/// @nodoc
class __$OpenTabsCopyWithImpl<$Res>
    implements _$OpenTabsCopyWith<$Res> {
  __$OpenTabsCopyWithImpl(this._self, this._then);

  final _OpenTabs _self;
  final $Res Function(_OpenTabs) _then;

/// Create a copy of OpenTabs
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? open = null,Object? active = null,Object? closed = null,}) {
  return _then(_OpenTabs(
open: null == open ? _self._open : open // ignore: cast_nullable_to_non_nullable
as List<OpenTab>,active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as int,closed: null == closed ? _self._closed : closed // ignore: cast_nullable_to_non_nullable
as List<ClosedTab>,
  ));
}


}

// dart format on
