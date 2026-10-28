// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'account_details_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AccountDetailsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountDetailsEvent()';
}


}

/// @nodoc
class $AccountDetailsEventCopyWith<$Res>  {
$AccountDetailsEventCopyWith(AccountDetailsEvent _, $Res Function(AccountDetailsEvent) __);
}


/// Adds pattern-matching-related methods to [AccountDetailsEvent].
extension AccountDetailsEventPatterns on AccountDetailsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AccountDetailsStartedEvent value)?  started,TResult Function( AccountDetailsDeviceRevokedEvent value)?  deviceRevoked,TResult Function( AccountDetailsSignedOutEvent value)?  signedOut,TResult Function( AccountDetailsPasswordChangedEvent value)?  passwordChanged,TResult Function( AccountDetailsRecoveryKeyRequestedEvent value)?  recoveryKeyRequested,TResult Function( AccountDetailsAccountDeletedEvent value)?  accountDeleted,TResult Function( AccountDetailsPasskeyAddedEvent value)?  passkeyAdded,TResult Function( AccountDetailsPasskeyRemovedEvent value)?  passkeyRemoved,TResult Function( AccountDetailsProviderLinkedEvent value)?  providerLinked,TResult Function( AccountDetailsProviderLinkCancelledEvent value)?  providerLinkCancelled,TResult Function( AccountDetailsProviderUnlinkedEvent value)?  providerUnlinked,TResult Function( AccountDetailsKeyShownEvent value)?  keyShown,TResult Function( AccountDetailsMessageDismissedEvent value)?  messageDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AccountDetailsStartedEvent() when started != null:
return started(_that);case AccountDetailsDeviceRevokedEvent() when deviceRevoked != null:
return deviceRevoked(_that);case AccountDetailsSignedOutEvent() when signedOut != null:
return signedOut(_that);case AccountDetailsPasswordChangedEvent() when passwordChanged != null:
return passwordChanged(_that);case AccountDetailsRecoveryKeyRequestedEvent() when recoveryKeyRequested != null:
return recoveryKeyRequested(_that);case AccountDetailsAccountDeletedEvent() when accountDeleted != null:
return accountDeleted(_that);case AccountDetailsPasskeyAddedEvent() when passkeyAdded != null:
return passkeyAdded(_that);case AccountDetailsPasskeyRemovedEvent() when passkeyRemoved != null:
return passkeyRemoved(_that);case AccountDetailsProviderLinkedEvent() when providerLinked != null:
return providerLinked(_that);case AccountDetailsProviderLinkCancelledEvent() when providerLinkCancelled != null:
return providerLinkCancelled(_that);case AccountDetailsProviderUnlinkedEvent() when providerUnlinked != null:
return providerUnlinked(_that);case AccountDetailsKeyShownEvent() when keyShown != null:
return keyShown(_that);case AccountDetailsMessageDismissedEvent() when messageDismissed != null:
return messageDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AccountDetailsStartedEvent value)  started,required TResult Function( AccountDetailsDeviceRevokedEvent value)  deviceRevoked,required TResult Function( AccountDetailsSignedOutEvent value)  signedOut,required TResult Function( AccountDetailsPasswordChangedEvent value)  passwordChanged,required TResult Function( AccountDetailsRecoveryKeyRequestedEvent value)  recoveryKeyRequested,required TResult Function( AccountDetailsAccountDeletedEvent value)  accountDeleted,required TResult Function( AccountDetailsPasskeyAddedEvent value)  passkeyAdded,required TResult Function( AccountDetailsPasskeyRemovedEvent value)  passkeyRemoved,required TResult Function( AccountDetailsProviderLinkedEvent value)  providerLinked,required TResult Function( AccountDetailsProviderLinkCancelledEvent value)  providerLinkCancelled,required TResult Function( AccountDetailsProviderUnlinkedEvent value)  providerUnlinked,required TResult Function( AccountDetailsKeyShownEvent value)  keyShown,required TResult Function( AccountDetailsMessageDismissedEvent value)  messageDismissed,}){
final _that = this;
switch (_that) {
case AccountDetailsStartedEvent():
return started(_that);case AccountDetailsDeviceRevokedEvent():
return deviceRevoked(_that);case AccountDetailsSignedOutEvent():
return signedOut(_that);case AccountDetailsPasswordChangedEvent():
return passwordChanged(_that);case AccountDetailsRecoveryKeyRequestedEvent():
return recoveryKeyRequested(_that);case AccountDetailsAccountDeletedEvent():
return accountDeleted(_that);case AccountDetailsPasskeyAddedEvent():
return passkeyAdded(_that);case AccountDetailsPasskeyRemovedEvent():
return passkeyRemoved(_that);case AccountDetailsProviderLinkedEvent():
return providerLinked(_that);case AccountDetailsProviderLinkCancelledEvent():
return providerLinkCancelled(_that);case AccountDetailsProviderUnlinkedEvent():
return providerUnlinked(_that);case AccountDetailsKeyShownEvent():
return keyShown(_that);case AccountDetailsMessageDismissedEvent():
return messageDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AccountDetailsStartedEvent value)?  started,TResult? Function( AccountDetailsDeviceRevokedEvent value)?  deviceRevoked,TResult? Function( AccountDetailsSignedOutEvent value)?  signedOut,TResult? Function( AccountDetailsPasswordChangedEvent value)?  passwordChanged,TResult? Function( AccountDetailsRecoveryKeyRequestedEvent value)?  recoveryKeyRequested,TResult? Function( AccountDetailsAccountDeletedEvent value)?  accountDeleted,TResult? Function( AccountDetailsPasskeyAddedEvent value)?  passkeyAdded,TResult? Function( AccountDetailsPasskeyRemovedEvent value)?  passkeyRemoved,TResult? Function( AccountDetailsProviderLinkedEvent value)?  providerLinked,TResult? Function( AccountDetailsProviderLinkCancelledEvent value)?  providerLinkCancelled,TResult? Function( AccountDetailsProviderUnlinkedEvent value)?  providerUnlinked,TResult? Function( AccountDetailsKeyShownEvent value)?  keyShown,TResult? Function( AccountDetailsMessageDismissedEvent value)?  messageDismissed,}){
final _that = this;
switch (_that) {
case AccountDetailsStartedEvent() when started != null:
return started(_that);case AccountDetailsDeviceRevokedEvent() when deviceRevoked != null:
return deviceRevoked(_that);case AccountDetailsSignedOutEvent() when signedOut != null:
return signedOut(_that);case AccountDetailsPasswordChangedEvent() when passwordChanged != null:
return passwordChanged(_that);case AccountDetailsRecoveryKeyRequestedEvent() when recoveryKeyRequested != null:
return recoveryKeyRequested(_that);case AccountDetailsAccountDeletedEvent() when accountDeleted != null:
return accountDeleted(_that);case AccountDetailsPasskeyAddedEvent() when passkeyAdded != null:
return passkeyAdded(_that);case AccountDetailsPasskeyRemovedEvent() when passkeyRemoved != null:
return passkeyRemoved(_that);case AccountDetailsProviderLinkedEvent() when providerLinked != null:
return providerLinked(_that);case AccountDetailsProviderLinkCancelledEvent() when providerLinkCancelled != null:
return providerLinkCancelled(_that);case AccountDetailsProviderUnlinkedEvent() when providerUnlinked != null:
return providerUnlinked(_that);case AccountDetailsKeyShownEvent() when keyShown != null:
return keyShown(_that);case AccountDetailsMessageDismissedEvent() when messageDismissed != null:
return messageDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( String deviceId)?  deviceRevoked,TResult Function()?  signedOut,TResult Function( String current,  String newPassword,  String repeat)?  passwordChanged,TResult Function( String currentPassword)?  recoveryKeyRequested,TResult Function( String password,  String typedUsername)?  accountDeleted,TResult Function( String currentPassword)?  passkeyAdded,TResult Function( String id,  String currentPassword)?  passkeyRemoved,TResult Function( ProviderKind provider,  String currentPassword)?  providerLinked,TResult Function()?  providerLinkCancelled,TResult Function( ProviderKind provider,  String currentPassword)?  providerUnlinked,TResult Function()?  keyShown,TResult Function()?  messageDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AccountDetailsStartedEvent() when started != null:
return started();case AccountDetailsDeviceRevokedEvent() when deviceRevoked != null:
return deviceRevoked(_that.deviceId);case AccountDetailsSignedOutEvent() when signedOut != null:
return signedOut();case AccountDetailsPasswordChangedEvent() when passwordChanged != null:
return passwordChanged(_that.current,_that.newPassword,_that.repeat);case AccountDetailsRecoveryKeyRequestedEvent() when recoveryKeyRequested != null:
return recoveryKeyRequested(_that.currentPassword);case AccountDetailsAccountDeletedEvent() when accountDeleted != null:
return accountDeleted(_that.password,_that.typedUsername);case AccountDetailsPasskeyAddedEvent() when passkeyAdded != null:
return passkeyAdded(_that.currentPassword);case AccountDetailsPasskeyRemovedEvent() when passkeyRemoved != null:
return passkeyRemoved(_that.id,_that.currentPassword);case AccountDetailsProviderLinkedEvent() when providerLinked != null:
return providerLinked(_that.provider,_that.currentPassword);case AccountDetailsProviderLinkCancelledEvent() when providerLinkCancelled != null:
return providerLinkCancelled();case AccountDetailsProviderUnlinkedEvent() when providerUnlinked != null:
return providerUnlinked(_that.provider,_that.currentPassword);case AccountDetailsKeyShownEvent() when keyShown != null:
return keyShown();case AccountDetailsMessageDismissedEvent() when messageDismissed != null:
return messageDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( String deviceId)  deviceRevoked,required TResult Function()  signedOut,required TResult Function( String current,  String newPassword,  String repeat)  passwordChanged,required TResult Function( String currentPassword)  recoveryKeyRequested,required TResult Function( String password,  String typedUsername)  accountDeleted,required TResult Function( String currentPassword)  passkeyAdded,required TResult Function( String id,  String currentPassword)  passkeyRemoved,required TResult Function( ProviderKind provider,  String currentPassword)  providerLinked,required TResult Function()  providerLinkCancelled,required TResult Function( ProviderKind provider,  String currentPassword)  providerUnlinked,required TResult Function()  keyShown,required TResult Function()  messageDismissed,}) {final _that = this;
switch (_that) {
case AccountDetailsStartedEvent():
return started();case AccountDetailsDeviceRevokedEvent():
return deviceRevoked(_that.deviceId);case AccountDetailsSignedOutEvent():
return signedOut();case AccountDetailsPasswordChangedEvent():
return passwordChanged(_that.current,_that.newPassword,_that.repeat);case AccountDetailsRecoveryKeyRequestedEvent():
return recoveryKeyRequested(_that.currentPassword);case AccountDetailsAccountDeletedEvent():
return accountDeleted(_that.password,_that.typedUsername);case AccountDetailsPasskeyAddedEvent():
return passkeyAdded(_that.currentPassword);case AccountDetailsPasskeyRemovedEvent():
return passkeyRemoved(_that.id,_that.currentPassword);case AccountDetailsProviderLinkedEvent():
return providerLinked(_that.provider,_that.currentPassword);case AccountDetailsProviderLinkCancelledEvent():
return providerLinkCancelled();case AccountDetailsProviderUnlinkedEvent():
return providerUnlinked(_that.provider,_that.currentPassword);case AccountDetailsKeyShownEvent():
return keyShown();case AccountDetailsMessageDismissedEvent():
return messageDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( String deviceId)?  deviceRevoked,TResult? Function()?  signedOut,TResult? Function( String current,  String newPassword,  String repeat)?  passwordChanged,TResult? Function( String currentPassword)?  recoveryKeyRequested,TResult? Function( String password,  String typedUsername)?  accountDeleted,TResult? Function( String currentPassword)?  passkeyAdded,TResult? Function( String id,  String currentPassword)?  passkeyRemoved,TResult? Function( ProviderKind provider,  String currentPassword)?  providerLinked,TResult? Function()?  providerLinkCancelled,TResult? Function( ProviderKind provider,  String currentPassword)?  providerUnlinked,TResult? Function()?  keyShown,TResult? Function()?  messageDismissed,}) {final _that = this;
switch (_that) {
case AccountDetailsStartedEvent() when started != null:
return started();case AccountDetailsDeviceRevokedEvent() when deviceRevoked != null:
return deviceRevoked(_that.deviceId);case AccountDetailsSignedOutEvent() when signedOut != null:
return signedOut();case AccountDetailsPasswordChangedEvent() when passwordChanged != null:
return passwordChanged(_that.current,_that.newPassword,_that.repeat);case AccountDetailsRecoveryKeyRequestedEvent() when recoveryKeyRequested != null:
return recoveryKeyRequested(_that.currentPassword);case AccountDetailsAccountDeletedEvent() when accountDeleted != null:
return accountDeleted(_that.password,_that.typedUsername);case AccountDetailsPasskeyAddedEvent() when passkeyAdded != null:
return passkeyAdded(_that.currentPassword);case AccountDetailsPasskeyRemovedEvent() when passkeyRemoved != null:
return passkeyRemoved(_that.id,_that.currentPassword);case AccountDetailsProviderLinkedEvent() when providerLinked != null:
return providerLinked(_that.provider,_that.currentPassword);case AccountDetailsProviderLinkCancelledEvent() when providerLinkCancelled != null:
return providerLinkCancelled();case AccountDetailsProviderUnlinkedEvent() when providerUnlinked != null:
return providerUnlinked(_that.provider,_that.currentPassword);case AccountDetailsKeyShownEvent() when keyShown != null:
return keyShown();case AccountDetailsMessageDismissedEvent() when messageDismissed != null:
return messageDismissed();case _:
  return null;

}
}

}

