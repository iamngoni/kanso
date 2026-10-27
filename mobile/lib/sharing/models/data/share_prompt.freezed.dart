// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'share_prompt.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SharePrompt {

 ShareRole? get role;
/// Create a copy of SharePrompt
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharePromptCopyWith<SharePrompt> get copyWith => _$SharePromptCopyWithImpl<SharePrompt>(this as SharePrompt, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharePrompt&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,role);

@override
String toString() {
  return 'SharePrompt(role: $role)';
}


}

/// @nodoc
abstract mixin class $SharePromptCopyWith<$Res>  {
  factory $SharePromptCopyWith(SharePrompt value, $Res Function(SharePrompt) _then) = _$SharePromptCopyWithImpl;
@useResult
$Res call({
 ShareRole role
});




}
/// @nodoc
class _$SharePromptCopyWithImpl<$Res>
    implements $SharePromptCopyWith<$Res> {
  _$SharePromptCopyWithImpl(this._self, this._then);

  final SharePrompt _self;
  final $Res Function(SharePrompt) _then;

/// Create a copy of SharePrompt
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? role = null,}) {
  return _then(_self.copyWith(
role: null == role ? _self.role! : role // ignore: cast_nullable_to_non_nullable
as ShareRole,
  ));
}

}


/// Adds pattern-matching-related methods to [SharePrompt].
extension SharePromptPatterns on SharePrompt {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SharePromptVerify value)?  verify,TResult Function( SharePromptKeyChanged value)?  keyChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SharePromptVerify() when verify != null:
return verify(_that);case SharePromptKeyChanged() when keyChanged != null:
return keyChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SharePromptVerify value)  verify,required TResult Function( SharePromptKeyChanged value)  keyChanged,}){
final _that = this;
switch (_that) {
case SharePromptVerify():
return verify(_that);case SharePromptKeyChanged():
return keyChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SharePromptVerify value)?  verify,TResult? Function( SharePromptKeyChanged value)?  keyChanged,}){
final _that = this;
switch (_that) {
case SharePromptVerify() when verify != null:
return verify(_that);case SharePromptKeyChanged() when keyChanged != null:
return keyChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PersonRow person,  ShareRole role)?  verify,TResult Function( PersonRow offered,  PersonRow? pinned,  ShareRole? role)?  keyChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SharePromptVerify() when verify != null:
return verify(_that.person,_that.role);case SharePromptKeyChanged() when keyChanged != null:
return keyChanged(_that.offered,_that.pinned,_that.role);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PersonRow person,  ShareRole role)  verify,required TResult Function( PersonRow offered,  PersonRow? pinned,  ShareRole? role)  keyChanged,}) {final _that = this;
switch (_that) {
case SharePromptVerify():
return verify(_that.person,_that.role);case SharePromptKeyChanged():
return keyChanged(_that.offered,_that.pinned,_that.role);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PersonRow person,  ShareRole role)?  verify,TResult? Function( PersonRow offered,  PersonRow? pinned,  ShareRole? role)?  keyChanged,}) {final _that = this;
switch (_that) {
case SharePromptVerify() when verify != null:
return verify(_that.person,_that.role);case SharePromptKeyChanged() when keyChanged != null:
return keyChanged(_that.offered,_that.pinned,_that.role);case _:
  return null;

}
}

}

/// @nodoc


class SharePromptVerify implements SharePrompt {
  const SharePromptVerify({required this.person, required this.role});
  

 final  PersonRow person;
@override final  ShareRole role;

/// Create a copy of SharePrompt
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharePromptVerifyCopyWith<SharePromptVerify> get copyWith => _$SharePromptVerifyCopyWithImpl<SharePromptVerify>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharePromptVerify&&(identical(other.person, person) || other.person == person)&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,person,role);

@override
String toString() {
  return 'SharePrompt.verify(person: $person, role: $role)';
}


}

/// @nodoc
abstract mixin class $SharePromptVerifyCopyWith<$Res> implements $SharePromptCopyWith<$Res> {
  factory $SharePromptVerifyCopyWith(SharePromptVerify value, $Res Function(SharePromptVerify) _then) = _$SharePromptVerifyCopyWithImpl;
@override @useResult
$Res call({
 PersonRow person, ShareRole role
});




}
/// @nodoc
class _$SharePromptVerifyCopyWithImpl<$Res>
    implements $SharePromptVerifyCopyWith<$Res> {
  _$SharePromptVerifyCopyWithImpl(this._self, this._then);

  final SharePromptVerify _self;
  final $Res Function(SharePromptVerify) _then;

/// Create a copy of SharePrompt
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? person = null,Object? role = null,}) {
  return _then(SharePromptVerify(
person: null == person ? _self.person : person // ignore: cast_nullable_to_non_nullable
as PersonRow,role: null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole,
  ));
}


}

/// @nodoc


class SharePromptKeyChanged implements SharePrompt {
  const SharePromptKeyChanged({required this.offered, this.pinned, this.role});
  

 final  PersonRow offered;
 final  PersonRow? pinned;
@override final  ShareRole? role;

/// Create a copy of SharePrompt
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharePromptKeyChangedCopyWith<SharePromptKeyChanged> get copyWith => _$SharePromptKeyChangedCopyWithImpl<SharePromptKeyChanged>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharePromptKeyChanged&&(identical(other.offered, offered) || other.offered == offered)&&(identical(other.pinned, pinned) || other.pinned == pinned)&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,offered,pinned,role);

@override
String toString() {
  return 'SharePrompt.keyChanged(offered: $offered, pinned: $pinned, role: $role)';
}


}

/// @nodoc
abstract mixin class $SharePromptKeyChangedCopyWith<$Res> implements $SharePromptCopyWith<$Res> {
  factory $SharePromptKeyChangedCopyWith(SharePromptKeyChanged value, $Res Function(SharePromptKeyChanged) _then) = _$SharePromptKeyChangedCopyWithImpl;
@override @useResult
$Res call({
 PersonRow offered, PersonRow? pinned, ShareRole? role
});




}
/// @nodoc
class _$SharePromptKeyChangedCopyWithImpl<$Res>
    implements $SharePromptKeyChangedCopyWith<$Res> {
  _$SharePromptKeyChangedCopyWithImpl(this._self, this._then);

  final SharePromptKeyChanged _self;
  final $Res Function(SharePromptKeyChanged) _then;

/// Create a copy of SharePrompt
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? offered = null,Object? pinned = freezed,Object? role = freezed,}) {
  return _then(SharePromptKeyChanged(
offered: null == offered ? _self.offered : offered // ignore: cast_nullable_to_non_nullable
as PersonRow,pinned: freezed == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as PersonRow?,role: freezed == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole?,
  ));
}


}

// dart format on
