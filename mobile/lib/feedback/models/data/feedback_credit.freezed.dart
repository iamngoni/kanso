// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'feedback_credit.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FeedbackGithubUser {

 String get login;/// An https avatar URL on GitHub's avatar host.
 String get avatarUrl;
/// Create a copy of FeedbackGithubUser
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackGithubUserCopyWith<FeedbackGithubUser> get copyWith => _$FeedbackGithubUserCopyWithImpl<FeedbackGithubUser>(this as FeedbackGithubUser, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackGithubUser&&(identical(other.login, login) || other.login == login)&&(identical(other.avatarUrl, avatarUrl) || other.avatarUrl == avatarUrl));
}


@override
int get hashCode => Object.hash(runtimeType,login,avatarUrl);

@override
String toString() {
  return 'FeedbackGithubUser(login: $login, avatarUrl: $avatarUrl)';
}


}

/// @nodoc
abstract mixin class $FeedbackGithubUserCopyWith<$Res>  {
  factory $FeedbackGithubUserCopyWith(FeedbackGithubUser value, $Res Function(FeedbackGithubUser) _then) = _$FeedbackGithubUserCopyWithImpl;
@useResult
$Res call({
 String login, String avatarUrl
});




}
/// @nodoc
class _$FeedbackGithubUserCopyWithImpl<$Res>
    implements $FeedbackGithubUserCopyWith<$Res> {
  _$FeedbackGithubUserCopyWithImpl(this._self, this._then);

  final FeedbackGithubUser _self;
  final $Res Function(FeedbackGithubUser) _then;

/// Create a copy of FeedbackGithubUser
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? login = null,Object? avatarUrl = null,}) {
  return _then(_self.copyWith(
login: null == login ? _self.login : login // ignore: cast_nullable_to_non_nullable
as String,avatarUrl: null == avatarUrl ? _self.avatarUrl : avatarUrl // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [FeedbackGithubUser].
extension FeedbackGithubUserPatterns on FeedbackGithubUser {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FeedbackGithubUser value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FeedbackGithubUser() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FeedbackGithubUser value)  $default,){
final _that = this;
switch (_that) {
case _FeedbackGithubUser():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FeedbackGithubUser value)?  $default,){
final _that = this;
switch (_that) {
case _FeedbackGithubUser() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String login,  String avatarUrl)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FeedbackGithubUser() when $default != null:
return $default(_that.login,_that.avatarUrl);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String login,  String avatarUrl)  $default,) {final _that = this;
switch (_that) {
case _FeedbackGithubUser():
return $default(_that.login,_that.avatarUrl);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String login,  String avatarUrl)?  $default,) {final _that = this;
switch (_that) {
case _FeedbackGithubUser() when $default != null:
return $default(_that.login,_that.avatarUrl);case _:
  return null;

}
}

}

/// @nodoc


class _FeedbackGithubUser implements FeedbackGithubUser {
  const _FeedbackGithubUser({required this.login, required this.avatarUrl});
  

@override final  String login;
/// An https avatar URL on GitHub's avatar host.
@override final  String avatarUrl;

/// Create a copy of FeedbackGithubUser
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FeedbackGithubUserCopyWith<_FeedbackGithubUser> get copyWith => __$FeedbackGithubUserCopyWithImpl<_FeedbackGithubUser>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FeedbackGithubUser&&(identical(other.login, login) || other.login == login)&&(identical(other.avatarUrl, avatarUrl) || other.avatarUrl == avatarUrl));
}


@override
int get hashCode => Object.hash(runtimeType,login,avatarUrl);

@override
String toString() {
  return 'FeedbackGithubUser(login: $login, avatarUrl: $avatarUrl)';
}


}

