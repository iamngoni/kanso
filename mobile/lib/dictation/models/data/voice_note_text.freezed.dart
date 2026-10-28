// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voice_note_text.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$VoiceNoteText {

 String get title; String get body;
/// Create a copy of VoiceNoteText
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VoiceNoteTextCopyWith<VoiceNoteText> get copyWith => _$VoiceNoteTextCopyWithImpl<VoiceNoteText>(this as VoiceNoteText, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VoiceNoteText&&(identical(other.title, title) || other.title == title)&&(identical(other.body, body) || other.body == body));
}


@override
int get hashCode => Object.hash(runtimeType,title,body);

@override
String toString() {
  return 'VoiceNoteText(title: $title, body: $body)';
}


}

/// @nodoc
abstract mixin class $VoiceNoteTextCopyWith<$Res>  {
  factory $VoiceNoteTextCopyWith(VoiceNoteText value, $Res Function(VoiceNoteText) _then) = _$VoiceNoteTextCopyWithImpl;
@useResult
$Res call({
 String title, String body
});




}
/// @nodoc
class _$VoiceNoteTextCopyWithImpl<$Res>
    implements $VoiceNoteTextCopyWith<$Res> {
  _$VoiceNoteTextCopyWithImpl(this._self, this._then);

  final VoiceNoteText _self;
  final $Res Function(VoiceNoteText) _then;

/// Create a copy of VoiceNoteText
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? title = null,Object? body = null,}) {
  return _then(_self.copyWith(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [VoiceNoteText].
extension VoiceNoteTextPatterns on VoiceNoteText {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VoiceNoteText value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VoiceNoteText() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VoiceNoteText value)  $default,){
final _that = this;
switch (_that) {
case _VoiceNoteText():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VoiceNoteText value)?  $default,){
final _that = this;
switch (_that) {
case _VoiceNoteText() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String title,  String body)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VoiceNoteText() when $default != null:
return $default(_that.title,_that.body);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String title,  String body)  $default,) {final _that = this;
switch (_that) {
case _VoiceNoteText():
return $default(_that.title,_that.body);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String title,  String body)?  $default,) {final _that = this;
switch (_that) {
case _VoiceNoteText() when $default != null:
return $default(_that.title,_that.body);case _:
  return null;

}
}

}

/// @nodoc


class _VoiceNoteText implements VoiceNoteText {
  const _VoiceNoteText({required this.title, required this.body});
  

@override final  String title;
@override final  String body;

/// Create a copy of VoiceNoteText
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VoiceNoteTextCopyWith<_VoiceNoteText> get copyWith => __$VoiceNoteTextCopyWithImpl<_VoiceNoteText>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VoiceNoteText&&(identical(other.title, title) || other.title == title)&&(identical(other.body, body) || other.body == body));
}


@override
int get hashCode => Object.hash(runtimeType,title,body);

@override
String toString() {
  return 'VoiceNoteText(title: $title, body: $body)';
}


}

/// @nodoc
abstract mixin class _$VoiceNoteTextCopyWith<$Res> implements $VoiceNoteTextCopyWith<$Res> {
  factory _$VoiceNoteTextCopyWith(_VoiceNoteText value, $Res Function(_VoiceNoteText) _then) = __$VoiceNoteTextCopyWithImpl;
@override @useResult
$Res call({
 String title, String body
});




}
/// @nodoc
class __$VoiceNoteTextCopyWithImpl<$Res>
    implements _$VoiceNoteTextCopyWith<$Res> {
  __$VoiceNoteTextCopyWithImpl(this._self, this._then);

  final _VoiceNoteText _self;
  final $Res Function(_VoiceNoteText) _then;

/// Create a copy of VoiceNoteText
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? title = null,Object? body = null,}) {
  return _then(_VoiceNoteText(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
