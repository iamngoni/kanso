// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_window_info.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteWindowInfo {

 String get noteId; String get title;
/// Create a copy of NoteWindowInfo
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteWindowInfoCopyWith<NoteWindowInfo> get copyWith => _$NoteWindowInfoCopyWithImpl<NoteWindowInfo>(this as NoteWindowInfo, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowInfo&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,title);

@override
String toString() {
  return 'NoteWindowInfo(noteId: $noteId, title: $title)';
}


}

/// @nodoc
abstract mixin class $NoteWindowInfoCopyWith<$Res>  {
  factory $NoteWindowInfoCopyWith(NoteWindowInfo value, $Res Function(NoteWindowInfo) _then) = _$NoteWindowInfoCopyWithImpl;
@useResult
$Res call({
 String noteId, String title
});




}
/// @nodoc
class _$NoteWindowInfoCopyWithImpl<$Res>
    implements $NoteWindowInfoCopyWith<$Res> {
  _$NoteWindowInfoCopyWithImpl(this._self, this._then);

  final NoteWindowInfo _self;
  final $Res Function(NoteWindowInfo) _then;

/// Create a copy of NoteWindowInfo
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? title = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteWindowInfo].
extension NoteWindowInfoPatterns on NoteWindowInfo {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteWindowInfo value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteWindowInfo() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteWindowInfo value)  $default,){
final _that = this;
switch (_that) {
case _NoteWindowInfo():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteWindowInfo value)?  $default,){
final _that = this;
switch (_that) {
case _NoteWindowInfo() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String title)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteWindowInfo() when $default != null:
return $default(_that.noteId,_that.title);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String title)  $default,) {final _that = this;
switch (_that) {
case _NoteWindowInfo():
return $default(_that.noteId,_that.title);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String title)?  $default,) {final _that = this;
switch (_that) {
case _NoteWindowInfo() when $default != null:
return $default(_that.noteId,_that.title);case _:
  return null;

}
}

}

/// @nodoc


class _NoteWindowInfo implements NoteWindowInfo {
  const _NoteWindowInfo({required this.noteId, this.title = ''});
  

@override final  String noteId;
@override@JsonKey() final  String title;

/// Create a copy of NoteWindowInfo
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteWindowInfoCopyWith<_NoteWindowInfo> get copyWith => __$NoteWindowInfoCopyWithImpl<_NoteWindowInfo>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteWindowInfo&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,title);

@override
String toString() {
  return 'NoteWindowInfo(noteId: $noteId, title: $title)';
}


}

/// @nodoc
abstract mixin class _$NoteWindowInfoCopyWith<$Res> implements $NoteWindowInfoCopyWith<$Res> {
  factory _$NoteWindowInfoCopyWith(_NoteWindowInfo value, $Res Function(_NoteWindowInfo) _then) = __$NoteWindowInfoCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String title
});




}
/// @nodoc
class __$NoteWindowInfoCopyWithImpl<$Res>
    implements _$NoteWindowInfoCopyWith<$Res> {
  __$NoteWindowInfoCopyWithImpl(this._self, this._then);

  final _NoteWindowInfo _self;
  final $Res Function(_NoteWindowInfo) _then;

/// Create a copy of NoteWindowInfo
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? title = null,}) {
  return _then(_NoteWindowInfo(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