/// @nodoc
abstract mixin class _$FeedbackGithubUserCopyWith<$Res> implements $FeedbackGithubUserCopyWith<$Res> {
  factory _$FeedbackGithubUserCopyWith(_FeedbackGithubUser value, $Res Function(_FeedbackGithubUser) _then) = __$FeedbackGithubUserCopyWithImpl;
@override @useResult
$Res call({
 String login, String avatarUrl
});




}
/// @nodoc
class __$FeedbackGithubUserCopyWithImpl<$Res>
    implements _$FeedbackGithubUserCopyWith<$Res> {
  __$FeedbackGithubUserCopyWithImpl(this._self, this._then);

  final _FeedbackGithubUser _self;
  final $Res Function(_FeedbackGithubUser) _then;

/// Create a copy of FeedbackGithubUser
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? login = null,Object? avatarUrl = null,}) {
  return _then(_FeedbackGithubUser(
login: null == login ? _self.login : login // ignore: cast_nullable_to_non_nullable
as String,avatarUrl: null == avatarUrl ? _self.avatarUrl : avatarUrl // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$FeedbackGithubSearch {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackGithubSearch);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackGithubSearch()';
}


}

/// @nodoc
class $FeedbackGithubSearchCopyWith<$Res>  {
$FeedbackGithubSearchCopyWith(FeedbackGithubSearch _, $Res Function(FeedbackGithubSearch) __);
}


/// Adds pattern-matching-related methods to [FeedbackGithubSearch].
extension FeedbackGithubSearchPatterns on FeedbackGithubSearch {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( FeedbackGithubSearchIdle value)?  idle,TResult Function( FeedbackGithubSearchSearching value)?  searching,TResult Function( FeedbackGithubSearchFound value)?  found,TResult Function( FeedbackGithubSearchUnavailable value)?  unavailable,required TResult orElse(),}){
final _that = this;
switch (_that) {
case FeedbackGithubSearchIdle() when idle != null:
return idle(_that);case FeedbackGithubSearchSearching() when searching != null:
return searching(_that);case FeedbackGithubSearchFound() when found != null:
return found(_that);case FeedbackGithubSearchUnavailable() when unavailable != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( FeedbackGithubSearchIdle value)  idle,required TResult Function( FeedbackGithubSearchSearching value)  searching,required TResult Function( FeedbackGithubSearchFound value)  found,required TResult Function( FeedbackGithubSearchUnavailable value)  unavailable,}){
final _that = this;
switch (_that) {
case FeedbackGithubSearchIdle():
return idle(_that);case FeedbackGithubSearchSearching():
return searching(_that);case FeedbackGithubSearchFound():
return found(_that);case FeedbackGithubSearchUnavailable():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( FeedbackGithubSearchIdle value)?  idle,TResult? Function( FeedbackGithubSearchSearching value)?  searching,TResult? Function( FeedbackGithubSearchFound value)?  found,TResult? Function( FeedbackGithubSearchUnavailable value)?  unavailable,}){
final _that = this;
switch (_that) {
case FeedbackGithubSearchIdle() when idle != null:
return idle(_that);case FeedbackGithubSearchSearching() when searching != null:
return searching(_that);case FeedbackGithubSearchFound() when found != null:
return found(_that);case FeedbackGithubSearchUnavailable() when unavailable != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  idle,TResult Function()?  searching,TResult Function( List<FeedbackGithubUser> users)?  found,TResult Function()?  unavailable,required TResult orElse(),}) {final _that = this;
switch (_that) {
case FeedbackGithubSearchIdle() when idle != null:
return idle();case FeedbackGithubSearchSearching() when searching != null:
return searching();case FeedbackGithubSearchFound() when found != null:
return found(_that.users);case FeedbackGithubSearchUnavailable() when unavailable != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  idle,required TResult Function()  searching,required TResult Function( List<FeedbackGithubUser> users)  found,required TResult Function()  unavailable,}) {final _that = this;
switch (_that) {
case FeedbackGithubSearchIdle():
return idle();case FeedbackGithubSearchSearching():
return searching();case FeedbackGithubSearchFound():
return found(_that.users);case FeedbackGithubSearchUnavailable():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  idle,TResult? Function()?  searching,TResult? Function( List<FeedbackGithubUser> users)?  found,TResult? Function()?  unavailable,}) {final _that = this;
switch (_that) {
case FeedbackGithubSearchIdle() when idle != null:
return idle();case FeedbackGithubSearchSearching() when searching != null:
return searching();case FeedbackGithubSearchFound() when found != null:
return found(_that.users);case FeedbackGithubSearchUnavailable() when unavailable != null:
return unavailable();case _:
  return null;

}
}

}

/// @nodoc


class FeedbackGithubSearchIdle implements FeedbackGithubSearch {
  const FeedbackGithubSearchIdle();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackGithubSearchIdle);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackGithubSearch.idle()';
}


}




/// @nodoc


class FeedbackGithubSearchSearching implements FeedbackGithubSearch {
  const FeedbackGithubSearchSearching();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackGithubSearchSearching);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackGithubSearch.searching()';
}


}




/// @nodoc


class FeedbackGithubSearchFound implements FeedbackGithubSearch {
  const FeedbackGithubSearchFound(final  List<FeedbackGithubUser> users): _users = users;
  

 final  List<FeedbackGithubUser> _users;
 List<FeedbackGithubUser> get users {
  if (_users is EqualUnmodifiableListView) return _users;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_users);
}


/// Create a copy of FeedbackGithubSearch
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FeedbackGithubSearchFoundCopyWith<FeedbackGithubSearchFound> get copyWith => _$FeedbackGithubSearchFoundCopyWithImpl<FeedbackGithubSearchFound>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackGithubSearchFound&&const DeepCollectionEquality().equals(other._users, _users));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_users));

@override
String toString() {
  return 'FeedbackGithubSearch.found(users: $users)';
}


}

/// @nodoc
abstract mixin class $FeedbackGithubSearchFoundCopyWith<$Res> implements $FeedbackGithubSearchCopyWith<$Res> {
  factory $FeedbackGithubSearchFoundCopyWith(FeedbackGithubSearchFound value, $Res Function(FeedbackGithubSearchFound) _then) = _$FeedbackGithubSearchFoundCopyWithImpl;
@useResult
$Res call({
 List<FeedbackGithubUser> users
});




}
/// @nodoc
class _$FeedbackGithubSearchFoundCopyWithImpl<$Res>
    implements $FeedbackGithubSearchFoundCopyWith<$Res> {
  _$FeedbackGithubSearchFoundCopyWithImpl(this._self, this._then);

  final FeedbackGithubSearchFound _self;
  final $Res Function(FeedbackGithubSearchFound) _then;

/// Create a copy of FeedbackGithubSearch
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? users = null,}) {
  return _then(FeedbackGithubSearchFound(
null == users ? _self._users : users // ignore: cast_nullable_to_non_nullable
as List<FeedbackGithubUser>,
  ));
}


}

/// @nodoc


class FeedbackGithubSearchUnavailable implements FeedbackGithubSearch {
  const FeedbackGithubSearchUnavailable();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FeedbackGithubSearchUnavailable);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FeedbackGithubSearch.unavailable()';
}


}




// dart format on
