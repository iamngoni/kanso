// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'reminder_response.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ReminderResponse {

 String get key; String get noteId; int get line; ReminderAnswer get action;
/// Create a copy of ReminderResponse
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ReminderResponseCopyWith<ReminderResponse> get copyWith => _$ReminderResponseCopyWithImpl<ReminderResponse>(this as ReminderResponse, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ReminderResponse&&(identical(other.key, key) || other.key == key)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.line, line) || other.line == line)&&(identical(other.action, action) || other.action == action));
}


@override
int get hashCode => Object.hash(runtimeType,key,noteId,line,action);

@override
String toString() {
  return 'ReminderResponse(key: $key, noteId: $noteId, line: $line, action: $action)';
}


}

/// @nodoc
abstract mixin class $ReminderResponseCopyWith<$Res>  {
  factory $ReminderResponseCopyWith(ReminderResponse value, $Res Function(ReminderResponse) _then) = _$ReminderResponseCopyWithImpl;
@useResult
$Res call({
 String key, String noteId, int line, ReminderAnswer action
});




}
/// @nodoc
class _$ReminderResponseCopyWithImpl<$Res>
    implements $ReminderResponseCopyWith<$Res> {
  _$ReminderResponseCopyWithImpl(this._self, this._then);

  final ReminderResponse _self;
  final $Res Function(ReminderResponse) _then;

/// Create a copy of ReminderResponse
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? key = null,Object? noteId = null,Object? line = null,Object? action = null,}) {
  return _then(_self.copyWith(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as int,action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as ReminderAnswer,
  ));
}

}


/// Adds pattern-matching-related methods to [ReminderResponse].
extension ReminderResponsePatterns on ReminderResponse {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ReminderResponse value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ReminderResponse() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ReminderResponse value)  $default,){
final _that = this;
switch (_that) {
case _ReminderResponse():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ReminderResponse value)?  $default,){
final _that = this;
switch (_that) {
case _ReminderResponse() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String key,  String noteId,  int line,  ReminderAnswer action)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ReminderResponse() when $default != null:
return $default(_that.key,_that.noteId,_that.line,_that.action);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String key,  String noteId,  int line,  ReminderAnswer action)  $default,) {final _that = this;
switch (_that) {
case _ReminderResponse():
return $default(_that.key,_that.noteId,_that.line,_that.action);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String key,  String noteId,  int line,  ReminderAnswer action)?  $default,) {final _that = this;
switch (_that) {
case _ReminderResponse() when $default != null:
return $default(_that.key,_that.noteId,_that.line,_that.action);case _:
  return null;

}
}

}

/// @nodoc


class _ReminderResponse implements ReminderResponse {
  const _ReminderResponse({required this.key, required this.noteId, required this.line, required this.action});
  

@override final  String key;
@override final  String noteId;
@override final  int line;
@override final  ReminderAnswer action;

/// Create a copy of ReminderResponse
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ReminderResponseCopyWith<_ReminderResponse> get copyWith => __$ReminderResponseCopyWithImpl<_ReminderResponse>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ReminderResponse&&(identical(other.key, key) || other.key == key)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.line, line) || other.line == line)&&(identical(other.action, action) || other.action == action));
}


@override
int get hashCode => Object.hash(runtimeType,key,noteId,line,action);

@override
String toString() {
  return 'ReminderResponse(key: $key, noteId: $noteId, line: $line, action: $action)';
}


}

/// @nodoc
abstract mixin class _$ReminderResponseCopyWith<$Res> implements $ReminderResponseCopyWith<$Res> {
  factory _$ReminderResponseCopyWith(_ReminderResponse value, $Res Function(_ReminderResponse) _then) = __$ReminderResponseCopyWithImpl;
@override @useResult
$Res call({
 String key, String noteId, int line, ReminderAnswer action
});




}
/// @nodoc
class __$ReminderResponseCopyWithImpl<$Res>
    implements _$ReminderResponseCopyWith<$Res> {
  __$ReminderResponseCopyWithImpl(this._self, this._then);

  final _ReminderResponse _self;
  final $Res Function(_ReminderResponse) _then;

/// Create a copy of ReminderResponse
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? key = null,Object? noteId = null,Object? line = null,Object? action = null,}) {
  return _then(_ReminderResponse(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as int,action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as ReminderAnswer,
  ));
}


}

// dart format on
