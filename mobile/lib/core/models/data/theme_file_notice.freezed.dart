// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_file_notice.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemeFileNotice {

/// The file's name.
 String get file;/// What is wrong with it, in a sentence.
 String get sentence;
/// Create a copy of ThemeFileNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeFileNoticeCopyWith<ThemeFileNotice> get copyWith => _$ThemeFileNoticeCopyWithImpl<ThemeFileNotice>(this as ThemeFileNotice, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeFileNotice&&(identical(other.file, file) || other.file == file)&&(identical(other.sentence, sentence) || other.sentence == sentence));
}


@override
int get hashCode => Object.hash(runtimeType,file,sentence);

@override
String toString() {
  return 'ThemeFileNotice(file: $file, sentence: $sentence)';
}


}

/// @nodoc
abstract mixin class $ThemeFileNoticeCopyWith<$Res>  {
  factory $ThemeFileNoticeCopyWith(ThemeFileNotice value, $Res Function(ThemeFileNotice) _then) = _$ThemeFileNoticeCopyWithImpl;
@useResult
$Res call({
 String file, String sentence
});




}
/// @nodoc
class _$ThemeFileNoticeCopyWithImpl<$Res>
    implements $ThemeFileNoticeCopyWith<$Res> {
  _$ThemeFileNoticeCopyWithImpl(this._self, this._then);

  final ThemeFileNotice _self;
  final $Res Function(ThemeFileNotice) _then;

/// Create a copy of ThemeFileNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? file = null,Object? sentence = null,}) {
  return _then(_self.copyWith(
file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,sentence: null == sentence ? _self.sentence : sentence // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [ThemeFileNotice].
extension ThemeFileNoticePatterns on ThemeFileNotice {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemeFileNotice value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemeFileNotice() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemeFileNotice value)  $default,){
final _that = this;
switch (_that) {
case _ThemeFileNotice():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemeFileNotice value)?  $default,){
final _that = this;
switch (_that) {
case _ThemeFileNotice() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String file,  String sentence)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemeFileNotice() when $default != null:
return $default(_that.file,_that.sentence);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String file,  String sentence)  $default,) {final _that = this;
switch (_that) {
case _ThemeFileNotice():
return $default(_that.file,_that.sentence);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String file,  String sentence)?  $default,) {final _that = this;
switch (_that) {
case _ThemeFileNotice() when $default != null:
return $default(_that.file,_that.sentence);case _:
  return null;

}
}

}

/// @nodoc


class _ThemeFileNotice implements ThemeFileNotice {
  const _ThemeFileNotice({required this.file, required this.sentence});
  

/// The file's name.
@override final  String file;
/// What is wrong with it, in a sentence.
@override final  String sentence;

/// Create a copy of ThemeFileNotice
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemeFileNoticeCopyWith<_ThemeFileNotice> get copyWith => __$ThemeFileNoticeCopyWithImpl<_ThemeFileNotice>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemeFileNotice&&(identical(other.file, file) || other.file == file)&&(identical(other.sentence, sentence) || other.sentence == sentence));
}


@override
int get hashCode => Object.hash(runtimeType,file,sentence);

@override
String toString() {
  return 'ThemeFileNotice(file: $file, sentence: $sentence)';
}


}

/// @nodoc
abstract mixin class _$ThemeFileNoticeCopyWith<$Res> implements $ThemeFileNoticeCopyWith<$Res> {
  factory _$ThemeFileNoticeCopyWith(_ThemeFileNotice value, $Res Function(_ThemeFileNotice) _then) = __$ThemeFileNoticeCopyWithImpl;
@override @useResult
$Res call({
 String file, String sentence
});




}
/// @nodoc
class __$ThemeFileNoticeCopyWithImpl<$Res>
    implements _$ThemeFileNoticeCopyWith<$Res> {
  __$ThemeFileNoticeCopyWithImpl(this._self, this._then);

  final _ThemeFileNotice _self;
  final $Res Function(_ThemeFileNotice) _then;

/// Create a copy of ThemeFileNotice
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? file = null,Object? sentence = null,}) {
  return _then(_ThemeFileNotice(
file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,sentence: null == sentence ? _self.sentence : sentence // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
