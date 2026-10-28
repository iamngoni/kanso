// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'sign_in_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SignInEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SignInEvent()';
}


}

/// @nodoc
class $SignInEventCopyWith<$Res>  {
$SignInEventCopyWith(SignInEvent _, $Res Function(SignInEvent) __);
}


/// Adds pattern-matching-related methods to [SignInEvent].
extension SignInEventPatterns on SignInEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SignInStartedEvent value)?  started,TResult Function( SignInServerEditedEvent value)?  serverEdited,TResult Function( SignInServerChosenEvent value)?  serverChosen,TResult Function( SignInMethodsRetriedEvent value)?  methodsRetried,TResult Function( SignInModeChangedEvent value)?  modeChanged,TResult Function( SignInPasswordSubmittedEvent value)?  passwordSubmitted,TResult Function( SignInSignUpSubmittedEvent value)?  signUpSubmitted,TResult Function( SignInPasskeySignInPressedEvent value)?  passkeySignInPressed,TResult Function( SignInPasskeySignUpPressedEvent value)?  passkeySignUpPressed,TResult Function( SignInProviderPressedEvent value)?  providerPressed,TResult Function( SignInGithubOpenedEvent value)?  githubOpened,TResult Function( SignInProviderCancelledEvent value)?  providerCancelled,TResult Function( SignInUsernameChosenEvent value)?  usernameChosen,TResult Function( SignInBindingAnsweredEvent value)?  bindingAnswered,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SignInStartedEvent() when started != null:
return started(_that);case SignInServerEditedEvent() when serverEdited != null:
return serverEdited(_that);case SignInServerChosenEvent() when serverChosen != null:
return serverChosen(_that);case SignInMethodsRetriedEvent() when methodsRetried != null:
return methodsRetried(_that);case SignInModeChangedEvent() when modeChanged != null:
return modeChanged(_that);case SignInPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case SignInSignUpSubmittedEvent() when signUpSubmitted != null:
return signUpSubmitted(_that);case SignInPasskeySignInPressedEvent() when passkeySignInPressed != null:
return passkeySignInPressed(_that);case SignInPasskeySignUpPressedEvent() when passkeySignUpPressed != null:
return passkeySignUpPressed(_that);case SignInProviderPressedEvent() when providerPressed != null:
return providerPressed(_that);case SignInGithubOpenedEvent() when githubOpened != null:
return githubOpened(_that);case SignInProviderCancelledEvent() when providerCancelled != null:
return providerCancelled(_that);case SignInUsernameChosenEvent() when usernameChosen != null:
return usernameChosen(_that);case SignInBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SignInStartedEvent value)  started,required TResult Function( SignInServerEditedEvent value)  serverEdited,required TResult Function( SignInServerChosenEvent value)  serverChosen,required TResult Function( SignInMethodsRetriedEvent value)  methodsRetried,required TResult Function( SignInModeChangedEvent value)  modeChanged,required TResult Function( SignInPasswordSubmittedEvent value)  passwordSubmitted,required TResult Function( SignInSignUpSubmittedEvent value)  signUpSubmitted,required TResult Function( SignInPasskeySignInPressedEvent value)  passkeySignInPressed,required TResult Function( SignInPasskeySignUpPressedEvent value)  passkeySignUpPressed,required TResult Function( SignInProviderPressedEvent value)  providerPressed,required TResult Function( SignInGithubOpenedEvent value)  githubOpened,required TResult Function( SignInProviderCancelledEvent value)  providerCancelled,required TResult Function( SignInUsernameChosenEvent value)  usernameChosen,required TResult Function( SignInBindingAnsweredEvent value)  bindingAnswered,}){
final _that = this;
switch (_that) {
case SignInStartedEvent():
return started(_that);case SignInServerEditedEvent():
return serverEdited(_that);case SignInServerChosenEvent():
return serverChosen(_that);case SignInMethodsRetriedEvent():
return methodsRetried(_that);case SignInModeChangedEvent():
return modeChanged(_that);case SignInPasswordSubmittedEvent():
return passwordSubmitted(_that);case SignInSignUpSubmittedEvent():
return signUpSubmitted(_that);case SignInPasskeySignInPressedEvent():
return passkeySignInPressed(_that);case SignInPasskeySignUpPressedEvent():
return passkeySignUpPressed(_that);case SignInProviderPressedEvent():
return providerPressed(_that);case SignInGithubOpenedEvent():
return githubOpened(_that);case SignInProviderCancelledEvent():
return providerCancelled(_that);case SignInUsernameChosenEvent():
return usernameChosen(_that);case SignInBindingAnsweredEvent():
return bindingAnswered(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SignInStartedEvent value)?  started,TResult? Function( SignInServerEditedEvent value)?  serverEdited,TResult? Function( SignInServerChosenEvent value)?  serverChosen,TResult? Function( SignInMethodsRetriedEvent value)?  methodsRetried,TResult? Function( SignInModeChangedEvent value)?  modeChanged,TResult? Function( SignInPasswordSubmittedEvent value)?  passwordSubmitted,TResult? Function( SignInSignUpSubmittedEvent value)?  signUpSubmitted,TResult? Function( SignInPasskeySignInPressedEvent value)?  passkeySignInPressed,TResult? Function( SignInPasskeySignUpPressedEvent value)?  passkeySignUpPressed,TResult? Function( SignInProviderPressedEvent value)?  providerPressed,TResult? Function( SignInGithubOpenedEvent value)?  githubOpened,TResult? Function( SignInProviderCancelledEvent value)?  providerCancelled,TResult? Function( SignInUsernameChosenEvent value)?  usernameChosen,TResult? Function( SignInBindingAnsweredEvent value)?  bindingAnswered,}){
final _that = this;
switch (_that) {
case SignInStartedEvent() when started != null:
return started(_that);case SignInServerEditedEvent() when serverEdited != null:
return serverEdited(_that);case SignInServerChosenEvent() when serverChosen != null:
return serverChosen(_that);case SignInMethodsRetriedEvent() when methodsRetried != null:
return methodsRetried(_that);case SignInModeChangedEvent() when modeChanged != null:
return modeChanged(_that);case SignInPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that);case SignInSignUpSubmittedEvent() when signUpSubmitted != null:
return signUpSubmitted(_that);case SignInPasskeySignInPressedEvent() when passkeySignInPressed != null:
return passkeySignInPressed(_that);case SignInPasskeySignUpPressedEvent() when passkeySignUpPressed != null:
return passkeySignUpPressed(_that);case SignInProviderPressedEvent() when providerPressed != null:
return providerPressed(_that);case SignInGithubOpenedEvent() when githubOpened != null:
return githubOpened(_that);case SignInProviderCancelledEvent() when providerCancelled != null:
return providerCancelled(_that);case SignInUsernameChosenEvent() when usernameChosen != null:
return usernameChosen(_that);case SignInBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( bool editing)?  serverEdited,TResult Function( String address)?  serverChosen,TResult Function()?  methodsRetried,TResult Function( SignInMode mode)?  modeChanged,TResult Function( String username,  String password,  String deviceName)?  passwordSubmitted,TResult Function( String username,  String password,  String repeat,  String deviceName)?  signUpSubmitted,TResult Function( String deviceName)?  passkeySignInPressed,TResult Function( String username,  String deviceName)?  passkeySignUpPressed,TResult Function( ProviderKind provider,  String deviceName)?  providerPressed,TResult Function()?  githubOpened,TResult Function()?  providerCancelled,TResult Function( String username)?  usernameChosen,TResult Function( BindingAnswer? answer)?  bindingAnswered,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SignInStartedEvent() when started != null:
return started();case SignInServerEditedEvent() when serverEdited != null:
return serverEdited(_that.editing);case SignInServerChosenEvent() when serverChosen != null:
return serverChosen(_that.address);case SignInMethodsRetriedEvent() when methodsRetried != null:
return methodsRetried();case SignInModeChangedEvent() when modeChanged != null:
return modeChanged(_that.mode);case SignInPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.username,_that.password,_that.deviceName);case SignInSignUpSubmittedEvent() when signUpSubmitted != null:
return signUpSubmitted(_that.username,_that.password,_that.repeat,_that.deviceName);case SignInPasskeySignInPressedEvent() when passkeySignInPressed != null:
return passkeySignInPressed(_that.deviceName);case SignInPasskeySignUpPressedEvent() when passkeySignUpPressed != null:
return passkeySignUpPressed(_that.username,_that.deviceName);case SignInProviderPressedEvent() when providerPressed != null:
return providerPressed(_that.provider,_that.deviceName);case SignInGithubOpenedEvent() when githubOpened != null:
return githubOpened();case SignInProviderCancelledEvent() when providerCancelled != null:
return providerCancelled();case SignInUsernameChosenEvent() when usernameChosen != null:
return usernameChosen(_that.username);case SignInBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that.answer);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( bool editing)  serverEdited,required TResult Function( String address)  serverChosen,required TResult Function()  methodsRetried,required TResult Function( SignInMode mode)  modeChanged,required TResult Function( String username,  String password,  String deviceName)  passwordSubmitted,required TResult Function( String username,  String password,  String repeat,  String deviceName)  signUpSubmitted,required TResult Function( String deviceName)  passkeySignInPressed,required TResult Function( String username,  String deviceName)  passkeySignUpPressed,required TResult Function( ProviderKind provider,  String deviceName)  providerPressed,required TResult Function()  githubOpened,required TResult Function()  providerCancelled,required TResult Function( String username)  usernameChosen,required TResult Function( BindingAnswer? answer)  bindingAnswered,}) {final _that = this;
switch (_that) {
case SignInStartedEvent():
return started();case SignInServerEditedEvent():
return serverEdited(_that.editing);case SignInServerChosenEvent():
return serverChosen(_that.address);case SignInMethodsRetriedEvent():
return methodsRetried();case SignInModeChangedEvent():
return modeChanged(_that.mode);case SignInPasswordSubmittedEvent():
return passwordSubmitted(_that.username,_that.password,_that.deviceName);case SignInSignUpSubmittedEvent():
return signUpSubmitted(_that.username,_that.password,_that.repeat,_that.deviceName);case SignInPasskeySignInPressedEvent():
return passkeySignInPressed(_that.deviceName);case SignInPasskeySignUpPressedEvent():
return passkeySignUpPressed(_that.username,_that.deviceName);case SignInProviderPressedEvent():
return providerPressed(_that.provider,_that.deviceName);case SignInGithubOpenedEvent():
return githubOpened();case SignInProviderCancelledEvent():
return providerCancelled();case SignInUsernameChosenEvent():
return usernameChosen(_that.username);case SignInBindingAnsweredEvent():
return bindingAnswered(_that.answer);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( bool editing)?  serverEdited,TResult? Function( String address)?  serverChosen,TResult? Function()?  methodsRetried,TResult? Function( SignInMode mode)?  modeChanged,TResult? Function( String username,  String password,  String deviceName)?  passwordSubmitted,TResult? Function( String username,  String password,  String repeat,  String deviceName)?  signUpSubmitted,TResult? Function( String deviceName)?  passkeySignInPressed,TResult? Function( String username,  String deviceName)?  passkeySignUpPressed,TResult? Function( ProviderKind provider,  String deviceName)?  providerPressed,TResult? Function()?  githubOpened,TResult? Function()?  providerCancelled,TResult? Function( String username)?  usernameChosen,TResult? Function( BindingAnswer? answer)?  bindingAnswered,}) {final _that = this;
switch (_that) {
case SignInStartedEvent() when started != null:
return started();case SignInServerEditedEvent() when serverEdited != null:
return serverEdited(_that.editing);case SignInServerChosenEvent() when serverChosen != null:
return serverChosen(_that.address);case SignInMethodsRetriedEvent() when methodsRetried != null:
return methodsRetried();case SignInModeChangedEvent() when modeChanged != null:
return modeChanged(_that.mode);case SignInPasswordSubmittedEvent() when passwordSubmitted != null:
return passwordSubmitted(_that.username,_that.password,_that.deviceName);case SignInSignUpSubmittedEvent() when signUpSubmitted != null:
return signUpSubmitted(_that.username,_that.password,_that.repeat,_that.deviceName);case SignInPasskeySignInPressedEvent() when passkeySignInPressed != null:
return passkeySignInPressed(_that.deviceName);case SignInPasskeySignUpPressedEvent() when passkeySignUpPressed != null:
return passkeySignUpPressed(_that.username,_that.deviceName);case SignInProviderPressedEvent() when providerPressed != null:
return providerPressed(_that.provider,_that.deviceName);case SignInGithubOpenedEvent() when githubOpened != null:
return githubOpened();case SignInProviderCancelledEvent() when providerCancelled != null:
return providerCancelled();case SignInUsernameChosenEvent() when usernameChosen != null:
return usernameChosen(_that.username);case SignInBindingAnsweredEvent() when bindingAnswered != null:
return bindingAnswered(_that.answer);case _:
  return null;

}
}

}

