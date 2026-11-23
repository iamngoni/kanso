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
mixin _$FeedbackCredit {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackCredit);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackCredit()';
}


}

/// @nodoc
class $FeedbackCreditCopyWith<$Res>  {
$FeedbackCreditCopyWith(FeedbackCredit _, $Res Function(FeedbackCredit) __);
}


/// Adds pattern-matching-related methods to [FeedbackCredit].
extension FeedbackCreditPatterns on FeedbackCredit {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( FeedbackCredit_Name value)?  name,TResult Function( FeedbackCredit_Github value)?  github,required TResult orElse(),}){
final _that = this;
switch (_that) {
case FeedbackCredit_Name() when name != null:
return name(_that);case FeedbackCredit_Github() when github != null:
return github(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( FeedbackCredit_Name value)  name,required TResult Function( FeedbackCredit_Github value)  github,}){
final _that = this;
switch (_that) {
case FeedbackCredit_Name():
return name(_that);case FeedbackCredit_Github():
return github(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( FeedbackCredit_Name value)?  name,TResult? Function( FeedbackCredit_Github value)?  github,}){
final _that = this;
switch (_that) {
case FeedbackCredit_Name() when name != null:
return name(_that);case FeedbackCredit_Github() when github != null:
return github(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String name)?  name,TResult Function( String login)?  github,required TResult orElse(),}) {final _that = this;
switch (_that) {
case FeedbackCredit_Name() when name != null:
return name(_that.name);case FeedbackCredit_Github() when github != null:
return github(_that.login);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String name)  name,required TResult Function( String login)  github,}) {final _that = this;
switch (_that) {
case FeedbackCredit_Name():
return name(_that.name);case FeedbackCredit_Github():
return github(_that.login);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String name)?  name,TResult? Function( String login)?  github,}) {final _that = this;
switch (_that) {
case FeedbackCredit_Name() when name != null:
return name(_that.name);case FeedbackCredit_Github() when github != null:
return github(_that.login);case _:
  return null;

}
}

}

/// @nodoc


class FeedbackCredit_Name extends FeedbackCredit {
  const FeedbackCredit_Name({required this.name}): super._();
  

 final  String name;

/// Create a copy of FeedbackCredit
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackCredit_NameCopyWith<FeedbackCredit_Name> get copyWith => _$FeedbackCredit_NameCopyWithImpl<FeedbackCredit_Name>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackCredit_Name&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'FeedbackCredit.name(name: $name)';
}


}