/// @nodoc


class AccountDetailsStartedEvent implements AccountDetailsEvent {
  const AccountDetailsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountDetailsEvent.started()';
}


}




/// @nodoc


class AccountDetailsDeviceRevokedEvent implements AccountDetailsEvent {
  const AccountDetailsDeviceRevokedEvent(this.deviceId);
  

 final  String deviceId;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsDeviceRevokedEventCopyWith<AccountDetailsDeviceRevokedEvent> get copyWith => _$AccountDetailsDeviceRevokedEventCopyWithImpl<AccountDetailsDeviceRevokedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsDeviceRevokedEvent&&(identical(other.deviceId, deviceId) || other.deviceId == deviceId));
}


@override
int get hashCode => Object.hash(runtimeType,deviceId);

@override
String toString() {
  return 'AccountDetailsEvent.deviceRevoked(deviceId: $deviceId)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsDeviceRevokedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsDeviceRevokedEventCopyWith(AccountDetailsDeviceRevokedEvent value, $Res Function(AccountDetailsDeviceRevokedEvent) _then) = _$AccountDetailsDeviceRevokedEventCopyWithImpl;
@useResult
$Res call({
 String deviceId
});




}
/// @nodoc
class _$AccountDetailsDeviceRevokedEventCopyWithImpl<$Res>
    implements $AccountDetailsDeviceRevokedEventCopyWith<$Res> {
  _$AccountDetailsDeviceRevokedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsDeviceRevokedEvent _self;
  final $Res Function(AccountDetailsDeviceRevokedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deviceId = null,}) {
  return _then(AccountDetailsDeviceRevokedEvent(
null == deviceId ? _self.deviceId : deviceId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsSignedOutEvent implements AccountDetailsEvent {
  const AccountDetailsSignedOutEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsSignedOutEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountDetailsEvent.signedOut()';
}


}




/// @nodoc


class AccountDetailsPasswordChangedEvent implements AccountDetailsEvent {
  const AccountDetailsPasswordChangedEvent({required this.current, required this.newPassword, required this.repeat});
  

 final  String current;
 final  String newPassword;
 final  String repeat;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsPasswordChangedEventCopyWith<AccountDetailsPasswordChangedEvent> get copyWith => _$AccountDetailsPasswordChangedEventCopyWithImpl<AccountDetailsPasswordChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsPasswordChangedEvent&&(identical(other.current, current) || other.current == current)&&(identical(other.newPassword, newPassword) || other.newPassword == newPassword)&&(identical(other.repeat, repeat) || other.repeat == repeat));
}


@override
int get hashCode => Object.hash(runtimeType,current,newPassword,repeat);

@override
String toString() {
  return 'AccountDetailsEvent.passwordChanged(current: $current, newPassword: $newPassword, repeat: $repeat)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsPasswordChangedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsPasswordChangedEventCopyWith(AccountDetailsPasswordChangedEvent value, $Res Function(AccountDetailsPasswordChangedEvent) _then) = _$AccountDetailsPasswordChangedEventCopyWithImpl;
@useResult
$Res call({
 String current, String newPassword, String repeat
});




}
/// @nodoc
class _$AccountDetailsPasswordChangedEventCopyWithImpl<$Res>
    implements $AccountDetailsPasswordChangedEventCopyWith<$Res> {
  _$AccountDetailsPasswordChangedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsPasswordChangedEvent _self;
  final $Res Function(AccountDetailsPasswordChangedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? current = null,Object? newPassword = null,Object? repeat = null,}) {
  return _then(AccountDetailsPasswordChangedEvent(
current: null == current ? _self.current : current // ignore: cast_nullable_to_non_nullable
as String,newPassword: null == newPassword ? _self.newPassword : newPassword // ignore: cast_nullable_to_non_nullable
as String,repeat: null == repeat ? _self.repeat : repeat // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsRecoveryKeyRequestedEvent implements AccountDetailsEvent {
  const AccountDetailsRecoveryKeyRequestedEvent({required this.currentPassword});
  

 final  String currentPassword;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsRecoveryKeyRequestedEventCopyWith<AccountDetailsRecoveryKeyRequestedEvent> get copyWith => _$AccountDetailsRecoveryKeyRequestedEventCopyWithImpl<AccountDetailsRecoveryKeyRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsRecoveryKeyRequestedEvent&&(identical(other.currentPassword, currentPassword) || other.currentPassword == currentPassword));
}


@override
int get hashCode => Object.hash(runtimeType,currentPassword);

@override
String toString() {
  return 'AccountDetailsEvent.recoveryKeyRequested(currentPassword: $currentPassword)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsRecoveryKeyRequestedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsRecoveryKeyRequestedEventCopyWith(AccountDetailsRecoveryKeyRequestedEvent value, $Res Function(AccountDetailsRecoveryKeyRequestedEvent) _then) = _$AccountDetailsRecoveryKeyRequestedEventCopyWithImpl;
@useResult
$Res call({
 String currentPassword
});




}
/// @nodoc
class _$AccountDetailsRecoveryKeyRequestedEventCopyWithImpl<$Res>
    implements $AccountDetailsRecoveryKeyRequestedEventCopyWith<$Res> {
  _$AccountDetailsRecoveryKeyRequestedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsRecoveryKeyRequestedEvent _self;
  final $Res Function(AccountDetailsRecoveryKeyRequestedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? currentPassword = null,}) {
  return _then(AccountDetailsRecoveryKeyRequestedEvent(
currentPassword: null == currentPassword ? _self.currentPassword : currentPassword // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsAccountDeletedEvent implements AccountDetailsEvent {
  const AccountDetailsAccountDeletedEvent({required this.password, required this.typedUsername});
  

 final  String password;
 final  String typedUsername;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsAccountDeletedEventCopyWith<AccountDetailsAccountDeletedEvent> get copyWith => _$AccountDetailsAccountDeletedEventCopyWithImpl<AccountDetailsAccountDeletedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsAccountDeletedEvent&&(identical(other.password, password) || other.password == password)&&(identical(other.typedUsername, typedUsername) || other.typedUsername == typedUsername));
}


@override
int get hashCode => Object.hash(runtimeType,password,typedUsername);

@override
String toString() {
  return 'AccountDetailsEvent.accountDeleted(password: $password, typedUsername: $typedUsername)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsAccountDeletedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsAccountDeletedEventCopyWith(AccountDetailsAccountDeletedEvent value, $Res Function(AccountDetailsAccountDeletedEvent) _then) = _$AccountDetailsAccountDeletedEventCopyWithImpl;
@useResult
$Res call({
 String password, String typedUsername
});




}
/// @nodoc
class _$AccountDetailsAccountDeletedEventCopyWithImpl<$Res>
    implements $AccountDetailsAccountDeletedEventCopyWith<$Res> {
  _$AccountDetailsAccountDeletedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsAccountDeletedEvent _self;
  final $Res Function(AccountDetailsAccountDeletedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = null,Object? typedUsername = null,}) {
  return _then(AccountDetailsAccountDeletedEvent(
password: null == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String,typedUsername: null == typedUsername ? _self.typedUsername : typedUsername // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsPasskeyAddedEvent implements AccountDetailsEvent {
  const AccountDetailsPasskeyAddedEvent({required this.currentPassword});
  

 final  String currentPassword;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsPasskeyAddedEventCopyWith<AccountDetailsPasskeyAddedEvent> get copyWith => _$AccountDetailsPasskeyAddedEventCopyWithImpl<AccountDetailsPasskeyAddedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsPasskeyAddedEvent&&(identical(other.currentPassword, currentPassword) || other.currentPassword == currentPassword));
}


@override
int get hashCode => Object.hash(runtimeType,currentPassword);

@override
String toString() {
  return 'AccountDetailsEvent.passkeyAdded(currentPassword: $currentPassword)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsPasskeyAddedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsPasskeyAddedEventCopyWith(AccountDetailsPasskeyAddedEvent value, $Res Function(AccountDetailsPasskeyAddedEvent) _then) = _$AccountDetailsPasskeyAddedEventCopyWithImpl;
@useResult
$Res call({
 String currentPassword
});




}
/// @nodoc
class _$AccountDetailsPasskeyAddedEventCopyWithImpl<$Res>
    implements $AccountDetailsPasskeyAddedEventCopyWith<$Res> {
  _$AccountDetailsPasskeyAddedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsPasskeyAddedEvent _self;
  final $Res Function(AccountDetailsPasskeyAddedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? currentPassword = null,}) {
  return _then(AccountDetailsPasskeyAddedEvent(
currentPassword: null == currentPassword ? _self.currentPassword : currentPassword // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsPasskeyRemovedEvent implements AccountDetailsEvent {
  const AccountDetailsPasskeyRemovedEvent({required this.id, required this.currentPassword});
  

 final  String id;
 final  String currentPassword;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsPasskeyRemovedEventCopyWith<AccountDetailsPasskeyRemovedEvent> get copyWith => _$AccountDetailsPasskeyRemovedEventCopyWithImpl<AccountDetailsPasskeyRemovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsPasskeyRemovedEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.currentPassword, currentPassword) || other.currentPassword == currentPassword));
}


@override
int get hashCode => Object.hash(runtimeType,id,currentPassword);

@override
String toString() {
  return 'AccountDetailsEvent.passkeyRemoved(id: $id, currentPassword: $currentPassword)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsPasskeyRemovedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsPasskeyRemovedEventCopyWith(AccountDetailsPasskeyRemovedEvent value, $Res Function(AccountDetailsPasskeyRemovedEvent) _then) = _$AccountDetailsPasskeyRemovedEventCopyWithImpl;
@useResult
$Res call({
 String id, String currentPassword
});




}
/// @nodoc
class _$AccountDetailsPasskeyRemovedEventCopyWithImpl<$Res>
    implements $AccountDetailsPasskeyRemovedEventCopyWith<$Res> {
  _$AccountDetailsPasskeyRemovedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsPasskeyRemovedEvent _self;
  final $Res Function(AccountDetailsPasskeyRemovedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? currentPassword = null,}) {
  return _then(AccountDetailsPasskeyRemovedEvent(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,currentPassword: null == currentPassword ? _self.currentPassword : currentPassword // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsProviderLinkedEvent implements AccountDetailsEvent {
  const AccountDetailsProviderLinkedEvent({required this.provider, required this.currentPassword});
  

 final  ProviderKind provider;
 final  String currentPassword;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsProviderLinkedEventCopyWith<AccountDetailsProviderLinkedEvent> get copyWith => _$AccountDetailsProviderLinkedEventCopyWithImpl<AccountDetailsProviderLinkedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsProviderLinkedEvent&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.currentPassword, currentPassword) || other.currentPassword == currentPassword));
}


@override
int get hashCode => Object.hash(runtimeType,provider,currentPassword);

@override
String toString() {
  return 'AccountDetailsEvent.providerLinked(provider: $provider, currentPassword: $currentPassword)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsProviderLinkedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsProviderLinkedEventCopyWith(AccountDetailsProviderLinkedEvent value, $Res Function(AccountDetailsProviderLinkedEvent) _then) = _$AccountDetailsProviderLinkedEventCopyWithImpl;
@useResult
$Res call({
 ProviderKind provider, String currentPassword
});




}
/// @nodoc
class _$AccountDetailsProviderLinkedEventCopyWithImpl<$Res>
    implements $AccountDetailsProviderLinkedEventCopyWith<$Res> {
  _$AccountDetailsProviderLinkedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsProviderLinkedEvent _self;
  final $Res Function(AccountDetailsProviderLinkedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? provider = null,Object? currentPassword = null,}) {
  return _then(AccountDetailsProviderLinkedEvent(
provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as ProviderKind,currentPassword: null == currentPassword ? _self.currentPassword : currentPassword // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsProviderLinkCancelledEvent implements AccountDetailsEvent {
  const AccountDetailsProviderLinkCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsProviderLinkCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountDetailsEvent.providerLinkCancelled()';
}


}




/// @nodoc


class AccountDetailsProviderUnlinkedEvent implements AccountDetailsEvent {
  const AccountDetailsProviderUnlinkedEvent({required this.provider, required this.currentPassword});
  

 final  ProviderKind provider;
 final  String currentPassword;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsProviderUnlinkedEventCopyWith<AccountDetailsProviderUnlinkedEvent> get copyWith => _$AccountDetailsProviderUnlinkedEventCopyWithImpl<AccountDetailsProviderUnlinkedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsProviderUnlinkedEvent&&(identical(other.provider, provider) || other.provider == provider)&&(identical(other.currentPassword, currentPassword) || other.currentPassword == currentPassword));
}


@override
int get hashCode => Object.hash(runtimeType,provider,currentPassword);

@override
String toString() {
  return 'AccountDetailsEvent.providerUnlinked(provider: $provider, currentPassword: $currentPassword)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsProviderUnlinkedEventCopyWith<$Res> implements $AccountDetailsEventCopyWith<$Res> {
  factory $AccountDetailsProviderUnlinkedEventCopyWith(AccountDetailsProviderUnlinkedEvent value, $Res Function(AccountDetailsProviderUnlinkedEvent) _then) = _$AccountDetailsProviderUnlinkedEventCopyWithImpl;
@useResult
$Res call({
 ProviderKind provider, String currentPassword
});




}
/// @nodoc
class _$AccountDetailsProviderUnlinkedEventCopyWithImpl<$Res>
    implements $AccountDetailsProviderUnlinkedEventCopyWith<$Res> {
  _$AccountDetailsProviderUnlinkedEventCopyWithImpl(this._self, this._then);

  final AccountDetailsProviderUnlinkedEvent _self;
  final $Res Function(AccountDetailsProviderUnlinkedEvent) _then;

/// Create a copy of AccountDetailsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? provider = null,Object? currentPassword = null,}) {
  return _then(AccountDetailsProviderUnlinkedEvent(
provider: null == provider ? _self.provider : provider // ignore: cast_nullable_to_non_nullable
as ProviderKind,currentPassword: null == currentPassword ? _self.currentPassword : currentPassword // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountDetailsKeyShownEvent implements AccountDetailsEvent {
  const AccountDetailsKeyShownEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsKeyShownEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountDetailsEvent.keyShown()';
}


}




/// @nodoc


class AccountDetailsMessageDismissedEvent implements AccountDetailsEvent {
  const AccountDetailsMessageDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsMessageDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountDetailsEvent.messageDismissed()';
}


}




/// @nodoc
mixin _$AccountDetailsState {

 LoadStatus get status;/// The account; null until it is read.
 AccountDetails? get details;/// Why the account could not be read.
 ApplicationError? get loadError;/// The change being made, if one is.
 AccountTask? get busy;/// What is wrong with what was typed.
 FormProblem? get problem;/// Why the last change failed.
 ApplicationError? get error;/// What the last change that went through did.
 AccountNotice? get notice;/// The recovery key a change made, until the page shows it.
 RecoveryKeyText? get newKey;/// The provider being linked.
 ProviderKind? get linking;/// The code to type at GitHub while its link waits.
 ProviderCode? get code;/// How the page ended; null while it has not.
 AccountOutcome? get outcome;
/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountDetailsStateCopyWith<AccountDetailsState> get copyWith => _$AccountDetailsStateCopyWithImpl<AccountDetailsState>(this as AccountDetailsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountDetailsState&&(identical(other.status, status) || other.status == status)&&(identical(other.details, details) || other.details == details)&&(identical(other.loadError, loadError) || other.loadError == loadError)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.newKey, newKey) || other.newKey == newKey)&&(identical(other.linking, linking) || other.linking == linking)&&(identical(other.code, code) || other.code == code)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,status,details,loadError,busy,problem,error,notice,newKey,linking,code,outcome);

@override
String toString() {
  return 'AccountDetailsState(status: $status, details: $details, loadError: $loadError, busy: $busy, problem: $problem, error: $error, notice: $notice, newKey: $newKey, linking: $linking, code: $code, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class $AccountDetailsStateCopyWith<$Res>  {
  factory $AccountDetailsStateCopyWith(AccountDetailsState value, $Res Function(AccountDetailsState) _then) = _$AccountDetailsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, AccountDetails? details, ApplicationError? loadError, AccountTask? busy, FormProblem? problem, ApplicationError? error, AccountNotice? notice, RecoveryKeyText? newKey, ProviderKind? linking, ProviderCode? code, AccountOutcome? outcome
});