/// @nodoc


class SignInStartedEvent implements SignInEvent {
  const SignInStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SignInEvent.started()';
}


}




/// @nodoc


class SignInServerEditedEvent implements SignInEvent {
  const SignInServerEditedEvent({required this.editing});
  

 final  bool editing;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInServerEditedEventCopyWith<SignInServerEditedEvent> get copyWith => _$SignInServerEditedEventCopyWithImpl<SignInServerEditedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInServerEditedEvent&&(identical(other.editing, editing) || other.editing == editing));
}


@override
int get hashCode => Object.hash(runtimeType,editing);

@override
String toString() {
  return 'SignInEvent.serverEdited(editing: $editing)';
}


}

/// @nodoc
abstract mixin class $SignInServerEditedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInServerEditedEventCopyWith(SignInServerEditedEvent value, $Res Function(SignInServerEditedEvent) _then) = _$SignInServerEditedEventCopyWithImpl;
@useResult
$Res call({
 bool editing
});




}
/// @nodoc
class _$SignInServerEditedEventCopyWithImpl<$Res>
    implements $SignInServerEditedEventCopyWith<$Res> {
  _$SignInServerEditedEventCopyWithImpl(this._self, this._then);

  final SignInServerEditedEvent _self;
  final $Res Function(SignInServerEditedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? editing = null,}) {
  return _then(SignInServerEditedEvent(
editing: null == editing ? _self.editing : editing // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class SignInServerChosenEvent implements SignInEvent {
  const SignInServerChosenEvent(this.address);
  

 final  String address;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInServerChosenEventCopyWith<SignInServerChosenEvent> get copyWith => _$SignInServerChosenEventCopyWithImpl<SignInServerChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInServerChosenEvent&&(identical(other.address, address) || other.address == address));
}


@override
int get hashCode => Object.hash(runtimeType,address);

@override
String toString() {
  return 'SignInEvent.serverChosen(address: $address)';
}


}

