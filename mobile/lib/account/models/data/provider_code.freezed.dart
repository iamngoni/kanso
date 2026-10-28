// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'provider_code.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ProviderCode {

 String get userCode; String get verificationUri;
/// Create a copy of ProviderCode
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderCodeCopyWith<ProviderCode> get copyWith => _$ProviderCodeCopyWithImpl<ProviderCode>(this as ProviderCode, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderCode&&(identical(other.userCode, userCode) || other.userCode == userCode)&&(identical(other.verificationUri, verificationUri) || other.verificationUri == verificationUri));
}


@override
int get hashCode => Object.hash(runtimeType,userCode,verificationUri);

@override
String toString() {
  return 'ProviderCode(userCode: $userCode, verificationUri: $verificationUri)';
}


}

/// @nodoc
abstract mixin class $ProviderCodeCopyWith<$Res>  {
  factory $ProviderCodeCopyWith(ProviderCode value, $Res Function(ProviderCode) _then) = _$ProviderCodeCopyWithImpl;
@useResult
$Res call({
 String userCode, String verificationUri
});




}
/// @nodoc
class _$ProviderCodeCopyWithImpl<$Res>
    implements $ProviderCodeCopyWith<$Res> {
  _$ProviderCodeCopyWithImpl(this._self, this._then);

  final ProviderCode _self;
  final $Res Function(ProviderCode) _then;

/// Create a copy of ProviderCode
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? userCode = null,Object? verificationUri = null,}) {
  return _then(_self.copyWith(
userCode: null == userCode ? _self.userCode : userCode // ignore: cast_nullable_to_non_nullable
as String,verificationUri: null == verificationUri ? _self.verificationUri : verificationUri // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [ProviderCode].
extension ProviderCodePatterns on ProviderCode {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ProviderCode value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ProviderCode() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ProviderCode value)  $default,){
final _that = this;
switch (_that) {
case _ProviderCode():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ProviderCode value)?  $default,){
final _that = this;
switch (_that) {
case _ProviderCode() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String userCode,  String verificationUri)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ProviderCode() when $default != null:
return $default(_that.userCode,_that.verificationUri);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String userCode,  String verificationUri)  $default,) {final _that = this;
switch (_that) {
case _ProviderCode():
return $default(_that.userCode,_that.verificationUri);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String userCode,  String verificationUri)?  $default,) {final _that = this;
switch (_that) {
case _ProviderCode() when $default != null:
return $default(_that.userCode,_that.verificationUri);case _:
  return null;

}
}

}

/// @nodoc


class _ProviderCode implements ProviderCode {
  const _ProviderCode({required this.userCode, required this.verificationUri});
  

@override final  String userCode;
@override final  String verificationUri;

/// Create a copy of ProviderCode
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ProviderCodeCopyWith<_ProviderCode> get copyWith => __$ProviderCodeCopyWithImpl<_ProviderCode>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ProviderCode&&(identical(other.userCode, userCode) || other.userCode == userCode)&&(identical(other.verificationUri, verificationUri) || other.verificationUri == verificationUri));
}


@override
int get hashCode => Object.hash(runtimeType,userCode,verificationUri);

@override
String toString() {
  return 'ProviderCode(userCode: $userCode, verificationUri: $verificationUri)';
}


}

/// @nodoc
abstract mixin class _$ProviderCodeCopyWith<$Res> implements $ProviderCodeCopyWith<$Res> {
  factory _$ProviderCodeCopyWith(_ProviderCode value, $Res Function(_ProviderCode) _then) = __$ProviderCodeCopyWithImpl;
@override @useResult
$Res call({
 String userCode, String verificationUri
});




}
/// @nodoc
class __$ProviderCodeCopyWithImpl<$Res>
    implements _$ProviderCodeCopyWith<$Res> {
  __$ProviderCodeCopyWithImpl(this._self, this._then);

  final _ProviderCode _self;
  final $Res Function(_ProviderCode) _then;

/// Create a copy of ProviderCode
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? userCode = null,Object? verificationUri = null,}) {
  return _then(_ProviderCode(
userCode: null == userCode ? _self.userCode : userCode // ignore: cast_nullable_to_non_nullable
as String,verificationUri: null == verificationUri ? _self.verificationUri : verificationUri // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