$ApplicationErrorCopyWith<$Res>? get loadError;$ApplicationErrorCopyWith<$Res>? get error;$AccountNoticeCopyWith<$Res>? get notice;$ProviderCodeCopyWith<$Res>? get code;

}
/// @nodoc
class _$AccountDetailsStateCopyWithImpl<$Res>
    implements $AccountDetailsStateCopyWith<$Res> {
  _$AccountDetailsStateCopyWithImpl(this._self, this._then);

  final AccountDetailsState _self;
  final $Res Function(AccountDetailsState) _then;

/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? details = freezed,Object? loadError = freezed,Object? busy = freezed,Object? problem = freezed,Object? error = freezed,Object? notice = freezed,Object? newKey = freezed,Object? linking = freezed,Object? code = freezed,Object? outcome = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,details: freezed == details ? _self.details : details // ignore: cast_nullable_to_non_nullable
as AccountDetails?,loadError: freezed == loadError ? _self.loadError : loadError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,busy: freezed == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as AccountTask?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as AccountNotice?,newKey: freezed == newKey ? _self.newKey : newKey // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,linking: freezed == linking ? _self.linking : linking // ignore: cast_nullable_to_non_nullable
as ProviderKind?,code: freezed == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as ProviderCode?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}
/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get loadError {
    if (_self.loadError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.loadError!, (value) {
    return _then(_self.copyWith(loadError: value));
  });
}/// Create a copy of AccountDetailsState
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
}/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AccountNoticeCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $AccountNoticeCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}/// Create a copy of AccountDetailsState
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


