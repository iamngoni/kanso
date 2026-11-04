// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'feedback_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FeedbackSendResult {

 FeedbackOutcome get outcome;/// The JSON path the server refused, for [FeedbackOutcome.rejected].
 String? get field;/// Why it can't be sent, for [FeedbackOutcome.invalid].
 FeedbackProblem? get problem;
/// Create a copy of FeedbackSendResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackSendResultCopyWith<FeedbackSendResult> get copyWith => _$FeedbackSendResultCopyWithImpl<FeedbackSendResult>(this as FeedbackSendResult, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSendResult&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.field, field) || other.field == field)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,outcome,field,problem);

@override
String toString() {
  return 'FeedbackSendResult(outcome: $outcome, field: $field, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $FeedbackSendResultCopyWith<$Res>  {
  factory $FeedbackSendResultCopyWith(FeedbackSendResult value, $Res Function(FeedbackSendResult) _then) = _$FeedbackSendResultCopyWithImpl;
@useResult
$Res call({
 FeedbackOutcome outcome, String? field, FeedbackProblem? problem
});




}
/// @nodoc
class _$FeedbackSendResultCopyWithImpl<$Res>
    implements $FeedbackSendResultCopyWith<$Res> {
  _$FeedbackSendResultCopyWithImpl(this._self, this._then);

  final FeedbackSendResult _self;
  final $Res Function(FeedbackSendResult) _then;

/// Create a copy of FeedbackSendResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? outcome = null,Object? field = freezed,Object? problem = freezed,}) {
  return _then(_self.copyWith(
outcome: null == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as FeedbackOutcome,field: freezed == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FeedbackProblem?,
  ));
}

}


