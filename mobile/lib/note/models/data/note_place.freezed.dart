// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_place.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NotePlace {

/// The caret, in UTF-16 units of the note's text.
 int get caret;/// The scroll offset of the editor's page.
 double get scroll;
/// Create a copy of NotePlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotePlaceCopyWith<NotePlace> get copyWith => _$NotePlaceCopyWithImpl<NotePlace>(this as NotePlace, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotePlace&&(identical(other.caret, caret) || other.caret == caret)&&(identical(other.scroll, scroll) || other.scroll == scroll));
}


@override
int get hashCode => Object.hash(runtimeType,caret,scroll);

@override
String toString() {
  return 'NotePlace(caret: $caret, scroll: $scroll)';
}


}

/// @nodoc
abstract mixin class $NotePlaceCopyWith<$Res>  {
  factory $NotePlaceCopyWith(NotePlace value, $Res Function(NotePlace) _then) = _$NotePlaceCopyWithImpl;
@useResult
$Res call({
 int caret, double scroll
});




}
/// @nodoc
class _$NotePlaceCopyWithImpl<$Res>
    implements $NotePlaceCopyWith<$Res> {
  _$NotePlaceCopyWithImpl(this._self, this._then);

  final NotePlace _self;
  final $Res Function(NotePlace) _then;

/// Create a copy of NotePlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? caret = null,Object? scroll = null,}) {
  return _then(_self.copyWith(
caret: null == caret ? _self.caret : caret // ignore: cast_nullable_to_non_nullable
as int,scroll: null == scroll ? _self.scroll : scroll // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [NotePlace].
extension NotePlacePatterns on NotePlace {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NotePlace value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NotePlace() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NotePlace value)  $default,){
final _that = this;
switch (_that) {
case _NotePlace():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NotePlace value)?  $default,){
final _that = this;
switch (_that) {
case _NotePlace() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int caret,  double scroll)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NotePlace() when $default != null:
return $default(_that.caret,_that.scroll);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int caret,  double scroll)  $default,) {final _that = this;
switch (_that) {
case _NotePlace():
return $default(_that.caret,_that.scroll);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int caret,  double scroll)?  $default,) {final _that = this;
switch (_that) {
case _NotePlace() when $default != null:
return $default(_that.caret,_that.scroll);case _:
  return null;

}
}

}

/// @nodoc


class _NotePlace implements NotePlace {
  const _NotePlace({required this.caret, required this.scroll});
  

/// The caret, in UTF-16 units of the note's text.
@override final  int caret;
/// The scroll offset of the editor's page.
@override final  double scroll;

/// Create a copy of NotePlace
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NotePlaceCopyWith<_NotePlace> get copyWith => __$NotePlaceCopyWithImpl<_NotePlace>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NotePlace&&(identical(other.caret, caret) || other.caret == caret)&&(identical(other.scroll, scroll) || other.scroll == scroll));
}


@override
int get hashCode => Object.hash(runtimeType,caret,scroll);

@override
String toString() {
  return 'NotePlace(caret: $caret, scroll: $scroll)';
}


}

/// @nodoc
abstract mixin class _$NotePlaceCopyWith<$Res> implements $NotePlaceCopyWith<$Res> {
  factory _$NotePlaceCopyWith(_NotePlace value, $Res Function(_NotePlace) _then) = __$NotePlaceCopyWithImpl;
@override @useResult
$Res call({
 int caret, double scroll
});




}
/// @nodoc
class __$NotePlaceCopyWithImpl<$Res>
    implements _$NotePlaceCopyWith<$Res> {
  __$NotePlaceCopyWithImpl(this._self, this._then);

  final _NotePlace _self;
  final $Res Function(_NotePlace) _then;

/// Create a copy of NotePlace
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? caret = null,Object? scroll = null,}) {
  return _then(_NotePlace(
caret: null == caret ? _self.caret : caret // ignore: cast_nullable_to_non_nullable
as int,scroll: null == scroll ? _self.scroll : scroll // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

// dart format on
