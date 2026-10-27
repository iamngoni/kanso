// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'account_action_field.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AccountActionField {

 String get label;/// Whether what is typed is hidden.
 bool get obscure;
/// Create a copy of AccountActionField
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountActionFieldCopyWith<AccountActionField> get copyWith => _$AccountActionFieldCopyWithImpl<AccountActionField>(this as AccountActionField, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountActionField&&(identical(other.label, label) || other.label == label)&&(identical(other.obscure, obscure) || other.obscure == obscure));
}


@override
int get hashCode => Object.hash(runtimeType,label,obscure);

@override
String toString() {
  return 'AccountActionField(label: $label, obscure: $obscure)';
}


}

/// @nodoc
abstract mixin class $AccountActionFieldCopyWith<$Res>  {
  factory $AccountActionFieldCopyWith(AccountActionField value, $Res Function(AccountActionField) _then) = _$AccountActionFieldCopyWithImpl;
@useResult
$Res call({
 String label, bool obscure
});




}
/// @nodoc
class _$AccountActionFieldCopyWithImpl<$Res>
    implements $AccountActionFieldCopyWith<$Res> {
  _$AccountActionFieldCopyWithImpl(this._self, this._then);

  final AccountActionField _self;
  final $Res Function(AccountActionField) _then;

/// Create a copy of AccountActionField
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? label = null,Object? obscure = null,}) {
  return _then(_self.copyWith(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,obscure: null == obscure ? _self.obscure : obscure // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [AccountActionField].
extension AccountActionFieldPatterns on AccountActionField {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AccountActionField value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AccountActionField() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AccountActionField value)  $default,){
final _that = this;
switch (_that) {
case _AccountActionField():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AccountActionField value)?  $default,){
final _that = this;
switch (_that) {
case _AccountActionField() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String label,  bool obscure)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AccountActionField() when $default != null:
return $default(_that.label,_that.obscure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String label,  bool obscure)  $default,) {final _that = this;
switch (_that) {
case _AccountActionField():
return $default(_that.label,_that.obscure);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String label,  bool obscure)?  $default,) {final _that = this;
switch (_that) {
case _AccountActionField() when $default != null:
return $default(_that.label,_that.obscure);case _:
  return null;

}
}

}

/// @nodoc


class _AccountActionField implements AccountActionField {
  const _AccountActionField({required this.label, this.obscure = true});
  

@override final  String label;
/// Whether what is typed is hidden.
@override@JsonKey() final  bool obscure;

/// Create a copy of AccountActionField
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AccountActionFieldCopyWith<_AccountActionField> get copyWith => __$AccountActionFieldCopyWithImpl<_AccountActionField>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AccountActionField&&(identical(other.label, label) || other.label == label)&&(identical(other.obscure, obscure) || other.obscure == obscure));
}


@override
int get hashCode => Object.hash(runtimeType,label,obscure);

@override
String toString() {
  return 'AccountActionField(label: $label, obscure: $obscure)';
}


}

/// @nodoc
abstract mixin class _$AccountActionFieldCopyWith<$Res> implements $AccountActionFieldCopyWith<$Res> {
  factory _$AccountActionFieldCopyWith(_AccountActionField value, $Res Function(_AccountActionField) _then) = __$AccountActionFieldCopyWithImpl;
@override @useResult
$Res call({
 String label, bool obscure
});




}
/// @nodoc
class __$AccountActionFieldCopyWithImpl<$Res>
    implements _$AccountActionFieldCopyWith<$Res> {
  __$AccountActionFieldCopyWithImpl(this._self, this._then);

  final _AccountActionField _self;
  final $Res Function(_AccountActionField) _then;

/// Create a copy of AccountActionField
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? label = null,Object? obscure = null,}) {
  return _then(_AccountActionField(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,obscure: null == obscure ? _self.obscure : obscure // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
