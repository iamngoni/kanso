// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_import_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemeImportResult {

/// The imported themes' ids.
 List<String> get ids;/// The imported themes' names, in the same order.
 List<String> get names;
/// Create a copy of ThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeImportResultCopyWith<ThemeImportResult> get copyWith => _$ThemeImportResultCopyWithImpl<ThemeImportResult>(this as ThemeImportResult, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeImportResult&&const DeepCollectionEquality().equals(other.ids, ids)&&const DeepCollectionEquality().equals(other.names, names));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(ids),const DeepCollectionEquality().hash(names));

@override
String toString() {
  return 'ThemeImportResult(ids: $ids, names: $names)';
}


}

/// @nodoc
abstract mixin class $ThemeImportResultCopyWith<$Res>  {
  factory $ThemeImportResultCopyWith(ThemeImportResult value, $Res Function(ThemeImportResult) _then) = _$ThemeImportResultCopyWithImpl;
@useResult
$Res call({
 List<String> ids, List<String> names
});




}
/// @nodoc
class _$ThemeImportResultCopyWithImpl<$Res>
    implements $ThemeImportResultCopyWith<$Res> {
  _$ThemeImportResultCopyWithImpl(this._self, this._then);

  final ThemeImportResult _self;
  final $Res Function(ThemeImportResult) _then;

/// Create a copy of ThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? ids = null,Object? names = null,}) {
  return _then(_self.copyWith(
ids: null == ids ? _self.ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,names: null == names ? _self.names : names // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [ThemeImportResult].
extension ThemeImportResultPatterns on ThemeImportResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemeImportResult value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemeImportResult() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemeImportResult value)  $default,){
final _that = this;
switch (_that) {
case _ThemeImportResult():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemeImportResult value)?  $default,){
final _that = this;
switch (_that) {
case _ThemeImportResult() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<String> ids,  List<String> names)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemeImportResult() when $default != null:
return $default(_that.ids,_that.names);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<String> ids,  List<String> names)  $default,) {final _that = this;
switch (_that) {
case _ThemeImportResult():
return $default(_that.ids,_that.names);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<String> ids,  List<String> names)?  $default,) {final _that = this;
switch (_that) {
case _ThemeImportResult() when $default != null:
return $default(_that.ids,_that.names);case _:
  return null;

}
}

}

/// @nodoc


class _ThemeImportResult implements ThemeImportResult {
  const _ThemeImportResult({required final  List<String> ids, required final  List<String> names}): _ids = ids,_names = names;
  

/// The imported themes' ids.
 final  List<String> _ids;
/// The imported themes' ids.
@override List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}

/// The imported themes' names, in the same order.
 final  List<String> _names;
/// The imported themes' names, in the same order.
@override List<String> get names {
  if (_names is EqualUnmodifiableListView) return _names;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_names);
}


/// Create a copy of ThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemeImportResultCopyWith<_ThemeImportResult> get copyWith => __$ThemeImportResultCopyWithImpl<_ThemeImportResult>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemeImportResult&&const DeepCollectionEquality().equals(other._ids, _ids)&&const DeepCollectionEquality().equals(other._names, _names));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids),const DeepCollectionEquality().hash(_names));

@override
String toString() {
  return 'ThemeImportResult(ids: $ids, names: $names)';
}


}

/// @nodoc
abstract mixin class _$ThemeImportResultCopyWith<$Res> implements $ThemeImportResultCopyWith<$Res> {
  factory _$ThemeImportResultCopyWith(_ThemeImportResult value, $Res Function(_ThemeImportResult) _then) = __$ThemeImportResultCopyWithImpl;
@override @useResult
$Res call({
 List<String> ids, List<String> names
});




}
/// @nodoc
class __$ThemeImportResultCopyWithImpl<$Res>
    implements _$ThemeImportResultCopyWith<$Res> {
  __$ThemeImportResultCopyWithImpl(this._self, this._then);

  final _ThemeImportResult _self;
  final $Res Function(_ThemeImportResult) _then;

/// Create a copy of ThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? ids = null,Object? names = null,}) {
  return _then(_ThemeImportResult(
ids: null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,names: null == names ? _self._names : names // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