/// @nodoc
abstract mixin class $SignInServerChosenEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInServerChosenEventCopyWith(SignInServerChosenEvent value, $Res Function(SignInServerChosenEvent) _then) = _$SignInServerChosenEventCopyWithImpl;
@useResult
$Res call({
 String address
});




}
/// @nodoc
class _$SignInServerChosenEventCopyWithImpl<$Res>
    implements $SignInServerChosenEventCopyWith<$Res> {
  _$SignInServerChosenEventCopyWithImpl(this._self, this._then);

  final SignInServerChosenEvent _self;
  final $Res Function(SignInServerChosenEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? address = null,}) {
  return _then(SignInServerChosenEvent(
null == address ? _self.address : address // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInMethodsRetriedEvent implements SignInEvent {
  const SignInMethodsRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInMethodsRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SignInEvent.methodsRetried()';
}


}




/// @nodoc


class SignInModeChangedEvent implements SignInEvent {
  const SignInModeChangedEvent(this.mode);
  

 final  SignInMode mode;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInModeChangedEventCopyWith<SignInModeChangedEvent> get copyWith => _$SignInModeChangedEventCopyWithImpl<SignInModeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInModeChangedEvent&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,mode);

@override
String toString() {
  return 'SignInEvent.modeChanged(mode: $mode)';
}


}

