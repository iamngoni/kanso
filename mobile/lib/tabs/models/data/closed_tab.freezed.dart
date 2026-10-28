// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'closed_tab.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ClosedTab {

 TabMeta get meta; int get at;
/// Create a copy of ClosedTab
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ClosedTabCopyWith<ClosedTab> get copyWith => _$ClosedTabCopyWithImpl<ClosedTab>(this as ClosedTab, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ClosedTab&&(identical(other.meta, meta) || other.meta == meta)&&(identical(other.at, at) || other.at == at));
}


@override
int get hashCode => Object.hash(runtimeType,meta,at);

@override
String toString() {
  return 'ClosedTab(meta: $meta, at: $at)';
}


}

/// @nodoc
abstract mixin class $ClosedTabCopyWith<$Res>  {
  factory $ClosedTabCopyWith(ClosedTab value, $Res Function(ClosedTab) _then) = _$ClosedTabCopyWithImpl;
@useResult
$Res call({
 TabMeta meta, int at
});


$TabMetaCopyWith<$Res> get meta;

}
/// @nodoc
class _$ClosedTabCopyWithImpl<$Res>
    implements $ClosedTabCopyWith<$Res> {
  _$ClosedTabCopyWithImpl(this._self, this._then);

  final ClosedTab _self;
  final $Res Function(ClosedTab) _then;

/// Create a copy of ClosedTab
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? meta = null,Object? at = null,}) {
  return _then(_self.copyWith(
meta: null == meta ? _self.meta : meta // ignore: cast_nullable_to_non_nullable
as TabMeta,at: null == at ? _self.at : at // ignore: cast_nullable_to_non_nullable
as int,
  ));
}
/// Create a copy of ClosedTab
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$TabMetaCopyWith<$Res> get meta {
  
  return $TabMetaCopyWith<$Res>(_self.meta, (value) {
    return _then(_self.copyWith(meta: value));
  });
}
}


/// Adds pattern-matching-related methods to [ClosedTab].
extension ClosedTabPatterns on ClosedTab {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ClosedTab value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ClosedTab() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ClosedTab value)  $default,){
final _that = this;
switch (_that) {
case _ClosedTab():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ClosedTab value)?  $default,){
final _that = this;
switch (_that) {
case _ClosedTab() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( TabMeta meta,  int at)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ClosedTab() when $default != null:
return $default(_that.meta,_that.at);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( TabMeta meta,  int at)  $default,) {final _that = this;
switch (_that) {
case _ClosedTab():
return $default(_that.meta,_that.at);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( TabMeta meta,  int at)?  $default,) {final _that = this;
switch (_that) {
case _ClosedTab() when $default != null:
return $default(_that.meta,_that.at);case _:
  return null;

}
}

}

/// @nodoc


class _ClosedTab implements ClosedTab {
  const _ClosedTab({required this.meta, required this.at});
  

@override final  TabMeta meta;
@override final  int at;

/// Create a copy of ClosedTab
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ClosedTabCopyWith<_ClosedTab> get copyWith => __$ClosedTabCopyWithImpl<_ClosedTab>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ClosedTab&&(identical(other.meta, meta) || other.meta == meta)&&(identical(other.at, at) || other.at == at));
}


@override
int get hashCode => Object.hash(runtimeType,meta,at);

@override
String toString() {
  return 'ClosedTab(meta: $meta, at: $at)';
}


}

/// @nodoc
abstract mixin class _$ClosedTabCopyWith<$Res> implements $ClosedTabCopyWith<$Res> {
  factory _$ClosedTabCopyWith(_ClosedTab value, $Res Function(_ClosedTab) _then) = __$ClosedTabCopyWithImpl;
@override @useResult
$Res call({
 TabMeta meta, int at
});


@override $TabMetaCopyWith<$Res> get meta;

}
/// @nodoc
class __$ClosedTabCopyWithImpl<$Res>
    implements _$ClosedTabCopyWith<$Res> {
  __$ClosedTabCopyWithImpl(this._self, this._then);

  final _ClosedTab _self;
  final $Res Function(_ClosedTab) _then;

/// Create a copy of ClosedTab
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? meta = null,Object? at = null,}) {
  return _then(_ClosedTab(
meta: null == meta ? _self.meta : meta // ignore: cast_nullable_to_non_nullable
as TabMeta,at: null == at ? _self.at : at // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

/// Create a copy of ClosedTab
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$TabMetaCopyWith<$Res> get meta {
  
  return $TabMetaCopyWith<$Res>(_self.meta, (value) {
    return _then(_self.copyWith(meta: value));
  });
}
}

// dart format on
