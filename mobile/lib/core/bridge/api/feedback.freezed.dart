// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'feedback.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FeedbackSent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackSent()';
}


}

/// @nodoc
class $FeedbackSentCopyWith<$Res>  {
$FeedbackSentCopyWith(FeedbackSent _, $Res Function(FeedbackSent) __);
}


/// Adds pattern-matching-related methods to [FeedbackSent].
extension FeedbackSentPatterns on FeedbackSent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( FeedbackSent_Sent value)?  sent,TResult Function( FeedbackSent_Queued value)?  queued,TResult Function( FeedbackSent_Paused value)?  paused,TResult Function( FeedbackSent_RateLimited value)?  rateLimited,TResult Function( FeedbackSent_Rejected value)?  rejected,TResult Function( FeedbackSent_Invalid value)?  invalid,required TResult orElse(),}){
final _that = this;
switch (_that) {
case FeedbackSent_Sent() when sent != null:
return sent(_that);case FeedbackSent_Queued() when queued != null:
return queued(_that);case FeedbackSent_Paused() when paused != null:
return paused(_that);case FeedbackSent_RateLimited() when rateLimited != null:
return rateLimited(_that);case FeedbackSent_Rejected() when rejected != null:
return rejected(_that);case FeedbackSent_Invalid() when invalid != null:
return invalid(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( FeedbackSent_Sent value)  sent,required TResult Function( FeedbackSent_Queued value)  queued,required TResult Function( FeedbackSent_Paused value)  paused,required TResult Function( FeedbackSent_RateLimited value)  rateLimited,required TResult Function( FeedbackSent_Rejected value)  rejected,required TResult Function( FeedbackSent_Invalid value)  invalid,}){
final _that = this;
switch (_that) {
case FeedbackSent_Sent():
return sent(_that);case FeedbackSent_Queued():
return queued(_that);case FeedbackSent_Paused():
return paused(_that);case FeedbackSent_RateLimited():
return rateLimited(_that);case FeedbackSent_Rejected():
return rejected(_that);case FeedbackSent_Invalid():
return invalid(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( FeedbackSent_Sent value)?  sent,TResult? Function( FeedbackSent_Queued value)?  queued,TResult? Function( FeedbackSent_Paused value)?  paused,TResult? Function( FeedbackSent_RateLimited value)?  rateLimited,TResult? Function( FeedbackSent_Rejected value)?  rejected,TResult? Function( FeedbackSent_Invalid value)?  invalid,}){
final _that = this;
switch (_that) {
case FeedbackSent_Sent() when sent != null:
return sent(_that);case FeedbackSent_Queued() when queued != null:
return queued(_that);case FeedbackSent_Paused() when paused != null:
return paused(_that);case FeedbackSent_RateLimited() when rateLimited != null:
return rateLimited(_that);case FeedbackSent_Rejected() when rejected != null:
return rejected(_that);case FeedbackSent_Invalid() when invalid != null:
return invalid(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  sent,TResult Function()?  queued,TResult Function()?  paused,TResult Function()?  rateLimited,TResult Function( String field)?  rejected,TResult Function( FeedbackProblem problem)?  invalid,required TResult orElse(),}) {final _that = this;
switch (_that) {
case FeedbackSent_Sent() when sent != null:
return sent();case FeedbackSent_Queued() when queued != null:
return queued();case FeedbackSent_Paused() when paused != null:
return paused();case FeedbackSent_RateLimited() when rateLimited != null:
return rateLimited();case FeedbackSent_Rejected() when rejected != null:
return rejected(_that.field);case FeedbackSent_Invalid() when invalid != null:
return invalid(_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  sent,required TResult Function()  queued,required TResult Function()  paused,required TResult Function()  rateLimited,required TResult Function( String field)  rejected,required TResult Function( FeedbackProblem problem)  invalid,}) {final _that = this;
switch (_that) {
case FeedbackSent_Sent():
return sent();case FeedbackSent_Queued():
return queued();case FeedbackSent_Paused():
return paused();case FeedbackSent_RateLimited():
return rateLimited();case FeedbackSent_Rejected():
return rejected(_that.field);case FeedbackSent_Invalid():
return invalid(_that.problem);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  sent,TResult? Function()?  queued,TResult? Function()?  paused,TResult? Function()?  rateLimited,TResult? Function( String field)?  rejected,TResult? Function( FeedbackProblem problem)?  invalid,}) {final _that = this;
switch (_that) {
case FeedbackSent_Sent() when sent != null:
return sent();case FeedbackSent_Queued() when queued != null:
return queued();case FeedbackSent_Paused() when paused != null:
return paused();case FeedbackSent_RateLimited() when rateLimited != null:
return rateLimited();case FeedbackSent_Rejected() when rejected != null:
return rejected(_that.field);case FeedbackSent_Invalid() when invalid != null:
return invalid(_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class FeedbackSent_Sent extends FeedbackSent {
  const FeedbackSent_Sent(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent_Sent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackSent.sent()';
}


}




/// @nodoc


class FeedbackSent_Queued extends FeedbackSent {
  const FeedbackSent_Queued(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent_Queued);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackSent.queued()';
}


}




/// @nodoc


class FeedbackSent_Paused extends FeedbackSent {
  const FeedbackSent_Paused(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent_Paused);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackSent.paused()';
}


}




/// @nodoc


class FeedbackSent_RateLimited extends FeedbackSent {
  const FeedbackSent_RateLimited(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent_RateLimited);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackSent.rateLimited()';
}


}




/// @nodoc


class FeedbackSent_Rejected extends FeedbackSent {
  const FeedbackSent_Rejected({required this.field}): super._();
  

 final  String field;

/// Create a copy of FeedbackSent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackSent_RejectedCopyWith<FeedbackSent_Rejected> get copyWith => _$FeedbackSent_RejectedCopyWithImpl<FeedbackSent_Rejected>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent_Rejected&&(identical(other.field, field) || other.field == field));
}


@override
int get hashCode => Object.hash(runtimeType,field);

@override
String toString() {
  return 'FeedbackSent.rejected(field: $field)';
}


}

/// @nodoc
abstract mixin class $FeedbackSent_RejectedCopyWith<$Res> implements $FeedbackSentCopyWith<$Res> {
  factory $FeedbackSent_RejectedCopyWith(FeedbackSent_Rejected value, $Res Function(FeedbackSent_Rejected) _then) = _$FeedbackSent_RejectedCopyWithImpl;
@useResult
$Res call({
 String field
});




}
/// @nodoc
class _$FeedbackSent_RejectedCopyWithImpl<$Res>
    implements $FeedbackSent_RejectedCopyWith<$Res> {
  _$FeedbackSent_RejectedCopyWithImpl(this._self, this._then);

  final FeedbackSent_Rejected _self;
  final $Res Function(FeedbackSent_Rejected) _then;

/// Create a copy of FeedbackSent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field = null,}) {
  return _then(FeedbackSent_Rejected(
field: null == field ? _self.field : field // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class FeedbackSent_Invalid extends FeedbackSent {
  const FeedbackSent_Invalid({required this.problem}): super._();
  

 final  FeedbackProblem problem;

/// Create a copy of FeedbackSent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackSent_InvalidCopyWith<FeedbackSent_Invalid> get copyWith => _$FeedbackSent_InvalidCopyWithImpl<FeedbackSent_Invalid>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackSent_Invalid&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,problem);

@override
String toString() {
  return 'FeedbackSent.invalid(problem: $problem)';
}


}

/// @nodoc
abstract mixin class $FeedbackSent_InvalidCopyWith<$Res> implements $FeedbackSentCopyWith<$Res> {
  factory $FeedbackSent_InvalidCopyWith(FeedbackSent_Invalid value, $Res Function(FeedbackSent_Invalid) _then) = _$FeedbackSent_InvalidCopyWithImpl;
@useResult
$Res call({
 FeedbackProblem problem
});




}
/// @nodoc
class _$FeedbackSent_InvalidCopyWithImpl<$Res>
    implements $FeedbackSent_InvalidCopyWith<$Res> {
  _$FeedbackSent_InvalidCopyWithImpl(this._self, this._then);

  final FeedbackSent_Invalid _self;
  final $Res Function(FeedbackSent_Invalid) _then;

/// Create a copy of FeedbackSent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? problem = null,}) {
  return _then(FeedbackSent_Invalid(
problem: null == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FeedbackProblem,
  ));
}


}

// dart format on