/// @nodoc
abstract mixin class $SignInModeChangedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInModeChangedEventCopyWith(SignInModeChangedEvent value, $Res Function(SignInModeChangedEvent) _then) = _$SignInModeChangedEventCopyWithImpl;
@useResult
$Res call({
 SignInMode mode
});




}
/// @nodoc
class _$SignInModeChangedEventCopyWithImpl<$Res>
    implements $SignInModeChangedEventCopyWith<$Res> {
  _$SignInModeChangedEventCopyWithImpl(this._self, this._then);

  final SignInModeChangedEvent _self;
  final $Res Function(SignInModeChangedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? mode = null,}) {
  return _then(SignInModeChangedEvent(
null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as SignInMode,
  ));
}


}

/// @nodoc


class SignInPasswordSubmittedEvent implements SignInEvent {
  const SignInPasswordSubmittedEvent({required this.username, required this.password, required this.deviceName});
  

 final  String username;
 final  String password;
 final  String deviceName;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInPasswordSubmittedEventCopyWith<SignInPasswordSubmittedEvent> get copyWith => _$SignInPasswordSubmittedEventCopyWithImpl<SignInPasswordSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInPasswordSubmittedEvent&&(identical(other.username, username) || other.username == username)&&(identical(other.password, password) || other.password == password)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,username,password,deviceName);

