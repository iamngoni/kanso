// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'icon_theme_import_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$IconThemeImportResult {

/// The imported icon themes' ids.
 List<String> get ids;/// The imported icon themes' names, in the same order.
 List<String> get names;/// A sentence on what was imported (how many icons, which were left
/// out); empty when there is nothing to add.
 String get summary;
/// Create a copy of IconThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$IconThemeImportResultCopyWith<IconThemeImportResult> get copyWith => _$IconThemeImportResultCopyWithImpl<IconThemeImportResult>(this as IconThemeImportResult, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is IconThemeImportResult&&const DeepCollectionEquality().equals(other.ids, ids)&&const DeepCollectionEquality().equals(other.names, names)&&(identical(other.summary, summary) || other.summary == summary));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(ids),const DeepCollectionEquality().hash(names),summary);

@override
String toString() {
  return 'IconThemeImportResult(ids: $ids, names: $names, summary: $summary)';
}


}

/// @nodoc
abstract mixin class $IconThemeImportResultCopyWith<$Res>  {
  factory $IconThemeImportResultCopyWith(IconThemeImportResult value, $Res Function(IconThemeImportResult) _then) = _$IconThemeImportResultCopyWithImpl;
@useResult
$Res call({
 List<String> ids, List<String> names, String summary
});




}
/// @nodoc
class _$IconThemeImportResultCopyWithImpl<$Res>
    implements $IconThemeImportResultCopyWith<$Res> {
  _$IconThemeImportResultCopyWithImpl(this._self, this._then);

  final IconThemeImportResult _self;
  final $Res Function(IconThemeImportResult) _then;

/// Create a copy of IconThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? ids = null,Object? names = null,Object? summary = null,}) {
  return _then(_self.copyWith(
ids: null == ids ? _self.ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,names: null == names ? _self.names : names // ignore: cast_nullable_to_non_nullable
as List<String>,summary: null == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [IconThemeImportResult].
extension IconThemeImportResultPatterns on IconThemeImportResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _IconThemeImportResult value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _IconThemeImportResult() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _IconThemeImportResult value)  $default,){
final _that = this;
switch (_that) {
case _IconThemeImportResult():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _IconThemeImportResult value)?  $default,){
final _that = this;
switch (_that) {
case _IconThemeImportResult() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<String> ids,  List<String> names,  String summary)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _IconThemeImportResult() when $default != null:
return $default(_that.ids,_that.names,_that.summary);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<String> ids,  List<String> names,  String summary)  $default,) {final _that = this;
switch (_that) {
case _IconThemeImportResult():
return $default(_that.ids,_that.names,_that.summary);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<String> ids,  List<String> names,  String summary)?  $default,) {final _that = this;
switch (_that) {
case _IconThemeImportResult() when $default != null:
return $default(_that.ids,_that.names,_that.summary);case _:
  return null;

}
}

}

/// @nodoc


class _IconThemeImportResult implements IconThemeImportResult {
  const _IconThemeImportResult({required final  List<String> ids, required final  List<String> names, required this.summary}): _ids = ids,_names = names;
  

/// The imported icon themes' ids.
 final  List<String> _ids;
/// The imported icon themes' ids.
@override List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}

/// The imported icon themes' names, in the same order.
 final  List<String> _names;
/// The imported icon themes' names, in the same order.
@override List<String> get names {
  if (_names is EqualUnmodifiableListView) return _names;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_names);
}

/// A sentence on what was imported (how many icons, which were left
/// out); empty when there is nothing to add.
@override final  String summary;

/// Create a copy of IconThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$IconThemeImportResultCopyWith<_IconThemeImportResult> get copyWith => __$IconThemeImportResultCopyWithImpl<_IconThemeImportResult>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _IconThemeImportResult&&const DeepCollectionEquality().equals(other._ids, _ids)&&const DeepCollectionEquality().equals(other._names, _names)&&(identical(other.summary, summary) || other.summary == summary));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids),const DeepCollectionEquality().hash(_names),summary);

@override
String toString() {
  return 'IconThemeImportResult(ids: $ids, names: $names, summary: $summary)';
}


}

/// @nodoc
abstract mixin class _$IconThemeImportResultCopyWith<$Res> implements $IconThemeImportResultCopyWith<$Res> {
  factory _$IconThemeImportResultCopyWith(_IconThemeImportResult value, $Res Function(_IconThemeImportResult) _then) = __$IconThemeImportResultCopyWithImpl;
@override @useResult
$Res call({
 List<String> ids, List<String> names, String summary
});




}
/// @nodoc
class __$IconThemeImportResultCopyWithImpl<$Res>
    implements _$IconThemeImportResultCopyWith<$Res> {
  __$IconThemeImportResultCopyWithImpl(this._self, this._then);

  final _IconThemeImportResult _self;
  final $Res Function(_IconThemeImportResult) _then;

/// Create a copy of IconThemeImportResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? ids = null,Object? names = null,Object? summary = null,}) {
  return _then(_IconThemeImportResult(
ids: null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,names: null == names ? _self._names : names // ignore: cast_nullable_to_non_nullable
as List<String>,summary: null == summary ? _self.summary : summary // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