/// Adds pattern-matching-related methods to [AccountDetailsState].
extension AccountDetailsStatePatterns on AccountDetailsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AccountDetailsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AccountDetailsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AccountDetailsState value)  $default,){
final _that = this;
switch (_that) {
case _AccountDetailsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AccountDetailsState value)?  $default,){
final _that = this;
switch (_that) {
case _AccountDetailsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  AccountDetails? details,  ApplicationError? loadError,  AccountTask? busy,  FormProblem? problem,  ApplicationError? error,  AccountNotice? notice,  RecoveryKeyText? newKey,  ProviderKind? linking,  ProviderCode? code,  AccountOutcome? outcome)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AccountDetailsState() when $default != null:
return $default(_that.status,_that.details,_that.loadError,_that.busy,_that.problem,_that.error,_that.notice,_that.newKey,_that.linking,_that.code,_that.outcome);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  AccountDetails? details,  ApplicationError? loadError,  AccountTask? busy,  FormProblem? problem,  ApplicationError? error,  AccountNotice? notice,  RecoveryKeyText? newKey,  ProviderKind? linking,  ProviderCode? code,  AccountOutcome? outcome)  $default,) {final _that = this;
switch (_that) {
case _AccountDetailsState():
return $default(_that.status,_that.details,_that.loadError,_that.busy,_that.problem,_that.error,_that.notice,_that.newKey,_that.linking,_that.code,_that.outcome);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  AccountDetails? details,  ApplicationError? loadError,  AccountTask? busy,  FormProblem? problem,  ApplicationError? error,  AccountNotice? notice,  RecoveryKeyText? newKey,  ProviderKind? linking,  ProviderCode? code,  AccountOutcome? outcome)?  $default,) {final _that = this;
switch (_that) {
case _AccountDetailsState() when $default != null:
return $default(_that.status,_that.details,_that.loadError,_that.busy,_that.problem,_that.error,_that.notice,_that.newKey,_that.linking,_that.code,_that.outcome);case _:
  return null;

}
}

}

