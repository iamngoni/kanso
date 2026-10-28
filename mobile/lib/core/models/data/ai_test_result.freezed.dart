// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai_test_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AiTestResult {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiTestResult);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AiTestResult()';
}


}

/// @nodoc
class $AiTestResultCopyWith<$Res>  {
$AiTestResultCopyWith(AiTestResult _, $Res Function(AiTestResult) __);
}


/// Adds pattern-matching-related methods to [AiTestResult].
extension AiTestResultPatterns on AiTestResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AiTestOk value)?  ok,TResult Function( AiTestFailed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AiTestOk() when ok != null:
return ok(_that);case AiTestFailed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AiTestOk value)  ok,required TResult Function( AiTestFailed value)  failed,}){
final _that = this;
switch (_that) {
case AiTestOk():
return ok(_that);case AiTestFailed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AiTestOk value)?  ok,TResult? Function( AiTestFailed value)?  failed,}){
final _that = this;
switch (_that) {
case AiTestOk() when ok != null:
return ok(_that);case AiTestFailed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String said)?  ok,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AiTestOk() when ok != null:
return ok(_that.said);case AiTestFailed() when failed != null:
return failed(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String said)  ok,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case AiTestOk():
return ok(_that.said);case AiTestFailed():
return failed(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String said)?  ok,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case AiTestOk() when ok != null:
return ok(_that.said);case AiTestFailed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class AiTestOk implements AiTestResult {
  const AiTestOk({required this.said});
  

 final  String said;

/// Create a copy of AiTestResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiTestOkCopyWith<AiTestOk> get copyWith => _$AiTestOkCopyWithImpl<AiTestOk>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiTestOk&&(identical(other.said, said) || other.said == said));
}


@override
int get hashCode => Object.hash(runtimeType,said);

@override
String toString() {
  return 'AiTestResult.ok(said: $said)';
}


}

/// @nodoc
abstract mixin class $AiTestOkCopyWith<$Res> implements $AiTestResultCopyWith<$Res> {
  factory $AiTestOkCopyWith(AiTestOk value, $Res Function(AiTestOk) _then) = _$AiTestOkCopyWithImpl;
@useResult
$Res call({
 String said
});




}
/// @nodoc
class _$AiTestOkCopyWithImpl<$Res>
    implements $AiTestOkCopyWith<$Res> {
  _$AiTestOkCopyWithImpl(this._self, this._then);

  final AiTestOk _self;
  final $Res Function(AiTestOk) _then;

/// Create a copy of AiTestResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? said = null,}) {
  return _then(AiTestOk(
said: null == said ? _self.said : said // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AiTestFailed implements AiTestResult {
  const AiTestFailed({required this.message});
  

 final  String message;

/// Create a copy of AiTestResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AiTestFailedCopyWith<AiTestFailed> get copyWith => _$AiTestFailedCopyWithImpl<AiTestFailed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AiTestFailed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AiTestResult.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $AiTestFailedCopyWith<$Res> implements $AiTestResultCopyWith<$Res> {
  factory $AiTestFailedCopyWith(AiTestFailed value, $Res Function(AiTestFailed) _then) = _$AiTestFailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AiTestFailedCopyWithImpl<$Res>
    implements $AiTestFailedCopyWith<$Res> {
  _$AiTestFailedCopyWithImpl(this._self, this._then);

  final AiTestFailed _self;
  final $Res Function(AiTestFailed) _then;

/// Create a copy of AiTestResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AiTestFailed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