@override
String toString() {
  return 'SignInEvent.passwordSubmitted(username: $username, password: $password, deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $SignInPasswordSubmittedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInPasswordSubmittedEventCopyWith(SignInPasswordSubmittedEvent value, $Res Function(SignInPasswordSubmittedEvent) _then) = _$SignInPasswordSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String username, String password, String deviceName
});




}
/// @nodoc
class _$SignInPasswordSubmittedEventCopyWithImpl<$Res>
    implements $SignInPasswordSubmittedEventCopyWith<$Res> {
  _$SignInPasswordSubmittedEventCopyWithImpl(this._self, this._then);

  final SignInPasswordSubmittedEvent _self;
  final $Res Function(SignInPasswordSubmittedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? password = null,Object? deviceName = null,}) {
  return _then(SignInPasswordSubmittedEvent(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,password: null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInSignUpSubmittedEvent implements SignInEvent {
  const SignInSignUpSubmittedEvent({required this.username, required this.password, required this.repeat, required this.deviceName});
  

 final  String username;
 final  String password;
 final  String repeat;
 final  String deviceName;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInSignUpSubmittedEventCopyWith<SignInSignUpSubmittedEvent> get copyWith => _$SignInSignUpSubmittedEventCopyWithImpl<SignInSignUpSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInSignUpSubmittedEvent&&(identical(other.username, username) || other.username == username)&&(identical(other.password, password) || other.password == password)&&(identical(other.repeat, repeat) || other.repeat == repeat)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,username,password,repeat,deviceName);

@override
String toString() {
  return 'SignInEvent.signUpSubmitted(username: $username, password: $password, repeat: $repeat, deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $SignInSignUpSubmittedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInSignUpSubmittedEventCopyWith(SignInSignUpSubmittedEvent value, $Res Function(SignInSignUpSubmittedEvent) _then) = _$SignInSignUpSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String username, String password, String repeat, String deviceName
});




}
/// @nodoc
class _$SignInSignUpSubmittedEventCopyWithImpl<$Res>
    implements $SignInSignUpSubmittedEventCopyWith<$Res> {
  _$SignInSignUpSubmittedEventCopyWithImpl(this._self, this._then);

  final SignInSignUpSubmittedEvent _self;
  final $Res Function(SignInSignUpSubmittedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? password = null,Object? repeat = null,Object? deviceName = null,}) {
  return _then(SignInSignUpSubmittedEvent(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,password: null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,repeat: null == repeat ? _self.repeat : repeat // ignore: cast_nullable_to_non_nullable
as String,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInPasskeySignInPressedEvent implements SignInEvent {
  const SignInPasskeySignInPressedEvent({required this.deviceName});
  

 final  String deviceName;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInPasskeySignInPressedEventCopyWith<SignInPasskeySignInPressedEvent> get copyWith => _$SignInPasskeySignInPressedEventCopyWithImpl<SignInPasskeySignInPressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInPasskeySignInPressedEvent&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,deviceName);

@override
String toString() {
  return 'SignInEvent.passkeySignInPressed(deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $SignInPasskeySignInPressedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInPasskeySignInPressedEventCopyWith(SignInPasskeySignInPressedEvent value, $Res Function(SignInPasskeySignInPressedEvent) _then) = _$SignInPasskeySignInPressedEventCopyWithImpl;
@useResult
$Res call({
 String deviceName
});




}
/// @nodoc
class _$SignInPasskeySignInPressedEventCopyWithImpl<$Res>
    implements $SignInPasskeySignInPressedEventCopyWith<$Res> {
  _$SignInPasskeySignInPressedEventCopyWithImpl(this._self, this._then);

  final SignInPasskeySignInPressedEvent _self;
  final $Res Function(SignInPasskeySignInPressedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deviceName = null,}) {
  return _then(SignInPasskeySignInPressedEvent(
deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInPasskeySignUpPressedEvent implements SignInEvent {
  const SignInPasskeySignUpPressedEvent({required this.username, required this.deviceName});
  

 final  String username;
 final  String deviceName;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInPasskeySignUpPressedEventCopyWith<SignInPasskeySignUpPressedEvent> get copyWith => _$SignInPasskeySignUpPressedEventCopyWithImpl<SignInPasskeySignUpPressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInPasskeySignUpPressedEvent&&(identical(other.username, username) || other.username == username)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,username,deviceName);

@override
String toString() {
  return 'SignInEvent.passkeySignUpPressed(username: $username, deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $SignInPasskeySignUpPressedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInPasskeySignUpPressedEventCopyWith(SignInPasskeySignUpPressedEvent value, $Res Function(SignInPasskeySignUpPressedEvent) _then) = _$SignInPasskeySignUpPressedEventCopyWithImpl;
@useResult
$Res call({
 String username, String deviceName
});




}
/// @nodoc
class _$SignInPasskeySignUpPressedEventCopyWithImpl<$Res>
    implements $SignInPasskeySignUpPressedEventCopyWith<$Res> {
  _$SignInPasskeySignUpPressedEventCopyWithImpl(this._self, this._then);

  final SignInPasskeySignUpPressedEvent _self;
  final $Res Function(SignInPasskeySignUpPressedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? deviceName = null,}) {
  return _then(SignInPasskeySignUpPressedEvent(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInProviderPressedEvent implements SignInEvent {
  const SignInProviderPressedEvent({required this.provider, required this.deviceName});
  

 final  ProviderKind provider;
 final  String deviceName;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInProviderPressedEventCopyWith<SignInProviderPressedEvent> get copyWith => _$SignInProviderPressedEventCopyWithImpl<SignInProviderPressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInProviderPressedEvent&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.deviceName, deviceName) || other.deviceName == deviceName));
}


@override
int get hashCode => Object.hash(runtimeType,provider,deviceName);

@override
String toString() {
  return 'SignInEvent.providerPressed(provider: $provider, deviceName: $deviceName)';
}


}

/// @nodoc
abstract mixin class $SignInProviderPressedEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInProviderPressedEventCopyWith(SignInProviderPressedEvent value, $Res Function(SignInProviderPressedEvent) _then) = _$SignInProviderPressedEventCopyWithImpl;
@useResult
$Res call({
 ProviderKind provider, String deviceName
});




}
/// @nodoc
class _$SignInProviderPressedEventCopyWithImpl<$Res>
    implements $SignInProviderPressedEventCopyWith<$Res> {
  _$SignInProviderPressedEventCopyWithImpl(this._self, this._then);

  final SignInProviderPressedEvent _self;
  final $Res Function(SignInProviderPressedEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? provider = null,Object? deviceName = null,}) {
  return _then(SignInProviderPressedEvent(
provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as ProviderKind,deviceName: null == deviceName ? _self.deviceName : deviceName // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInGithubOpenedEvent implements SignInEvent {
  const SignInGithubOpenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInGithubOpenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SignInEvent.githubOpened()';
}


}




/// @nodoc


class SignInProviderCancelledEvent implements SignInEvent {
  const SignInProviderCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInProviderCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SignInEvent.providerCancelled()';
}


}




/// @nodoc


class SignInUsernameChosenEvent implements SignInEvent {
  const SignInUsernameChosenEvent(this.username);
  

 final  String username;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInUsernameChosenEventCopyWith<SignInUsernameChosenEvent> get copyWith => _$SignInUsernameChosenEventCopyWithImpl<SignInUsernameChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInUsernameChosenEvent&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'SignInEvent.usernameChosen(username: $username)';
}


}

/// @nodoc
abstract mixin class $SignInUsernameChosenEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInUsernameChosenEventCopyWith(SignInUsernameChosenEvent value, $Res Function(SignInUsernameChosenEvent) _then) = _$SignInUsernameChosenEventCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$SignInUsernameChosenEventCopyWithImpl<$Res>
    implements $SignInUsernameChosenEventCopyWith<$Res> {
  _$SignInUsernameChosenEventCopyWithImpl(this._self, this._then);

  final SignInUsernameChosenEvent _self;
  final $Res Function(SignInUsernameChosenEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(SignInUsernameChosenEvent(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SignInBindingAnsweredEvent implements SignInEvent {
  const SignInBindingAnsweredEvent(this.answer);
  

 final  BindingAnswer? answer;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInBindingAnsweredEventCopyWith<SignInBindingAnsweredEvent> get copyWith => _$SignInBindingAnsweredEventCopyWithImpl<SignInBindingAnsweredEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInBindingAnsweredEvent&&(identical(other.answer, answer) || other.answer == answer));
}


@override
int get hashCode => Object.hash(runtimeType,answer);

@override
String toString() {
  return 'SignInEvent.bindingAnswered(answer: $answer)';
}


}

/// @nodoc
abstract mixin class $SignInBindingAnsweredEventCopyWith<$Res> implements $SignInEventCopyWith<$Res> {
  factory $SignInBindingAnsweredEventCopyWith(SignInBindingAnsweredEvent value, $Res Function(SignInBindingAnsweredEvent) _then) = _$SignInBindingAnsweredEventCopyWithImpl;
@useResult
$Res call({
 BindingAnswer? answer
});




}
/// @nodoc
class _$SignInBindingAnsweredEventCopyWithImpl<$Res>
    implements $SignInBindingAnsweredEventCopyWith<$Res> {
  _$SignInBindingAnsweredEventCopyWithImpl(this._self, this._then);

  final SignInBindingAnsweredEvent _self;
  final $Res Function(SignInBindingAnsweredEvent) _then;

/// Create a copy of SignInEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? answer = freezed,}) {
  return _then(SignInBindingAnsweredEvent(
freezed == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as BindingAnswer?,
  ));
}


}

/// @nodoc
mixin _$SignInState {

/// The server in use: the default one until another is typed.
 String get server;/// The server a new account is made on unless another is typed.
 String get defaultServer;/// Whether the field for another server is open.
 bool get editingServer;/// Whether what the server offers has been read.
 LoadStatus get methodsStatus;/// What the server offers; null until it is read.
 ServerMethods? get methods; SignInMode get mode;/// What is running, if something is.
 SignInAction? get busy;/// What is wrong with what was typed.
 FormProblem? get problem;/// Why the last call failed.
 ApplicationError? get error;/// The code to type at GitHub while its flow waits.
 ProviderCode? get code;/// The username the library suggests for a provider's new identity;
/// non-null while the screen asks for one.
 String? get suggestedUsername;/// The question about the notes on this phone, while it waits.
 BindingQuestion? get question;/// Whether what is being done makes an account (not signs in to one).
 bool get creating;/// The recovery key of the account just made, to write down.
 RecoveryKeyText? get recoveryKey;/// How the flow ended; null while it has not.
 AccountOutcome? get outcome;
/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SignInStateCopyWith<SignInState> get copyWith => _$SignInStateCopyWithImpl<SignInState>(this as SignInState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SignInState&&(identical(other.server, server) || other.server == server)&&(identical(other.defaultServer, defaultServer) || other.defaultServer == defaultServer)&&(identical(other.editingServer, editingServer) || other.editingServer == editingServer)&&(identical(other.methodsStatus, methodsStatus) || other.methodsStatus == methodsStatus)&&(identical(other.methods, methods) || other.methods == methods)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.code, code) || other.code == code)&&(identical(other.suggestedUsername, suggestedUsername) || other.suggestedUsername == suggestedUsername)&&(identical(other.question, question) || other.question == question)&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.recoveryKey, recoveryKey) || other.recoveryKey == recoveryKey)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,server,defaultServer,editingServer,methodsStatus,methods,mode,busy,problem,error,code,suggestedUsername,question,creating,recoveryKey,outcome);

@override
String toString() {
  return 'SignInState(server: $server, defaultServer: $defaultServer, editingServer: $editingServer, methodsStatus: $methodsStatus, methods: $methods, mode: $mode, busy: $busy, problem: $problem, error: $error, code: $code, suggestedUsername: $suggestedUsername, question: $question, creating: $creating, recoveryKey: $recoveryKey, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class $SignInStateCopyWith<$Res>  {
  factory $SignInStateCopyWith(SignInState value, $Res Function(SignInState) _then) = _$SignInStateCopyWithImpl;
@useResult
$Res call({
 String server, String defaultServer, bool editingServer, LoadStatus methodsStatus, ServerMethods? methods, SignInMode mode, SignInAction? busy, FormProblem? problem, ApplicationError? error, ProviderCode? code, String? suggestedUsername, BindingQuestion? question, bool creating, RecoveryKeyText? recoveryKey, AccountOutcome? outcome
});


$ApplicationErrorCopyWith<$Res>? get error;$ProviderCodeCopyWith<$Res>? get code;

}
/// @nodoc
class _$SignInStateCopyWithImpl<$Res>
    implements $SignInStateCopyWith<$Res> {
  _$SignInStateCopyWithImpl(this._self, this._then);

  final SignInState _self;
  final $Res Function(SignInState) _then;

/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? server = null,Object? defaultServer = null,Object? editingServer = null,Object? methodsStatus = null,Object? methods = freezed,Object? mode = null,Object? busy = freezed,Object? problem = freezed,Object? error = freezed,Object? code = freezed,Object? suggestedUsername = freezed,Object? question = freezed,Object? creating = null,Object? recoveryKey = freezed,Object? outcome = freezed,}) {
  return _then(_self.copyWith(
server: null == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String,defaultServer: null == defaultServer ? _self.defaultServer : defaultServer // ignore: cast_nullable_to_non_nullable
as String,editingServer: null == editingServer ? _self.editingServer : editingServer // ignore: cast_nullable_to_non_nullable
as bool,methodsStatus: null == methodsStatus ? _self.methodsStatus : methodsStatus // ignore: cast_nullable_to_non_nullable
as LoadStatus,methods: freezed == methods ? _self.methods : methods // ignore: cast_nullable_to_non_nullable
as ServerMethods?,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as SignInMode,busy: freezed == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as SignInAction?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,code: freezed == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as ProviderCode?,suggestedUsername: freezed == suggestedUsername ? _self.suggestedUsername : suggestedUsername // ignore: cast_nullable_to_non_nullable
as String?,question: freezed == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as BindingQuestion?,creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,recoveryKey: freezed == recoveryKey ? _self.recoveryKey : recoveryKey // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}
/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ProviderCodeCopyWith<$Res>? get code {
    if (_self.code == null) {
    return null;
  }

  return $ProviderCodeCopyWith<$Res>(_self.code!, (value) {
    return _then(_self.copyWith(code: value));
  });
}
}


/// Adds pattern-matching-related methods to [SignInState].
extension SignInStatePatterns on SignInState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SignInState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SignInState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SignInState value)  $default,){
final _that = this;
switch (_that) {
case _SignInState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SignInState value)?  $default,){
final _that = this;
switch (_that) {
case _SignInState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String server,  String defaultServer,  bool editingServer,  LoadStatus methodsStatus,  ServerMethods? methods,  SignInMode mode,  SignInAction? busy,  FormProblem? problem,  ApplicationError? error,  ProviderCode? code,  String? suggestedUsername,  BindingQuestion? question,  bool creating,  RecoveryKeyText? recoveryKey,  AccountOutcome? outcome)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SignInState() when $default != null:
return $default(_that.server,_that.defaultServer,_that.editingServer,_that.methodsStatus,_that.methods,_that.mode,_that.busy,_that.problem,_that.error,_that.code,_that.suggestedUsername,_that.question,_that.creating,_that.recoveryKey,_that.outcome);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String server,  String defaultServer,  bool editingServer,  LoadStatus methodsStatus,  ServerMethods? methods,  SignInMode mode,  SignInAction? busy,  FormProblem? problem,  ApplicationError? error,  ProviderCode? code,  String? suggestedUsername,  BindingQuestion? question,  bool creating,  RecoveryKeyText? recoveryKey,  AccountOutcome? outcome)  $default,) {final _that = this;
switch (_that) {
case _SignInState():
return $default(_that.server,_that.defaultServer,_that.editingServer,_that.methodsStatus,_that.methods,_that.mode,_that.busy,_that.problem,_that.error,_that.code,_that.suggestedUsername,_that.question,_that.creating,_that.recoveryKey,_that.outcome);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String server,  String defaultServer,  bool editingServer,  LoadStatus methodsStatus,  ServerMethods? methods,  SignInMode mode,  SignInAction? busy,  FormProblem? problem,  ApplicationError? error,  ProviderCode? code,  String? suggestedUsername,  BindingQuestion? question,  bool creating,  RecoveryKeyText? recoveryKey,  AccountOutcome? outcome)?  $default,) {final _that = this;
switch (_that) {
case _SignInState() when $default != null:
return $default(_that.server,_that.defaultServer,_that.editingServer,_that.methodsStatus,_that.methods,_that.mode,_that.busy,_that.problem,_that.error,_that.code,_that.suggestedUsername,_that.question,_that.creating,_that.recoveryKey,_that.outcome);case _:
  return null;

}
}

}

