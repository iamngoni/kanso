// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dictated_text.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DictatedText {

 String get text;/// The caret's place in [text], in UTF-16 units. The editor puts the
/// caret after the dictated text; a caret inside it is for the editors
/// that can place one.
 int get caretUtf16;
/// Create a copy of DictatedText
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DictatedTextCopyWith<DictatedText> get copyWith => _$DictatedTextCopyWithImpl<DictatedText>(this as DictatedText, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DictatedText&&(identical(other.text, text) || other.text == text)&&(identical(other.caretUtf16, caretUtf16) || other.caretUtf16 == caretUtf16));
}


@override
int get hashCode => Object.hash(runtimeType,text,caretUtf16);

@override
String toString() {
  return 'DictatedText(text: $text, caretUtf16: $caretUtf16)';
}


}

/// @nodoc
abstract mixin class $DictatedTextCopyWith<$Res>  {
  factory $DictatedTextCopyWith(DictatedText value, $Res Function(DictatedText) _then) = _$DictatedTextCopyWithImpl;
@useResult
$Res call({
 String text, int caretUtf16
});




}
/// @nodoc
class _$DictatedTextCopyWithImpl<$Res>
    implements $DictatedTextCopyWith<$Res> {
  _$DictatedTextCopyWithImpl(this._self, this._then);

  final DictatedText _self;
  final $Res Function(DictatedText) _then;

/// Create a copy of DictatedText
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? text = null,Object? caretUtf16 = null,}) {
  return _then(_self.copyWith(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,caretUtf16: null == caretUtf16 ? _self.caretUtf16 : caretUtf16 // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [DictatedText].
extension DictatedTextPatterns on DictatedText {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DictatedText value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DictatedText() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DictatedText value)  $default,){
final _that = this;
switch (_that) {
case _DictatedText():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DictatedText value)?  $default,){
final _that = this;
switch (_that) {
case _DictatedText() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String text,  int caretUtf16)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DictatedText() when $default != null:
return $default(_that.text,_that.caretUtf16);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String text,  int caretUtf16)  $default,) {final _that = this;
switch (_that) {
case _DictatedText():
return $default(_that.text,_that.caretUtf16);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String text,  int caretUtf16)?  $default,) {final _that = this;
switch (_that) {
case _DictatedText() when $default != null:
return $default(_that.text,_that.caretUtf16);case _:
  return null;

}
}

}

/// @nodoc


class _DictatedText implements DictatedText {
  const _DictatedText({required this.text, required this.caretUtf16});
  

@override final  String text;
/// The caret's place in [text], in UTF-16 units. The editor puts the
/// caret after the dictated text; a caret inside it is for the editors
/// that can place one.
@override final  int caretUtf16;

/// Create a copy of DictatedText
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DictatedTextCopyWith<_DictatedText> get copyWith => __$DictatedTextCopyWithImpl<_DictatedText>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DictatedText&&(identical(other.text, text) || other.text == text)&&(identical(other.caretUtf16, caretUtf16) || other.caretUtf16 == caretUtf16));
}


@override
int get hashCode => Object.hash(runtimeType,text,caretUtf16);

@override
String toString() {
  return 'DictatedText(text: $text, caretUtf16: $caretUtf16)';
}


}

/// @nodoc
abstract mixin class _$DictatedTextCopyWith<$Res> implements $DictatedTextCopyWith<$Res> {
  factory _$DictatedTextCopyWith(_DictatedText value, $Res Function(_DictatedText) _then) = __$DictatedTextCopyWithImpl;
@override @useResult
$Res call({
 String text, int caretUtf16
});




}
/// @nodoc
class __$DictatedTextCopyWithImpl<$Res>
    implements _$DictatedTextCopyWith<$Res> {
  __$DictatedTextCopyWithImpl(this._self, this._then);

  final _DictatedText _self;
  final $Res Function(_DictatedText) _then;

/// Create a copy of DictatedText
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? text = null,Object? caretUtf16 = null,}) {
  return _then(_DictatedText(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,caretUtf16: null == caretUtf16 ? _self.caretUtf16 : caretUtf16 // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
