// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'open_tab.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$OpenTab {

 TabMeta get meta; NavigationHistory get history;
/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$OpenTabCopyWith<OpenTab> get copyWith => _$OpenTabCopyWithImpl<OpenTab>(this as OpenTab, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is OpenTab&&(identical(other.meta, meta) || other.meta == meta)&&(identical(other.history, history) || other.history == history));
}


@override
int get hashCode => Object.hash(runtimeType,meta,history);

@override
String toString() {
  return 'OpenTab(meta: $meta, history: $history)';
}


}

/// @nodoc
abstract mixin class $OpenTabCopyWith<$Res>  {
  factory $OpenTabCopyWith(OpenTab value, $Res Function(OpenTab) _then) = _$OpenTabCopyWithImpl;
@useResult
$Res call({
 TabMeta meta, NavigationHistory history
});


$TabMetaCopyWith<$Res> get meta;$NavigationHistoryCopyWith<$Res> get history;

}
/// @nodoc
class _$OpenTabCopyWithImpl<$Res>
    implements $OpenTabCopyWith<$Res> {
  _$OpenTabCopyWithImpl(this._self, this._then);

  final OpenTab _self;
  final $Res Function(OpenTab) _then;

/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? meta = null,Object? history = null,}) {
  return _then(_self.copyWith(
meta: null == meta ? _self.meta : meta // ignore: cast_nullable_to_non_nullable
as TabMeta,history: null == history ? _self.history : history // ignore: cast_nullable_to_non_nullable
as NavigationHistory,
  ));
}
/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$TabMetaCopyWith<$Res> get meta {
  
  return $TabMetaCopyWith<$Res>(_self.meta, (value) {
    return _then(_self.copyWith(meta: value));
  });
}/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NavigationHistoryCopyWith<$Res> get history {
  
  return $NavigationHistoryCopyWith<$Res>(_self.history, (value) {
    return _then(_self.copyWith(history: value));
  });
}
}


/// Adds pattern-matching-related methods to [OpenTab].
extension OpenTabPatterns on OpenTab {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _OpenTab value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _OpenTab() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _OpenTab value)  $default,){
final _that = this;
switch (_that) {
case _OpenTab():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _OpenTab value)?  $default,){
final _that = this;
switch (_that) {
case _OpenTab() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( TabMeta meta,  NavigationHistory history)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _OpenTab() when $default != null:
return $default(_that.meta,_that.history);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( TabMeta meta,  NavigationHistory history)  $default,) {final _that = this;
switch (_that) {
case _OpenTab():
return $default(_that.meta,_that.history);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( TabMeta meta,  NavigationHistory history)?  $default,) {final _that = this;
switch (_that) {
case _OpenTab() when $default != null:
return $default(_that.meta,_that.history);case _:
  return null;

}
}

}

/// @nodoc


class _OpenTab extends OpenTab {
  const _OpenTab({required this.meta, required this.history}): super._();
  

@override final  TabMeta meta;
@override final  NavigationHistory history;

/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$OpenTabCopyWith<_OpenTab> get copyWith => __$OpenTabCopyWithImpl<_OpenTab>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _OpenTab&&(identical(other.meta, meta) || other.meta == meta)&&(identical(other.history, history) || other.history == history));
}


@override
int get hashCode => Object.hash(runtimeType,meta,history);

@override
String toString() {
  return 'OpenTab(meta: $meta, history: $history)';
}


}

/// @nodoc
abstract mixin class _$OpenTabCopyWith<$Res> implements $OpenTabCopyWith<$Res> {
  factory _$OpenTabCopyWith(_OpenTab value, $Res Function(_OpenTab) _then) = __$OpenTabCopyWithImpl;
@override @useResult
$Res call({
 TabMeta meta, NavigationHistory history
});


@override $TabMetaCopyWith<$Res> get meta;@override $NavigationHistoryCopyWith<$Res> get history;

}
/// @nodoc
class __$OpenTabCopyWithImpl<$Res>
    implements _$OpenTabCopyWith<$Res> {
  __$OpenTabCopyWithImpl(this._self, this._then);

  final _OpenTab _self;
  final $Res Function(_OpenTab) _then;

/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? meta = null,Object? history = null,}) {
  return _then(_OpenTab(
meta: null == meta ? _self.meta : meta // ignore: cast_nullable_to_non_nullable
as TabMeta,history: null == history ? _self.history : history // ignore: cast_nullable_to_non_nullable
as NavigationHistory,
  ));
}

/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$TabMetaCopyWith<$Res> get meta {
  
  return $TabMetaCopyWith<$Res>(_self.meta, (value) {
    return _then(_self.copyWith(meta: value));
  });
}/// Create a copy of OpenTab
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NavigationHistoryCopyWith<$Res> get history {
  
  return $NavigationHistoryCopyWith<$Res>(_self.history, (value) {
    return _then(_self.copyWith(history: value));
  });
}
}

// dart format on