/// @nodoc


class _AccountDetailsState implements AccountDetailsState {
  const _AccountDetailsState({this.status = LoadStatus.loading, this.details, this.loadError, this.busy, this.problem, this.error, this.notice, this.newKey, this.linking, this.code, this.outcome});
  

@override@JsonKey() final  LoadStatus status;
/// The account; null until it is read.
@override final  AccountDetails? details;
/// Why the account could not be read.
@override final  ApplicationError? loadError;
/// The change being made, if one is.
@override final  AccountTask? busy;
/// What is wrong with what was typed.
@override final  FormProblem? problem;
/// Why the last change failed.
@override final  ApplicationError? error;
/// What the last change that went through did.
@override final  AccountNotice? notice;
/// The recovery key a change made, until the page shows it.
@override final  RecoveryKeyText? newKey;
/// The provider being linked.
@override final  ProviderKind? linking;
/// The code to type at GitHub while its link waits.
@override final  ProviderCode? code;
/// How the page ended; null while it has not.
@override final  AccountOutcome? outcome;

/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AccountDetailsStateCopyWith<_AccountDetailsState> get copyWith => __$AccountDetailsStateCopyWithImpl<_AccountDetailsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AccountDetailsState&&(identical(other.status, status) || other.status == status)&&(identical(other.details, details) || other.details == details)&&(identical(other.loadError, loadError) || other.loadError == loadError)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.newKey, newKey) || other.newKey == newKey)&&(identical(other.linking, linking) || other.linking == linking)&&(identical(other.code, code) || other.code == code)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,status,details,loadError,busy,problem,error,notice,newKey,linking,code,outcome);