/// @nodoc


class _SignInState extends SignInState {
  const _SignInState({this.server = '', this.defaultServer = '', this.editingServer = false, this.methodsStatus = LoadStatus.loading, this.methods, this.mode = SignInMode.signIn, this.busy, this.problem, this.error, this.code, this.suggestedUsername, this.question, this.creating = false, this.recoveryKey, this.outcome}): super._();
  

/// The server in use: the default one until another is typed.
@override@JsonKey() final  String server;
/// The server a new account is made on unless another is typed.
@override@JsonKey() final  String defaultServer;
/// Whether the field for another server is open.
@override@JsonKey() final  bool editingServer;
/// Whether what the server offers has been read.
@override@JsonKey() final  LoadStatus methodsStatus;
/// What the server offers; null until it is read.
@override final  ServerMethods? methods;
@override@JsonKey() final  SignInMode mode;
/// What is running, if something is.
@override final  SignInAction? busy;
/// What is wrong with what was typed.
@override final  FormProblem? problem;
/// Why the last call failed.
@override final  ApplicationError? error;
/// The code to type at GitHub while its flow waits.
@override final  ProviderCode? code;
/// The username the library suggests for a provider's new identity;
/// non-null while the screen asks for one.
@override final  String? suggestedUsername;
/// The question about the notes on this phone, while it waits.
@override final  BindingQuestion? question;
/// Whether what is being done makes an account (not signs in to one).
@override@JsonKey() final  bool creating;
/// The recovery key of the account just made, to write down.
@override final  RecoveryKeyText? recoveryKey;
/// How the flow ended; null while it has not.
@override final  AccountOutcome? outcome;

/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SignInStateCopyWith<_SignInState> get copyWith => __$SignInStateCopyWithImpl<_SignInState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SignInState&&(identical(other.server, server) || other.server == server)&&(identical(other.defaultServer, defaultServer) || other.defaultServer == defaultServer)&&(identical(other.editingServer, editingServer) || other.editingServer == editingServer)&&(identical(other.methodsStatus, methodsStatus) || other.methodsStatus == methodsStatus)&&(identical(other.methods, methods) || other.methods == methods)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.code, code) || other.code == code)&&(identical(other.suggestedUsername, suggestedUsername) || other.suggestedUsername == suggestedUsername)&&(identical(other.question, question) || other.question == question)&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.recoveryKey, recoveryKey) || other.recoveryKey == recoveryKey)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,server,defaultServer,editingServer,methodsStatus,methods,mode,busy,problem,error,code,suggestedUsername,question,creating,recoveryKey,outcome);

