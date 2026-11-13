// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'capture_task_cubit.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CaptureTaskState {

/// Whether the saved choices have been read.
 bool get loaded;/// The mode the capture sheet is in; the last one chosen.
 CaptureMode get mode;/// Where a captured task is written.
 CaptureTarget get target;/// How the date and repeat at the end of the typed text read, or null.
 String? get hint;/// Where the task being saved, or the last one, has got to.
 CaptureTaskStatus get status;
/// Create a copy of CaptureTaskState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CaptureTaskStateCopyWith<CaptureTaskState> get copyWith => _$CaptureTaskStateCopyWithImpl<CaptureTaskState>(this as CaptureTaskState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CaptureTaskState&&(identical(other.loaded, loaded) || other.loaded == loaded)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.target, target) || other.target == target)&&(identical(other.hint, hint) || other.hint == hint)&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,loaded,mode,target,hint,status);

@override
String toString() {
  return 'CaptureTaskState(loaded: $loaded, mode: $mode, target: $target, hint: $hint, status: $status)';
}


}

/// @nodoc
abstract mixin class $CaptureTaskStateCopyWith<$Res>  {
  factory $CaptureTaskStateCopyWith(CaptureTaskState value, $Res Function(CaptureTaskState) _then) = _$CaptureTaskStateCopyWithImpl;
@useResult
$Res call({
 bool loaded, CaptureMode mode, CaptureTarget target, String? hint, CaptureTaskStatus status
});




}
/// @nodoc
class _$CaptureTaskStateCopyWithImpl<$Res>
    implements $CaptureTaskStateCopyWith<$Res> {
  _$CaptureTaskStateCopyWithImpl(this._self, this._then);

  final CaptureTaskState _self;
  final $Res Function(CaptureTaskState) _then;

/// Create a copy of CaptureTaskState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? loaded = null,Object? mode = null,Object? target = null,Object? hint = freezed,Object? status = null,}) {
  return _then(_self.copyWith(
loaded: null == loaded ? _self.loaded : loaded // ignore: cast_nullable_to_non_nullable
as bool,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as CaptureMode,target: null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as CaptureTarget,hint: freezed == hint ? _self.hint : hint // ignore: cast_nullable_to_non_nullable
as String?,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CaptureTaskStatus,
  ));
}

}


/// Adds pattern-matching-related methods to [CaptureTaskState].
extension CaptureTaskStatePatterns on CaptureTaskState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CaptureTaskState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CaptureTaskState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CaptureTaskState value)  $default,){
final _that = this;
switch (_that) {
case _CaptureTaskState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CaptureTaskState value)?  $default,){
final _that = this;
switch (_that) {
case _CaptureTaskState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool loaded,  CaptureMode mode,  CaptureTarget target,  String? hint,  CaptureTaskStatus status)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CaptureTaskState() when $default != null:
return $default(_that.loaded,_that.mode,_that.target,_that.hint,_that.status);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool loaded,  CaptureMode mode,  CaptureTarget target,  String? hint,  CaptureTaskStatus status)  $default,) {final _that = this;
switch (_that) {
case _CaptureTaskState():
return $default(_that.loaded,_that.mode,_that.target,_that.hint,_that.status);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool loaded,  CaptureMode mode,  CaptureTarget target,  String? hint,  CaptureTaskStatus status)?  $default,) {final _that = this;
switch (_that) {
case _CaptureTaskState() when $default != null:
return $default(_that.loaded,_that.mode,_that.target,_that.hint,_that.status);case _:
  return null;

}
}

}

/// @nodoc


class _CaptureTaskState implements CaptureTaskState {
  const _CaptureTaskState({this.loaded = false, this.mode = CaptureMode.note, this.target = CaptureTarget.dailyNote, this.hint, this.status = CaptureTaskStatus.idle});
  

/// Whether the saved choices have been read.
@override@JsonKey() final  bool loaded;
/// The mode the capture sheet is in; the last one chosen.
@override@JsonKey() final  CaptureMode mode;
/// Where a captured task is written.
@override@JsonKey() final  CaptureTarget target;
/// How the date and repeat at the end of the typed text read, or null.
@override final  String? hint;
/// Where the task being saved, or the last one, has got to.
@override@JsonKey() final  CaptureTaskStatus status;

/// Create a copy of CaptureTaskState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CaptureTaskStateCopyWith<_CaptureTaskState> get copyWith => __$CaptureTaskStateCopyWithImpl<_CaptureTaskState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CaptureTaskState&&(identical(other.loaded, loaded) || other.loaded == loaded)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.target, target) || other.target == target)&&(identical(other.hint, hint) || other.hint == hint)&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,loaded,mode,target,hint,status);

@override
String toString() {
  return 'CaptureTaskState(loaded: $loaded, mode: $mode, target: $target, hint: $hint, status: $status)';
}


}

/// @nodoc
abstract mixin class _$CaptureTaskStateCopyWith<$Res> implements $CaptureTaskStateCopyWith<$Res> {
  factory _$CaptureTaskStateCopyWith(_CaptureTaskState value, $Res Function(_CaptureTaskState) _then) = __$CaptureTaskStateCopyWithImpl;
@override @useResult
$Res call({
 bool loaded, CaptureMode mode, CaptureTarget target, String? hint, CaptureTaskStatus status
});




}
/// @nodoc
class __$CaptureTaskStateCopyWithImpl<$Res>
    implements _$CaptureTaskStateCopyWith<$Res> {
  __$CaptureTaskStateCopyWithImpl(this._self, this._then);

  final _CaptureTaskState _self;
  final $Res Function(_CaptureTaskState) _then;

/// Create a copy of CaptureTaskState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? loaded = null,Object? mode = null,Object? target = null,Object? hint = freezed,Object? status = null,}) {
  return _then(_CaptureTaskState(
loaded: null == loaded ? _self.loaded : loaded // ignore: cast_nullable_to_non_nullable
as bool,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as CaptureMode,target: null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as CaptureTarget,hint: freezed == hint ? _self.hint : hint // ignore: cast_nullable_to_non_nullable
as String?,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CaptureTaskStatus,
  ));
}


}

// dart format on