@override
String toString() {
  return 'AccountDetailsState(status: $status, details: $details, loadError: $loadError, busy: $busy, problem: $problem, error: $error, notice: $notice, newKey: $newKey, linking: $linking, code: $code, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class _$AccountDetailsStateCopyWith<$Res> implements $AccountDetailsStateCopyWith<$Res> {
  factory _$AccountDetailsStateCopyWith(_AccountDetailsState value, $Res Function(_AccountDetailsState) _then) = __$AccountDetailsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, AccountDetails? details, ApplicationError? loadError, AccountTask? busy, FormProblem? problem, ApplicationError? error, AccountNotice? notice, RecoveryKeyText? newKey, ProviderKind? linking, ProviderCode? code, AccountOutcome? outcome
});


@override $ApplicationErrorCopyWith<$Res>? get loadError;@override $ApplicationErrorCopyWith<$Res>? get error;@override $AccountNoticeCopyWith<$Res>? get notice;@override $ProviderCodeCopyWith<$Res>? get code;

}
/// @nodoc
class __$AccountDetailsStateCopyWithImpl<$Res>
    implements _$AccountDetailsStateCopyWith<$Res> {
  __$AccountDetailsStateCopyWithImpl(this._self, this._then);

  final _AccountDetailsState _self;
  final $Res Function(_AccountDetailsState) _then;

/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? details = freezed,Object? loadError = freezed,Object? busy = freezed,Object? problem = freezed,Object? error = freezed,Object? notice = freezed,Object? newKey = freezed,Object? linking = freezed,Object? code = freezed,Object? outcome = freezed,}) {
  return _then(_AccountDetailsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,details: freezed == details ? _self.details : details // ignore: cast_nullable_to_non_nullable
as AccountDetails?,loadError: freezed == loadError ? _self.loadError : loadError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,busy: freezed == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as AccountTask?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as FormProblem?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as AccountNotice?,newKey: freezed == newKey ? _self.newKey : newKey // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,linking: freezed == linking ? _self.linking : linking // ignore: cast_nullable_to_non_nullable
as ProviderKind?,code: freezed == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as ProviderCode?,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AccountOutcome?,
  ));
}

/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get loadError {
    if (_self.loadError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.loadError!, (value) {
    return _then(_self.copyWith(loadError: value));
  });
}/// Create a copy of AccountDetailsState
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
}/// Create a copy of AccountDetailsState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AccountNoticeCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $AccountNoticeCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}/// Create a copy of AccountDetailsState
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