/// Adds pattern-matching-related methods to [FeedbackSendResult].
extension FeedbackSendResultPatterns on FeedbackSendResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FeedbackSendResult value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FeedbackSendResult() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FeedbackSendResult value)  $default,){
final _that = this;
switch (_that) {
case _FeedbackSendResult():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FeedbackSendResult value)?  $default,){
final _that = this;
switch (_that) {
case _FeedbackSendResult() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( FeedbackOutcome outcome,  String? field,  FeedbackProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FeedbackSendResult() when $default != null:
return $default(_that.outcome,_that.field,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( FeedbackOutcome outcome,  String? field,  FeedbackProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _FeedbackSendResult():
return $default(_that.outcome,_that.field,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( FeedbackOutcome outcome,  String? field,  FeedbackProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _FeedbackSendResult() when $default != null:
return $default(_that.outcome,_that.field,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _FeedbackSendResult implements FeedbackSendResult {
  const _FeedbackSendResult({required this.outcome, this.field, this.problem});
  

@override final  FeedbackOutcome outcome;
/// The JSON path the server refused, for [FeedbackOutcome.rejected].
@override final  String? field;
/// Why it can't be sent, for [FeedbackOutcome.invalid].
@override final  FeedbackProblem? problem;

/// Create a copy of FeedbackSendResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FeedbackSendResultCopyWith<_FeedbackSendResult> get copyWith => __$FeedbackSendResultCopyWithImpl<_FeedbackSendResult>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FeedbackSendResult&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.field, field) || other.field == field)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,outcome,field,problem);

@override
String toString() {
  return 'FeedbackSendResult(outcome: $outcome, field: $field, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$FeedbackSendResultCopyWith<$Res> implements $FeedbackSendResultCopyWith<$Res> {
  factory _$FeedbackSendResultCopyWith(_FeedbackSendResult value, $Res Function(_FeedbackSendResult) _then) = __$FeedbackSendResultCopyWithImpl;
@override @useResult
$Res call({
 FeedbackOutcome outcome, String? field, FeedbackProblem? problem
});




}
/// @nodoc
class __$FeedbackSendResultCopyWithImpl<$Res>
    implements _$FeedbackSendResultCopyWith<$Res> {
  __$FeedbackSendResultCopyWithImpl(this._self, this._then);

  final _FeedbackSendResult _self;
  final $Res Function(_FeedbackSendResult) _then;

/// Create a copy of FeedbackSendResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? outcome = null,Object? field = freezed,Object? problem = freezed,}) {
  return _then(_FeedbackSendResult(
outcome: null == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as FeedbackOutcome,field: freezed == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FeedbackProblem?,
  ));
}


}

/// @nodoc
mixin _$FeedbackPayload {

 String get json; FeedbackProblem? get problem;
/// Create a copy of FeedbackPayload
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackPayloadCopyWith<FeedbackPayload> get copyWith => _$FeedbackPayloadCopyWithImpl<FeedbackPayload>(this as FeedbackPayload, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackPayload&&(identical(other.json, json) || other.json == json)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,json,problem);

@override
String toString() {
  return 'FeedbackPayload(json: $json, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $FeedbackPayloadCopyWith<$Res>  {
  factory $FeedbackPayloadCopyWith(FeedbackPayload value, $Res Function(FeedbackPayload) _then) = _$FeedbackPayloadCopyWithImpl;
@useResult
$Res call({
 String json, FeedbackProblem? problem
});




}
/// @nodoc
class _$FeedbackPayloadCopyWithImpl<$Res>
    implements $FeedbackPayloadCopyWith<$Res> {
  _$FeedbackPayloadCopyWithImpl(this._self, this._then);

  final FeedbackPayload _self;
  final $Res Function(FeedbackPayload) _then;

/// Create a copy of FeedbackPayload
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? json = null,Object? problem = freezed,}) {
  return _then(_self.copyWith(
json: null == json ? _self.json : json // ignore: cast_nullable_to_non_nullable
as String,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FeedbackProblem?,
  ));
}

}


/// Adds pattern-matching-related methods to [FeedbackPayload].
extension FeedbackPayloadPatterns on FeedbackPayload {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FeedbackPayload value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FeedbackPayload() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FeedbackPayload value)  $default,){
final _that = this;
switch (_that) {
case _FeedbackPayload():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FeedbackPayload value)?  $default,){
final _that = this;
switch (_that) {
case _FeedbackPayload() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String json,  FeedbackProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FeedbackPayload() when $default != null:
return $default(_that.json,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String json,  FeedbackProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _FeedbackPayload():
return $default(_that.json,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String json,  FeedbackProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _FeedbackPayload() when $default != null:
return $default(_that.json,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _FeedbackPayload implements FeedbackPayload {
  const _FeedbackPayload({this.json = '', this.problem});
  

@override@JsonKey() final  String json;
@override final  FeedbackProblem? problem;

/// Create a copy of FeedbackPayload
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FeedbackPayloadCopyWith<_FeedbackPayload> get copyWith => __$FeedbackPayloadCopyWithImpl<_FeedbackPayload>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FeedbackPayload&&(identical(other.json, json) || other.json == json)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,json,problem);

@override
String toString() {
  return 'FeedbackPayload(json: $json, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$FeedbackPayloadCopyWith<$Res> implements $FeedbackPayloadCopyWith<$Res> {
  factory _$FeedbackPayloadCopyWith(_FeedbackPayload value, $Res Function(_FeedbackPayload) _then) = __$FeedbackPayloadCopyWithImpl;
@override @useResult
$Res call({
 String json, FeedbackProblem? problem
});




}
/// @nodoc
class __$FeedbackPayloadCopyWithImpl<$Res>
    implements _$FeedbackPayloadCopyWith<$Res> {
  __$FeedbackPayloadCopyWithImpl(this._self, this._then);

  final _FeedbackPayload _self;
  final $Res Function(_FeedbackPayload) _then;

/// Create a copy of FeedbackPayload
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? json = null,Object? problem = freezed,}) {
  return _then(_FeedbackPayload(
json: null == json ? _self.json : json // ignore: cast_nullable_to_non_nullable
as String,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FeedbackProblem?,
  ));
}


}

// dart format on
