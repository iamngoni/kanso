// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_drag_data.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteDragData {

/// The notes, in the order they are listed; never empty.
 List<String> get noteIds;/// Each note's title, in the same order; empty for a locked note.
 List<String> get titles;
/// Create a copy of NoteDragData
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDragDataCopyWith<NoteDragData> get copyWith => _$NoteDragDataCopyWithImpl<NoteDragData>(this as NoteDragData, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDragData&&const DeepCollectionEquality().equals(other.noteIds, noteIds)&&const DeepCollectionEquality().equals(other.titles, titles));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(noteIds),const DeepCollectionEquality().hash(titles));

@override
String toString() {
  return 'NoteDragData(noteIds: $noteIds, titles: $titles)';
}


}

/// @nodoc
abstract mixin class $NoteDragDataCopyWith<$Res>  {
  factory $NoteDragDataCopyWith(NoteDragData value, $Res Function(NoteDragData) _then) = _$NoteDragDataCopyWithImpl;
@useResult
$Res call({
 List<String> noteIds, List<String> titles
});




}
/// @nodoc
class _$NoteDragDataCopyWithImpl<$Res>
    implements $NoteDragDataCopyWith<$Res> {
  _$NoteDragDataCopyWithImpl(this._self, this._then);

  final NoteDragData _self;
  final $Res Function(NoteDragData) _then;

/// Create a copy of NoteDragData
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteIds = null,Object? titles = null,}) {
  return _then(_self.copyWith(
noteIds: null == noteIds ? _self.noteIds : noteIds // ignore: cast_nullable_to_non_nullable
as List<String>,titles: null == titles ? _self.titles : titles // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteDragData].
extension NoteDragDataPatterns on NoteDragData {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteDragData value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteDragData() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteDragData value)  $default,){
final _that = this;
switch (_that) {
case _NoteDragData():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteDragData value)?  $default,){
final _that = this;
switch (_that) {
case _NoteDragData() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<String> noteIds,  List<String> titles)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteDragData() when $default != null:
return $default(_that.noteIds,_that.titles);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<String> noteIds,  List<String> titles)  $default,) {final _that = this;
switch (_that) {
case _NoteDragData():
return $default(_that.noteIds,_that.titles);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<String> noteIds,  List<String> titles)?  $default,) {final _that = this;
switch (_that) {
case _NoteDragData() when $default != null:
return $default(_that.noteIds,_that.titles);case _:
  return null;

}
}

}

/// @nodoc


class _NoteDragData implements NoteDragData {
  const _NoteDragData({required final  List<String> noteIds, required final  List<String> titles}): _noteIds = noteIds,_titles = titles;
  

/// The notes, in the order they are listed; never empty.
 final  List<String> _noteIds;
/// The notes, in the order they are listed; never empty.
@override List<String> get noteIds {
  if (_noteIds is EqualUnmodifiableListView) return _noteIds;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_noteIds);
}

/// Each note's title, in the same order; empty for a locked note.
 final  List<String> _titles;
/// Each note's title, in the same order; empty for a locked note.
@override List<String> get titles {
  if (_titles is EqualUnmodifiableListView) return _titles;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_titles);
}


/// Create a copy of NoteDragData
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteDragDataCopyWith<_NoteDragData> get copyWith => __$NoteDragDataCopyWithImpl<_NoteDragData>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteDragData&&const DeepCollectionEquality().equals(other._noteIds, _noteIds)&&const DeepCollectionEquality().equals(other._titles, _titles));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_noteIds),const DeepCollectionEquality().hash(_titles));

@override
String toString() {
  return 'NoteDragData(noteIds: $noteIds, titles: $titles)';
}


}

/// @nodoc
abstract mixin class _$NoteDragDataCopyWith<$Res> implements $NoteDragDataCopyWith<$Res> {
  factory _$NoteDragDataCopyWith(_NoteDragData value, $Res Function(_NoteDragData) _then) = __$NoteDragDataCopyWithImpl;
@override @useResult
$Res call({
 List<String> noteIds, List<String> titles
});




}
/// @nodoc
class __$NoteDragDataCopyWithImpl<$Res>
    implements _$NoteDragDataCopyWith<$Res> {
  __$NoteDragDataCopyWithImpl(this._self, this._then);

  final _NoteDragData _self;
  final $Res Function(_NoteDragData) _then;

/// Create a copy of NoteDragData
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteIds = null,Object? titles = null,}) {
  return _then(_NoteDragData(
noteIds: null == noteIds ? _self._noteIds : noteIds // ignore: cast_nullable_to_non_nullable
as List<String>,titles: null == titles ? _self._titles : titles // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