/// @nodoc
abstract mixin class $FeedbackCredit_NameCopyWith<$Res> implements $FeedbackCreditCopyWith<$Res> {
  factory $FeedbackCredit_NameCopyWith(FeedbackCredit_Name value, $Res Function(FeedbackCredit_Name) _then) = _$FeedbackCredit_NameCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$FeedbackCredit_NameCopyWithImpl<$Res>
    implements $FeedbackCredit_NameCopyWith<$Res> {
  _$FeedbackCredit_NameCopyWithImpl(this._self, this._then);

  final FeedbackCredit_Name _self;
  final $Res Function(FeedbackCredit_Name) _then;

/// Create a copy of FeedbackCredit
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(FeedbackCredit_Name(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class FeedbackCredit_Github extends FeedbackCredit {
  const FeedbackCredit_Github({required this.login}): super._();
  

 final  String login;

/// Create a copy of FeedbackCredit
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackCredit_GithubCopyWith<FeedbackCredit_Github> get copyWith => _$FeedbackCredit_GithubCopyWithImpl<FeedbackCredit_Github>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackCredit_Github&&(identical(other.login, login) || other.login == login));
}


@override
int get hashCode => Object.hash(runtimeType,login);

@override
String toString() {
  return 'FeedbackCredit.github(login: $login)';
}


}

/// @nodoc
abstract mixin class $FeedbackCredit_GithubCopyWith<$Res> implements $FeedbackCreditCopyWith<$Res> {
  factory $FeedbackCredit_GithubCopyWith(FeedbackCredit_Github value, $Res Function(FeedbackCredit_Github) _then) = _$FeedbackCredit_GithubCopyWithImpl;
@useResult
$Res call({
 String login
});




}
/// @nodoc
class _$FeedbackCredit_GithubCopyWithImpl<$Res>
    implements $FeedbackCredit_GithubCopyWith<$Res> {
  _$FeedbackCredit_GithubCopyWithImpl(this._self, this._then);

  final FeedbackCredit_Github _self;
  final $Res Function(FeedbackCredit_Github) _then;

/// Create a copy of FeedbackCredit
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? login = null,}) {
  return _then(FeedbackCredit_Github(
login: null == login ? _self.login : login // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

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

/// @nodoc
mixin _$FeedbackUsers {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackUsers);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackUsers()';
}


}

/// @nodoc
class $FeedbackUsersCopyWith<$Res>  {
$FeedbackUsersCopyWith(FeedbackUsers _, $Res Function(FeedbackUsers) __);
}


/// Adds pattern-matching-related methods to [FeedbackUsers].
extension FeedbackUsersPatterns on FeedbackUsers {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( FeedbackUsers_Found value)?  found,TResult Function( FeedbackUsers_Unavailable value)?  unavailable,required TResult orElse(),}){
final _that = this;
switch (_that) {
case FeedbackUsers_Found() when found != null:
return found(_that);case FeedbackUsers_Unavailable() when unavailable != null:
return unavailable(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( FeedbackUsers_Found value)  found,required TResult Function( FeedbackUsers_Unavailable value)  unavailable,}){
final _that = this;
switch (_that) {
case FeedbackUsers_Found():
return found(_that);case FeedbackUsers_Unavailable():
return unavailable(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( FeedbackUsers_Found value)?  found,TResult? Function( FeedbackUsers_Unavailable value)?  unavailable,}){
final _that = this;
switch (_that) {
case FeedbackUsers_Found() when found != null:
return found(_that);case FeedbackUsers_Unavailable() when unavailable != null:
return unavailable(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<FeedbackGithubUser> users)?  found,TResult Function()?  unavailable,required TResult orElse(),}) {final _that = this;
switch (_that) {
case FeedbackUsers_Found() when found != null:
return found(_that.users);case FeedbackUsers_Unavailable() when unavailable != null:
return unavailable();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<FeedbackGithubUser> users)  found,required TResult Function()  unavailable,}) {final _that = this;
switch (_that) {
case FeedbackUsers_Found():
return found(_that.users);case FeedbackUsers_Unavailable():
return unavailable();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<FeedbackGithubUser> users)?  found,TResult? Function()?  unavailable,}) {final _that = this;
switch (_that) {
case FeedbackUsers_Found() when found != null:
return found(_that.users);case FeedbackUsers_Unavailable() when unavailable != null:
return unavailable();case _:
  return null;

}
}

}

/// @nodoc


class FeedbackUsers_Found extends FeedbackUsers {
  const FeedbackUsers_Found({required final  List<FeedbackGithubUser> users}): _users = users,super._();
  

 final  List<FeedbackGithubUser> _users;
 List<FeedbackGithubUser> get users {
  if (_users is EqualUnmodifiableListView) return _users;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_users);
}


/// Create a copy of FeedbackUsers
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackUsers_FoundCopyWith<FeedbackUsers_Found> get copyWith => _$FeedbackUsers_FoundCopyWithImpl<FeedbackUsers_Found>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackUsers_Found&&const DeepCollectionEquality().equals(other._users, _users));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_users));

@override
String toString() {
  return 'FeedbackUsers.found(users: $users)';
}


}

/// @nodoc
abstract mixin class $FeedbackUsers_FoundCopyWith<$Res> implements $FeedbackUsersCopyWith<$Res> {
  factory $FeedbackUsers_FoundCopyWith(FeedbackUsers_Found value, $Res Function(FeedbackUsers_Found) _then) = _$FeedbackUsers_FoundCopyWithImpl;
@useResult
$Res call({
 List<FeedbackGithubUser> users
});




}
/// @nodoc
class _$FeedbackUsers_FoundCopyWithImpl<$Res>
    implements $FeedbackUsers_FoundCopyWith<$Res> {
  _$FeedbackUsers_FoundCopyWithImpl(this._self, this._then);

  final FeedbackUsers_Found _self;
  final $Res Function(FeedbackUsers_Found) _then;

/// Create a copy of FeedbackUsers
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? users = null,}) {
  return _then(FeedbackUsers_Found(
users: null == users ? _self._users : users // ignore: cast_nullable_to_non_nullable
as List<FeedbackGithubUser>,
  ));
}


}

/// @nodoc


class FeedbackUsers_Unavailable extends FeedbackUsers {
  const FeedbackUsers_Unavailable(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackUsers_Unavailable);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackUsers.unavailable()';
}


}




// dart format on