@override
String toString() {
  return 'SignInState(server: $server, defaultServer: $defaultServer, editingServer: $editingServer, methodsStatus: $methodsStatus, methods: $methods, mode: $mode, busy: $busy, problem: $problem, error: $error, code: $code, suggestedUsername: $suggestedUsername, question: $question, creating: $creating, recoveryKey: $recoveryKey, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class _$SignInStateCopyWith<$Res> implements $SignInStateCopyWith<$Res> {
  factory _$SignInStateCopyWith(_SignInState value, $Res Function(_SignInState) _then) = __$SignInStateCopyWithImpl;
@override @useResult
$Res call({
 String server, String defaultServer, bool editingServer, LoadStatus methodsStatus, ServerMethods? methods, SignInMode mode, SignInAction? busy, FormProblem? problem, ApplicationError? error, ProviderCode? code, String? suggestedUsername, BindingQuestion? question, bool creating, RecoveryKeyText? recoveryKey, AccountOutcome? outcome
});


@override $ApplicationErrorCopyWith<$Res>? get error;@override $ProviderCodeCopyWith<$Res>? get code;

}
/// @nodoc
class __$SignInStateCopyWithImpl<$Res>
    implements _$SignInStateCopyWith<$Res> {
  __$SignInStateCopyWithImpl(this._self, this._then);

  final _SignInState _self;
  final $Res Function(_SignInState) _then;

/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? server = null,Object? defaultServer = null,Object? editingServer = null,Object? methodsStatus = null,Object? methods = freezed,Object? mode = null,Object? busy = freezed,Object? problem = freezed,Object? error = freezed,Object? code = freezed,Object? suggestedUsername = freezed,Object? question = freezed,Object? creating = null,Object? recoveryKey = freezed,Object? outcome = freezed,}) {
  return _then(_SignInState(
server: null == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String,defaultServer: null == defaultServer ? _self.defaultServer : defaultServer // ignore: cast_nullable_to_non_nullable
as String,editingServer: null == editingServer ? _self.editingServer : editingServer // ignore: cast_nullable_to_non_nullable
as bool,methodsStatus: null == methodsStatus ? _self.methodsStatus : methodsStatus // ignore: cast_nullable_to_non_nullable
as LoadStatus,methods: freezed == methods ? _self.methods : methods // ignore: cast_nullable_to_non_nullable
as ServerMethods?,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as SignInMode,busy: freezed == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as SignInAction?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,code: freezed == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as ProviderCode?,suggestedUsername: freezed == suggestedUsername ? _self.suggestedUsername : suggestedUsername // ignore: cast_nullable_to_non_nullable
as String?,question: freezed == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as BindingQuestion?,creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,recoveryKey: freezed == recoveryKey ? _self.recoveryKey : recoveryKey // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}

/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}/// Create a copy of SignInState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ProviderCodeCopyWith<$Res>? get code {
    if (_self.code == null) {
    return null;
  }

  return $ProviderCodeCopyWith<$Res>(_self.code!, (value) {
    return _then(_self.copyWith(code: value));
  });
}
}

// dart format on
