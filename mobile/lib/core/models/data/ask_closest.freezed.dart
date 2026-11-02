// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_closest.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskClosest {

 String get noteId; String get title; int get start; int get end;
/// Create a copy of AskClosest
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskClosestCopyWith<AskClosest> get copyWith => _$AskClosestCopyWithImpl<AskClosest>(this as AskClosest, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskClosest&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.title, title) || other.title == title)&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,title,start,end);

@override
String toString() {
  return 'AskClosest(noteId: $noteId, title: $title, start: $start, end: $end)';
}


}

/// @nodoc
abstract mixin class $AskClosestCopyWith<$Res>  {
  factory $AskClosestCopyWith(AskClosest value, $Res Function(AskClosest) _then) = _$AskClosestCopyWithImpl;
@useResult
$Res call({
 String noteId, String title, int start, int end
});




}
/// @nodoc
class _$AskClosestCopyWithImpl<$Res>
    implements $AskClosestCopyWith<$Res> {
  _$AskClosestCopyWithImpl(this._self, this._then);

  final AskClosest _self;
  final $Res Function(AskClosest) _then;

/// Create a copy of AskClosest
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? title = null,Object? start = null,Object? end = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [AskClosest].
extension AskClosestPatterns on AskClosest {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskClosest value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskClosest() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskClosest value)  $default,){
final _that = this;
switch (_that) {
case _AskClosest():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskClosest value)?  $default,){
final _that = this;
switch (_that) {
case _AskClosest() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String title,  int start,  int end)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskClosest() when $default != null:
return $default(_that.noteId,_that.title,_that.start,_that.end);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String title,  int start,  int end)  $default,) {final _that = this;
switch (_that) {
case _AskClosest():
return $default(_that.noteId,_that.title,_that.start,_that.end);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String title,  int start,  int end)?  $default,) {final _that = this;
switch (_that) {
case _AskClosest() when $default != null:
return $default(_that.noteId,_that.title,_that.start,_that.end);case _:
  return null;

}
}

}

/// @nodoc


class _AskClosest implements AskClosest {
  const _AskClosest({required this.noteId, this.title = '', this.start = 0, this.end = 0});
  

@override final  String noteId;
@override@JsonKey() final  String title;
@override@JsonKey() final  int start;
@override@JsonKey() final  int end;

/// Create a copy of AskClosest
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskClosestCopyWith<_AskClosest> get copyWith => __$AskClosestCopyWithImpl<_AskClosest>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskClosest&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.title, title) || other.title == title)&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,title,start,end);

@override
String toString() {
  return 'AskClosest(noteId: $noteId, title: $title, start: $start, end: $end)';
}


}

/// @nodoc
abstract mixin class _$AskClosestCopyWith<$Res> implements $AskClosestCopyWith<$Res> {
  factory _$AskClosestCopyWith(_AskClosest value, $Res Function(_AskClosest) _then) = __$AskClosestCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String title, int start, int end
});




}
/// @nodoc
class __$AskClosestCopyWithImpl<$Res>
    implements _$AskClosestCopyWith<$Res> {
  __$AskClosestCopyWithImpl(this._self, this._then);

  final _AskClosest _self;
  final $Res Function(_AskClosest) _then;

/// Create a copy of AskClosest
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? title = null,Object? start = null,Object? end = null,}) {
  return _then(_AskClosest(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
