// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'text_diff.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TextDiff {

 int get start; int get end; String get text;
/// Create a copy of TextDiff
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TextDiffCopyWith<TextDiff> get copyWith => _$TextDiffCopyWithImpl<TextDiff>(this as TextDiff, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TextDiff&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,start,end,text);

@override
String toString() {
  return 'TextDiff(start: $start, end: $end, text: $text)';
}


}

/// @nodoc
abstract mixin class $TextDiffCopyWith<$Res>  {
  factory $TextDiffCopyWith(TextDiff value, $Res Function(TextDiff) _then) = _$TextDiffCopyWithImpl;
@useResult
$Res call({
 int start, int end, String text
});




}
/// @nodoc
class _$TextDiffCopyWithImpl<$Res>
    implements $TextDiffCopyWith<$Res> {
  _$TextDiffCopyWithImpl(this._self, this._then);

  final TextDiff _self;
  final $Res Function(TextDiff) _then;

/// Create a copy of TextDiff
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? start = null,Object? end = null,Object? text = null,}) {
  return _then(_self.copyWith(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [TextDiff].
extension TextDiffPatterns on TextDiff {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TextDiff value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TextDiff() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TextDiff value)  $default,){
final _that = this;
switch (_that) {
case _TextDiff():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TextDiff value)?  $default,){
final _that = this;
switch (_that) {
case _TextDiff() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int start,  int end,  String text)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TextDiff() when $default != null:
return $default(_that.start,_that.end,_that.text);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int start,  int end,  String text)  $default,) {final _that = this;
switch (_that) {
case _TextDiff():
return $default(_that.start,_that.end,_that.text);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int start,  int end,  String text)?  $default,) {final _that = this;
switch (_that) {
case _TextDiff() when $default != null:
return $default(_that.start,_that.end,_that.text);case _:
  return null;

}
}

}

/// @nodoc


class _TextDiff extends TextDiff {
  const _TextDiff({required this.start, required this.end, required this.text}): super._();
  

@override final  int start;
@override final  int end;
@override final  String text;

/// Create a copy of TextDiff
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TextDiffCopyWith<_TextDiff> get copyWith => __$TextDiffCopyWithImpl<_TextDiff>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TextDiff&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,start,end,text);

@override
String toString() {
  return 'TextDiff(start: $start, end: $end, text: $text)';
}


}

/// @nodoc
abstract mixin class _$TextDiffCopyWith<$Res> implements $TextDiffCopyWith<$Res> {
  factory _$TextDiffCopyWith(_TextDiff value, $Res Function(_TextDiff) _then) = __$TextDiffCopyWithImpl;
@override @useResult
$Res call({
 int start, int end, String text
});




}
/// @nodoc
class __$TextDiffCopyWithImpl<$Res>
    implements _$TextDiffCopyWith<$Res> {
  __$TextDiffCopyWithImpl(this._self, this._then);

  final _TextDiff _self;
  final $Res Function(_TextDiff) _then;

/// Create a copy of TextDiff
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? start = null,Object? end = null,Object? text = null,}) {
  return _then(_TextDiff(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
