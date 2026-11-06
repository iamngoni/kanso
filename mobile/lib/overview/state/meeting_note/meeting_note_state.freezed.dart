// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'meeting_note_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MeetingNoteState {

 bool get working; String? get createdNoteId; String? get error;
/// Create a copy of MeetingNoteState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MeetingNoteStateCopyWith<MeetingNoteState> get copyWith => _$MeetingNoteStateCopyWithImpl<MeetingNoteState>(this as MeetingNoteState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MeetingNoteState&&(identical(other.working, working) || other.working == working)&&(identical(other.createdNoteId, createdNoteId) || other.createdNoteId == createdNoteId)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,working,createdNoteId,error);

@override
String toString() {
  return 'MeetingNoteState(working: $working, createdNoteId: $createdNoteId, error: $error)';
}


}

/// @nodoc
abstract mixin class $MeetingNoteStateCopyWith<$Res>  {
  factory $MeetingNoteStateCopyWith(MeetingNoteState value, $Res Function(MeetingNoteState) _then) = _$MeetingNoteStateCopyWithImpl;
@useResult
$Res call({
 bool working, String? createdNoteId, String? error
});




}
/// @nodoc
class _$MeetingNoteStateCopyWithImpl<$Res>
    implements $MeetingNoteStateCopyWith<$Res> {
  _$MeetingNoteStateCopyWithImpl(this._self, this._then);

  final MeetingNoteState _self;
  final $Res Function(MeetingNoteState) _then;

/// Create a copy of MeetingNoteState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? working = null,Object? createdNoteId = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,createdNoteId: freezed == createdNoteId ? _self.createdNoteId : createdNoteId // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [MeetingNoteState].
extension MeetingNoteStatePatterns on MeetingNoteState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MeetingNoteState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MeetingNoteState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MeetingNoteState value)  $default,){
final _that = this;
switch (_that) {
case _MeetingNoteState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MeetingNoteState value)?  $default,){
final _that = this;
switch (_that) {
case _MeetingNoteState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool working,  String? createdNoteId,  String? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MeetingNoteState() when $default != null:
return $default(_that.working,_that.createdNoteId,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool working,  String? createdNoteId,  String? error)  $default,) {final _that = this;
switch (_that) {
case _MeetingNoteState():
return $default(_that.working,_that.createdNoteId,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool working,  String? createdNoteId,  String? error)?  $default,) {final _that = this;
switch (_that) {
case _MeetingNoteState() when $default != null:
return $default(_that.working,_that.createdNoteId,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _MeetingNoteState implements MeetingNoteState {
  const _MeetingNoteState({this.working = false, this.createdNoteId, this.error});
  

@override@JsonKey() final  bool working;
@override final  String? createdNoteId;
@override final  String? error;

/// Create a copy of MeetingNoteState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MeetingNoteStateCopyWith<_MeetingNoteState> get copyWith => __$MeetingNoteStateCopyWithImpl<_MeetingNoteState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MeetingNoteState&&(identical(other.working, working) || other.working == working)&&(identical(other.createdNoteId, createdNoteId) || other.createdNoteId == createdNoteId)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,working,createdNoteId,error);

@override
String toString() {
  return 'MeetingNoteState(working: $working, createdNoteId: $createdNoteId, error: $error)';
}


}

/// @nodoc
abstract mixin class _$MeetingNoteStateCopyWith<$Res> implements $MeetingNoteStateCopyWith<$Res> {
  factory _$MeetingNoteStateCopyWith(_MeetingNoteState value, $Res Function(_MeetingNoteState) _then) = __$MeetingNoteStateCopyWithImpl;
@override @useResult
$Res call({
 bool working, String? createdNoteId, String? error
});




}
/// @nodoc
class __$MeetingNoteStateCopyWithImpl<$Res>
    implements _$MeetingNoteStateCopyWith<$Res> {
  __$MeetingNoteStateCopyWithImpl(this._self, this._then);

  final _MeetingNoteState _self;
  final $Res Function(_MeetingNoteState) _then;

/// Create a copy of MeetingNoteState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? working = null,Object? createdNoteId = freezed,Object? error = freezed,}) {
  return _then(_MeetingNoteState(
working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,createdNoteId: freezed == createdNoteId ? _self.createdNoteId : createdNoteId // ignore: cast_nullable_to_non_nullable
as String?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
