// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dictation_edit.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DictationEdit {

/// What the edit replaces, in the note's text now.
 TextRange get range;/// What goes in its place, spacing included.
 String get text;
/// Create a copy of DictationEdit
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictationEditCopyWith<DictationEdit> get copyWith => _$DictationEditCopyWithImpl<DictationEdit>(this as DictationEdit, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictationEdit&&(identical(other.range, range) || other.range == range)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,range,text);

@override
String toString() {
  return 'DictationEdit(range: $range, text: $text)';
}


}

/// @nodoc
abstract mixin class $DictationEditCopyWith<$Res>  {
  factory $DictationEditCopyWith(DictationEdit value, $Res Function(DictationEdit) _then) = _$DictationEditCopyWithImpl;
@useResult
$Res call({
 TextRange range, String text
});




}
/// @nodoc
class _$DictationEditCopyWithImpl<$Res>
    implements $DictationEditCopyWith<$Res> {
  _$DictationEditCopyWithImpl(this._self, this._then);

  final DictationEdit _self;
  final $Res Function(DictationEdit) _then;

/// Create a copy of DictationEdit
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? range = null,Object? text = null,}) {
  return _then(_self.copyWith(
range: null == range ? _self.range : range // ignore: cast_nullable_to_non_nullable
as TextRange,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [DictationEdit].
extension DictationEditPatterns on DictationEdit {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DictationEdit value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DictationEdit() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DictationEdit value)  $default,){
final _that = this;
switch (_that) {
case _DictationEdit():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DictationEdit value)?  $default,){
final _that = this;
switch (_that) {
case _DictationEdit() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( TextRange range,  String text)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DictationEdit() when $default != null:
return $default(_that.range,_that.text);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( TextRange range,  String text)  $default,) {final _that = this;
switch (_that) {
case _DictationEdit():
return $default(_that.range,_that.text);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( TextRange range,  String text)?  $default,) {final _that = this;
switch (_that) {
case _DictationEdit() when $default != null:
return $default(_that.range,_that.text);case _:
  return null;

}
}

}

/// @nodoc


class _DictationEdit implements DictationEdit {
  const _DictationEdit({required this.range, required this.text});
  

/// What the edit replaces, in the note's text now.
@override final  TextRange range;
/// What goes in its place, spacing included.
@override final  String text;

/// Create a copy of DictationEdit
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DictationEditCopyWith<_DictationEdit> get copyWith => __$DictationEditCopyWithImpl<_DictationEdit>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DictationEdit&&(identical(other.range, range) || other.range == range)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,range,text);

@override
String toString() {
  return 'DictationEdit(range: $range, text: $text)';
}


}

/// @nodoc
abstract mixin class _$DictationEditCopyWith<$Res> implements $DictationEditCopyWith<$Res> {
  factory _$DictationEditCopyWith(_DictationEdit value, $Res Function(_DictationEdit) _then) = __$DictationEditCopyWithImpl;
@override @useResult
$Res call({
 TextRange range, String text
});




}
/// @nodoc
class __$DictationEditCopyWithImpl<$Res>
    implements _$DictationEditCopyWith<$Res> {
  __$DictationEditCopyWithImpl(this._self, this._then);

  final _DictationEdit _self;
  final $Res Function(_DictationEdit) _then;

/// Create a copy of DictationEdit
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? range = null,Object? text = null,}) {
  return _then(_DictationEdit(
range: null == range ? _self.range : range // ignore: cast_nullable_to_non_nullable
as TextRange,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
