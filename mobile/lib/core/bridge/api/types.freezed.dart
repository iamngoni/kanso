// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AccountState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountState()';
}


}

/// @nodoc
class $AccountStateCopyWith<$Res>  {
$AccountStateCopyWith(AccountState _, $Res Function(AccountState) __);
}


/// Adds pattern-matching-related methods to [AccountState].
extension AccountStatePatterns on AccountState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AccountState_SignedOut value)?  signedOut,TResult Function( AccountState_SignedIn value)?  signedIn,TResult Function( AccountState_Locked value)?  locked,TResult Function( AccountState_Question value)?  question,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut(_that);case AccountState_SignedIn() when signedIn != null:
return signedIn(_that);case AccountState_Locked() when locked != null:
return locked(_that);case AccountState_Question() when question != null:
return question(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AccountState_SignedOut value)  signedOut,required TResult Function( AccountState_SignedIn value)  signedIn,required TResult Function( AccountState_Locked value)  locked,required TResult Function( AccountState_Question value)  question,}){
final _that = this;
switch (_that) {
case AccountState_SignedOut():
return signedOut(_that);case AccountState_SignedIn():
return signedIn(_that);case AccountState_Locked():
return locked(_that);case AccountState_Question():
return question(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AccountState_SignedOut value)?  signedOut,TResult? Function( AccountState_SignedIn value)?  signedIn,TResult? Function( AccountState_Locked value)?  locked,TResult? Function( AccountState_Question value)?  question,}){
final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut(_that);case AccountState_SignedIn() when signedIn != null:
return signedIn(_that);case AccountState_Locked() when locked != null:
return locked(_that);case AccountState_Question() when question != null:
return question(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  signedOut,TResult Function( String username,  String server)?  signedIn,TResult Function( LockedInfo field0)?  locked,TResult Function( BindingQuestion field0)?  question,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut();case AccountState_SignedIn() when signedIn != null:
return signedIn(_that.username,_that.server);case AccountState_Locked() when locked != null:
return locked(_that.field0);case AccountState_Question() when question != null:
return question(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  signedOut,required TResult Function( String username,  String server)  signedIn,required TResult Function( LockedInfo field0)  locked,required TResult Function( BindingQuestion field0)  question,}) {final _that = this;
switch (_that) {
case AccountState_SignedOut():
return signedOut();case AccountState_SignedIn():
return signedIn(_that.username,_that.server);case AccountState_Locked():
return locked(_that.field0);case AccountState_Question():
return question(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  signedOut,TResult? Function( String username,  String server)?  signedIn,TResult? Function( LockedInfo field0)?  locked,TResult? Function( BindingQuestion field0)?  question,}) {final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut();case AccountState_SignedIn() when signedIn != null:
return signedIn(_that.username,_that.server);case AccountState_Locked() when locked != null:
return locked(_that.field0);case AccountState_Question() when question != null:
return question(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class AccountState_SignedOut extends AccountState {
  const AccountState_SignedOut(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_SignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountState.signedOut()';
}


}




/// @nodoc


class AccountState_SignedIn extends AccountState {
  const AccountState_SignedIn({required this.username, required this.server}): super._();
  

 final  String username;
 final  String server;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountState_SignedInCopyWith<AccountState_SignedIn> get copyWith => _$AccountState_SignedInCopyWithImpl<AccountState_SignedIn>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_SignedIn&&(identical(other.username, username) || other.username == username)&&(identical(other.server, server) || other.server == server));
}


@override
int get hashCode => Object.hash(runtimeType,username,server);

@override
String toString() {
  return 'AccountState.signedIn(username: $username, server: $server)';
}


}

/// @nodoc
abstract mixin class $AccountState_SignedInCopyWith<$Res> implements $AccountStateCopyWith<$Res> {
  factory $AccountState_SignedInCopyWith(AccountState_SignedIn value, $Res Function(AccountState_SignedIn) _then) = _$AccountState_SignedInCopyWithImpl;
@useResult
$Res call({
 String username, String server
});




}
/// @nodoc
class _$AccountState_SignedInCopyWithImpl<$Res>
    implements $AccountState_SignedInCopyWith<$Res> {
  _$AccountState_SignedInCopyWithImpl(this._self, this._then);

  final AccountState_SignedIn _self;
  final $Res Function(AccountState_SignedIn) _then;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? server = null,}) {
  return _then(AccountState_SignedIn(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,server: null == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountState_Locked extends AccountState {
  const AccountState_Locked(this.field0): super._();
  

 final  LockedInfo field0;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountState_LockedCopyWith<AccountState_Locked> get copyWith => _$AccountState_LockedCopyWithImpl<AccountState_Locked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_Locked&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AccountState.locked(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AccountState_LockedCopyWith<$Res> implements $AccountStateCopyWith<$Res> {
  factory $AccountState_LockedCopyWith(AccountState_Locked value, $Res Function(AccountState_Locked) _then) = _$AccountState_LockedCopyWithImpl;
@useResult
$Res call({
 LockedInfo field0
});




}
/// @nodoc
class _$AccountState_LockedCopyWithImpl<$Res>
    implements $AccountState_LockedCopyWith<$Res> {
  _$AccountState_LockedCopyWithImpl(this._self, this._then);

  final AccountState_Locked _self;
  final $Res Function(AccountState_Locked) _then;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AccountState_Locked(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as LockedInfo,
  ));
}


}

/// @nodoc


class AccountState_Question extends AccountState {
  const AccountState_Question(this.field0): super._();
  

 final  BindingQuestion field0;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountState_QuestionCopyWith<AccountState_Question> get copyWith => _$AccountState_QuestionCopyWithImpl<AccountState_Question>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_Question&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AccountState.question(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AccountState_QuestionCopyWith<$Res> implements $AccountStateCopyWith<$Res> {
  factory $AccountState_QuestionCopyWith(AccountState_Question value, $Res Function(AccountState_Question) _then) = _$AccountState_QuestionCopyWithImpl;
@useResult
$Res call({
 BindingQuestion field0
});




}
/// @nodoc
class _$AccountState_QuestionCopyWithImpl<$Res>
    implements $AccountState_QuestionCopyWith<$Res> {
  _$AccountState_QuestionCopyWithImpl(this._self, this._then);

  final AccountState_Question _self;
  final $Res Function(AccountState_Question) _then;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AccountState_Question(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as BindingQuestion,
  ));
}


}

/// @nodoc
mixin _$ApprovalPhase {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase()';
}


}

/// @nodoc
class $ApprovalPhaseCopyWith<$Res>  {
$ApprovalPhaseCopyWith(ApprovalPhase _, $Res Function(ApprovalPhase) __);
}


/// Adds pattern-matching-related methods to [ApprovalPhase].
extension ApprovalPhasePatterns on ApprovalPhase {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ApprovalPhase_Requesting value)?  requesting,TResult Function( ApprovalPhase_Waiting value)?  waiting,TResult Function( ApprovalPhase_Compare value)?  compare,TResult Function( ApprovalPhase_Unlocking value)?  unlocking,TResult Function( ApprovalPhase_Approved value)?  approved,TResult Function( ApprovalPhase_NoDevices value)?  noDevices,TResult Function( ApprovalPhase_Cancelled value)?  cancelled,TResult Function( ApprovalPhase_Expired value)?  expired,TResult Function( ApprovalPhase_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting(_that);case ApprovalPhase_Waiting() when waiting != null:
return waiting(_that);case ApprovalPhase_Compare() when compare != null:
return compare(_that);case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking(_that);case ApprovalPhase_Approved() when approved != null:
return approved(_that);case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices(_that);case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled(_that);case ApprovalPhase_Expired() when expired != null:
return expired(_that);case ApprovalPhase_Failed() when failed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ApprovalPhase_Requesting value)  requesting,required TResult Function( ApprovalPhase_Waiting value)  waiting,required TResult Function( ApprovalPhase_Compare value)  compare,required TResult Function( ApprovalPhase_Unlocking value)  unlocking,required TResult Function( ApprovalPhase_Approved value)  approved,required TResult Function( ApprovalPhase_NoDevices value)  noDevices,required TResult Function( ApprovalPhase_Cancelled value)  cancelled,required TResult Function( ApprovalPhase_Expired value)  expired,required TResult Function( ApprovalPhase_Failed value)  failed,}){
final _that = this;
switch (_that) {
case ApprovalPhase_Requesting():
return requesting(_that);case ApprovalPhase_Waiting():
return waiting(_that);case ApprovalPhase_Compare():
return compare(_that);case ApprovalPhase_Unlocking():
return unlocking(_that);case ApprovalPhase_Approved():
return approved(_that);case ApprovalPhase_NoDevices():
return noDevices(_that);case ApprovalPhase_Cancelled():
return cancelled(_that);case ApprovalPhase_Expired():
return expired(_that);case ApprovalPhase_Failed():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ApprovalPhase_Requesting value)?  requesting,TResult? Function( ApprovalPhase_Waiting value)?  waiting,TResult? Function( ApprovalPhase_Compare value)?  compare,TResult? Function( ApprovalPhase_Unlocking value)?  unlocking,TResult? Function( ApprovalPhase_Approved value)?  approved,TResult? Function( ApprovalPhase_NoDevices value)?  noDevices,TResult? Function( ApprovalPhase_Cancelled value)?  cancelled,TResult? Function( ApprovalPhase_Expired value)?  expired,TResult? Function( ApprovalPhase_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting(_that);case ApprovalPhase_Waiting() when waiting != null:
return waiting(_that);case ApprovalPhase_Compare() when compare != null:
return compare(_that);case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking(_that);case ApprovalPhase_Approved() when approved != null:
return approved(_that);case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices(_that);case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled(_that);case ApprovalPhase_Expired() when expired != null:
return expired(_that);case ApprovalPhase_Failed() when failed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  requesting,TResult Function()?  waiting,TResult Function()?  compare,TResult Function()?  unlocking,TResult Function()?  approved,TResult Function()?  noDevices,TResult Function()?  cancelled,TResult Function()?  expired,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting();case ApprovalPhase_Waiting() when waiting != null:
return waiting();case ApprovalPhase_Compare() when compare != null:
return compare();case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking();case ApprovalPhase_Approved() when approved != null:
return approved();case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices();case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled();case ApprovalPhase_Expired() when expired != null:
return expired();case ApprovalPhase_Failed() when failed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  requesting,required TResult Function()  waiting,required TResult Function()  compare,required TResult Function()  unlocking,required TResult Function()  approved,required TResult Function()  noDevices,required TResult Function()  cancelled,required TResult Function()  expired,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case ApprovalPhase_Requesting():
return requesting();case ApprovalPhase_Waiting():
return waiting();case ApprovalPhase_Compare():
return compare();case ApprovalPhase_Unlocking():
return unlocking();case ApprovalPhase_Approved():
return approved();case ApprovalPhase_NoDevices():
return noDevices();case ApprovalPhase_Cancelled():
return cancelled();case ApprovalPhase_Expired():
return expired();case ApprovalPhase_Failed():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  requesting,TResult? Function()?  waiting,TResult? Function()?  compare,TResult? Function()?  unlocking,TResult? Function()?  approved,TResult? Function()?  noDevices,TResult? Function()?  cancelled,TResult? Function()?  expired,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting();case ApprovalPhase_Waiting() when waiting != null:
return waiting();case ApprovalPhase_Compare() when compare != null:
return compare();case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking();case ApprovalPhase_Approved() when approved != null:
return approved();case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices();case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled();case ApprovalPhase_Expired() when expired != null:
return expired();case ApprovalPhase_Failed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class ApprovalPhase_Requesting extends ApprovalPhase {
  const ApprovalPhase_Requesting(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Requesting);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.requesting()';
}


}




/// @nodoc


class ApprovalPhase_Waiting extends ApprovalPhase {
  const ApprovalPhase_Waiting(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Waiting);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.waiting()';
}


}




/// @nodoc


class ApprovalPhase_Compare extends ApprovalPhase {
  const ApprovalPhase_Compare(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Compare);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.compare()';
}


}




/// @nodoc


class ApprovalPhase_Unlocking extends ApprovalPhase {
  const ApprovalPhase_Unlocking(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Unlocking);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.unlocking()';
}


}




/// @nodoc


class ApprovalPhase_Approved extends ApprovalPhase {
  const ApprovalPhase_Approved(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Approved);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.approved()';
}


}




/// @nodoc


class ApprovalPhase_NoDevices extends ApprovalPhase {
  const ApprovalPhase_NoDevices(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_NoDevices);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.noDevices()';
}


}




/// @nodoc


class ApprovalPhase_Cancelled extends ApprovalPhase {
  const ApprovalPhase_Cancelled(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Cancelled);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.cancelled()';
}


}




/// @nodoc


class ApprovalPhase_Expired extends ApprovalPhase {
  const ApprovalPhase_Expired(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Expired);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.expired()';
}


}




/// @nodoc


class ApprovalPhase_Failed extends ApprovalPhase {
  const ApprovalPhase_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of ApprovalPhase
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ApprovalPhase_FailedCopyWith<ApprovalPhase_Failed> get copyWith => _$ApprovalPhase_FailedCopyWithImpl<ApprovalPhase_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApprovalPhase.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $ApprovalPhase_FailedCopyWith<$Res> implements $ApprovalPhaseCopyWith<$Res> {
  factory $ApprovalPhase_FailedCopyWith(ApprovalPhase_Failed value, $Res Function(ApprovalPhase_Failed) _then) = _$ApprovalPhase_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ApprovalPhase_FailedCopyWithImpl<$Res>
    implements $ApprovalPhase_FailedCopyWith<$Res> {
  _$ApprovalPhase_FailedCopyWithImpl(this._self, this._then);

  final ApprovalPhase_Failed _self;
  final $Res Function(ApprovalPhase_Failed) _then;

/// Create a copy of ApprovalPhase
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ApprovalPhase_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$CommentPlacement {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentPlacement);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CommentPlacement()';
}


}

/// @nodoc
class $CommentPlacementCopyWith<$Res>  {
$CommentPlacementCopyWith(CommentPlacement _, $Res Function(CommentPlacement) __);
}


/// Adds pattern-matching-related methods to [CommentPlacement].
extension CommentPlacementPatterns on CommentPlacement {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CommentPlacement_Anchored value)?  anchored,TResult Function( CommentPlacement_Orphaned value)?  orphaned,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CommentPlacement_Anchored() when anchored != null:
return anchored(_that);case CommentPlacement_Orphaned() when orphaned != null:
return orphaned(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CommentPlacement_Anchored value)  anchored,required TResult Function( CommentPlacement_Orphaned value)  orphaned,}){
final _that = this;
switch (_that) {
case CommentPlacement_Anchored():
return anchored(_that);case CommentPlacement_Orphaned():
return orphaned(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CommentPlacement_Anchored value)?  anchored,TResult? Function( CommentPlacement_Orphaned value)?  orphaned,}){
final _that = this;
switch (_that) {
case CommentPlacement_Anchored() when anchored != null:
return anchored(_that);case CommentPlacement_Orphaned() when orphaned != null:
return orphaned(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int start,  int end)?  anchored,TResult Function( int at)?  orphaned,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CommentPlacement_Anchored() when anchored != null:
return anchored(_that.start,_that.end);case CommentPlacement_Orphaned() when orphaned != null:
return orphaned(_that.at);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int start,  int end)  anchored,required TResult Function( int at)  orphaned,}) {final _that = this;
switch (_that) {
case CommentPlacement_Anchored():
return anchored(_that.start,_that.end);case CommentPlacement_Orphaned():
return orphaned(_that.at);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int start,  int end)?  anchored,TResult? Function( int at)?  orphaned,}) {final _that = this;
switch (_that) {
case CommentPlacement_Anchored() when anchored != null:
return anchored(_that.start,_that.end);case CommentPlacement_Orphaned() when orphaned != null:
return orphaned(_that.at);case _:
  return null;

}
}

}

/// @nodoc


class CommentPlacement_Anchored extends CommentPlacement {
  const CommentPlacement_Anchored({required this.start, required this.end}): super._();
  

 final  int start;
 final  int end;

/// Create a copy of CommentPlacement
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentPlacement_AnchoredCopyWith<CommentPlacement_Anchored> get copyWith => _$CommentPlacement_AnchoredCopyWithImpl<CommentPlacement_Anchored>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentPlacement_Anchored&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end));
}


@override
int get hashCode => Object.hash(runtimeType,start,end);

@override
String toString() {
  return 'CommentPlacement.anchored(start: $start, end: $end)';
}


}

/// @nodoc
abstract mixin class $CommentPlacement_AnchoredCopyWith<$Res> implements $CommentPlacementCopyWith<$Res> {
  factory $CommentPlacement_AnchoredCopyWith(CommentPlacement_Anchored value, $Res Function(CommentPlacement_Anchored) _then) = _$CommentPlacement_AnchoredCopyWithImpl;
@useResult
$Res call({
 int start, int end
});




}
/// @nodoc
class _$CommentPlacement_AnchoredCopyWithImpl<$Res>
    implements $CommentPlacement_AnchoredCopyWith<$Res> {
  _$CommentPlacement_AnchoredCopyWithImpl(this._self, this._then);

  final CommentPlacement_Anchored _self;
  final $Res Function(CommentPlacement_Anchored) _then;

/// Create a copy of CommentPlacement
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? start = null,Object? end = null,}) {
  return _then(CommentPlacement_Anchored(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class CommentPlacement_Orphaned extends CommentPlacement {
  const CommentPlacement_Orphaned({required this.at}): super._();
  

 final  int at;

/// Create a copy of CommentPlacement
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentPlacement_OrphanedCopyWith<CommentPlacement_Orphaned> get copyWith => _$CommentPlacement_OrphanedCopyWithImpl<CommentPlacement_Orphaned>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentPlacement_Orphaned&&(identical(other.at, at) || other.at == at));
}


@override
int get hashCode => Object.hash(runtimeType,at);

@override
String toString() {
  return 'CommentPlacement.orphaned(at: $at)';
}


}

/// @nodoc
abstract mixin class $CommentPlacement_OrphanedCopyWith<$Res> implements $CommentPlacementCopyWith<$Res> {
  factory $CommentPlacement_OrphanedCopyWith(CommentPlacement_Orphaned value, $Res Function(CommentPlacement_Orphaned) _then) = _$CommentPlacement_OrphanedCopyWithImpl;
@useResult
$Res call({
 int at
});




}
/// @nodoc
class _$CommentPlacement_OrphanedCopyWithImpl<$Res>
    implements $CommentPlacement_OrphanedCopyWith<$Res> {
  _$CommentPlacement_OrphanedCopyWithImpl(this._self, this._then);

  final CommentPlacement_Orphaned _self;
  final $Res Function(CommentPlacement_Orphaned) _then;

/// Create a copy of CommentPlacement
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? at = null,}) {
  return _then(CommentPlacement_Orphaned(
at: null == at ? _self.at : at // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
mixin _$EditCommand {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditCommand()';
}


}

/// @nodoc
class $EditCommandCopyWith<$Res>  {
$EditCommandCopyWith(EditCommand _, $Res Function(EditCommand) __);
}


/// Adds pattern-matching-related methods to [EditCommand].
extension EditCommandPatterns on EditCommand {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( EditCommand_Enter value)?  enter,TResult Function( EditCommand_Tab value)?  tab,TResult Function( EditCommand_Format value)?  format,TResult Function( EditCommand_CycleHeading value)?  cycleHeading,TResult Function( EditCommand_WikiLink value)?  wikiLink,TResult Function( EditCommand_Insert value)?  insert,required TResult orElse(),}){
final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that);case EditCommand_Tab() when tab != null:
return tab(_that);case EditCommand_Format() when format != null:
return format(_that);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading(_that);case EditCommand_WikiLink() when wikiLink != null:
return wikiLink(_that);case EditCommand_Insert() when insert != null:
return insert(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( EditCommand_Enter value)  enter,required TResult Function( EditCommand_Tab value)  tab,required TResult Function( EditCommand_Format value)  format,required TResult Function( EditCommand_CycleHeading value)  cycleHeading,required TResult Function( EditCommand_WikiLink value)  wikiLink,required TResult Function( EditCommand_Insert value)  insert,}){
final _that = this;
switch (_that) {
case EditCommand_Enter():
return enter(_that);case EditCommand_Tab():
return tab(_that);case EditCommand_Format():
return format(_that);case EditCommand_CycleHeading():
return cycleHeading(_that);case EditCommand_WikiLink():
return wikiLink(_that);case EditCommand_Insert():
return insert(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( EditCommand_Enter value)?  enter,TResult? Function( EditCommand_Tab value)?  tab,TResult? Function( EditCommand_Format value)?  format,TResult? Function( EditCommand_CycleHeading value)?  cycleHeading,TResult? Function( EditCommand_WikiLink value)?  wikiLink,TResult? Function( EditCommand_Insert value)?  insert,}){
final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that);case EditCommand_Tab() when tab != null:
return tab(_that);case EditCommand_Format() when format != null:
return format(_that);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading(_that);case EditCommand_WikiLink() when wikiLink != null:
return wikiLink(_that);case EditCommand_Insert() when insert != null:
return insert(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool plain)?  enter,TResult Function( bool out)?  tab,TResult Function( FormatKind format)?  format,TResult Function()?  cycleHeading,TResult Function()?  wikiLink,TResult Function( Range16 range,  String text)?  insert,required TResult orElse(),}) {final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that.plain);case EditCommand_Tab() when tab != null:
return tab(_that.out);case EditCommand_Format() when format != null:
return format(_that.format);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading();case EditCommand_WikiLink() when wikiLink != null:
return wikiLink();case EditCommand_Insert() when insert != null:
return insert(_that.range,_that.text);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool plain)  enter,required TResult Function( bool out)  tab,required TResult Function( FormatKind format)  format,required TResult Function()  cycleHeading,required TResult Function()  wikiLink,required TResult Function( Range16 range,  String text)  insert,}) {final _that = this;
switch (_that) {
case EditCommand_Enter():
return enter(_that.plain);case EditCommand_Tab():
return tab(_that.out);case EditCommand_Format():
return format(_that.format);case EditCommand_CycleHeading():
return cycleHeading();case EditCommand_WikiLink():
return wikiLink();case EditCommand_Insert():
return insert(_that.range,_that.text);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool plain)?  enter,TResult? Function( bool out)?  tab,TResult? Function( FormatKind format)?  format,TResult? Function()?  cycleHeading,TResult? Function()?  wikiLink,TResult? Function( Range16 range,  String text)?  insert,}) {final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that.plain);case EditCommand_Tab() when tab != null:
return tab(_that.out);case EditCommand_Format() when format != null:
return format(_that.format);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading();case EditCommand_WikiLink() when wikiLink != null:
return wikiLink();case EditCommand_Insert() when insert != null:
return insert(_that.range,_that.text);case _:
  return null;

}
}

}

/// @nodoc


class EditCommand_Enter extends EditCommand {
  const EditCommand_Enter({required this.plain}): super._();
  

 final  bool plain;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_EnterCopyWith<EditCommand_Enter> get copyWith => _$EditCommand_EnterCopyWithImpl<EditCommand_Enter>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Enter&&(identical(other.plain, plain) || other.plain == plain));
}


@override
int get hashCode => Object.hash(runtimeType,plain);

@override
String toString() {
  return 'EditCommand.enter(plain: $plain)';
}


}

/// @nodoc
abstract mixin class $EditCommand_EnterCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_EnterCopyWith(EditCommand_Enter value, $Res Function(EditCommand_Enter) _then) = _$EditCommand_EnterCopyWithImpl;
@useResult
$Res call({
 bool plain
});




}
/// @nodoc
class _$EditCommand_EnterCopyWithImpl<$Res>
    implements $EditCommand_EnterCopyWith<$Res> {
  _$EditCommand_EnterCopyWithImpl(this._self, this._then);

  final EditCommand_Enter _self;
  final $Res Function(EditCommand_Enter) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? plain = null,}) {
  return _then(EditCommand_Enter(
plain: null == plain ? _self.plain : plain // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class EditCommand_Tab extends EditCommand {
  const EditCommand_Tab({required this.out}): super._();
  

 final  bool out;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_TabCopyWith<EditCommand_Tab> get copyWith => _$EditCommand_TabCopyWithImpl<EditCommand_Tab>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Tab&&(identical(other.out, out) || other.out == out));
}


@override
int get hashCode => Object.hash(runtimeType,out);

@override
String toString() {
  return 'EditCommand.tab(out: $out)';
}


}

/// @nodoc
abstract mixin class $EditCommand_TabCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_TabCopyWith(EditCommand_Tab value, $Res Function(EditCommand_Tab) _then) = _$EditCommand_TabCopyWithImpl;
@useResult
$Res call({
 bool out
});




}
/// @nodoc
class _$EditCommand_TabCopyWithImpl<$Res>
    implements $EditCommand_TabCopyWith<$Res> {
  _$EditCommand_TabCopyWithImpl(this._self, this._then);

  final EditCommand_Tab _self;
  final $Res Function(EditCommand_Tab) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? out = null,}) {
  return _then(EditCommand_Tab(
out: null == out ? _self.out : out // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class EditCommand_Format extends EditCommand {
  const EditCommand_Format({required this.format}): super._();
  

 final  FormatKind format;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_FormatCopyWith<EditCommand_Format> get copyWith => _$EditCommand_FormatCopyWithImpl<EditCommand_Format>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Format&&(identical(other.format, format) || other.format == format));
}


@override
int get hashCode => Object.hash(runtimeType,format);

@override
String toString() {
  return 'EditCommand.format(format: $format)';
}


}

/// @nodoc
abstract mixin class $EditCommand_FormatCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_FormatCopyWith(EditCommand_Format value, $Res Function(EditCommand_Format) _then) = _$EditCommand_FormatCopyWithImpl;
@useResult
$Res call({
 FormatKind format
});




}
/// @nodoc
class _$EditCommand_FormatCopyWithImpl<$Res>
    implements $EditCommand_FormatCopyWith<$Res> {
  _$EditCommand_FormatCopyWithImpl(this._self, this._then);

  final EditCommand_Format _self;
  final $Res Function(EditCommand_Format) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? format = null,}) {
  return _then(EditCommand_Format(
format: null == format ? _self.format : format // ignore: cast_nullable_to_non_nullable
as FormatKind,
  ));
}


}

/// @nodoc


class EditCommand_CycleHeading extends EditCommand {
  const EditCommand_CycleHeading(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_CycleHeading);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditCommand.cycleHeading()';
}


}




/// @nodoc


class EditCommand_WikiLink extends EditCommand {
  const EditCommand_WikiLink(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_WikiLink);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditCommand.wikiLink()';
}


}




/// @nodoc


class EditCommand_Insert extends EditCommand {
  const EditCommand_Insert({required this.range, required this.text}): super._();
  

 final  Range16 range;
 final  String text;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_InsertCopyWith<EditCommand_Insert> get copyWith => _$EditCommand_InsertCopyWithImpl<EditCommand_Insert>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Insert&&(identical(other.range, range) || other.range == range)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,range,text);

@override
String toString() {
  return 'EditCommand.insert(range: $range, text: $text)';
}


}

/// @nodoc
abstract mixin class $EditCommand_InsertCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_InsertCopyWith(EditCommand_Insert value, $Res Function(EditCommand_Insert) _then) = _$EditCommand_InsertCopyWithImpl;
@useResult
$Res call({
 Range16 range, String text
});




}
/// @nodoc
class _$EditCommand_InsertCopyWithImpl<$Res>
    implements $EditCommand_InsertCopyWith<$Res> {
  _$EditCommand_InsertCopyWithImpl(this._self, this._then);

  final EditCommand_Insert _self;
  final $Res Function(EditCommand_Insert) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? range = null,Object? text = null,}) {
  return _then(EditCommand_Insert(
range: null == range ? _self.range : range // ignore: cast_nullable_to_non_nullable
as Range16,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$EditorEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent()';
}


}

/// @nodoc
class $EditorEventCopyWith<$Res>  {
$EditorEventCopyWith(EditorEvent _, $Res Function(EditorEvent) __);
}


/// Adds pattern-matching-related methods to [EditorEvent].
extension EditorEventPatterns on EditorEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( EditorEvent_Saved value)?  saved,TResult Function( EditorEvent_Incoming value)?  incoming,TResult Function( EditorEvent_Reloaded value)?  reloaded,TResult Function( EditorEvent_Closed value)?  closed,TResult Function( EditorEvent_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that);case EditorEvent_Incoming() when incoming != null:
return incoming(_that);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that);case EditorEvent_Closed() when closed != null:
return closed(_that);case EditorEvent_Failed() when failed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( EditorEvent_Saved value)  saved,required TResult Function( EditorEvent_Incoming value)  incoming,required TResult Function( EditorEvent_Reloaded value)  reloaded,required TResult Function( EditorEvent_Closed value)  closed,required TResult Function( EditorEvent_Failed value)  failed,}){
final _that = this;
switch (_that) {
case EditorEvent_Saved():
return saved(_that);case EditorEvent_Incoming():
return incoming(_that);case EditorEvent_Reloaded():
return reloaded(_that);case EditorEvent_Closed():
return closed(_that);case EditorEvent_Failed():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( EditorEvent_Saved value)?  saved,TResult? Function( EditorEvent_Incoming value)?  incoming,TResult? Function( EditorEvent_Reloaded value)?  reloaded,TResult? Function( EditorEvent_Closed value)?  closed,TResult? Function( EditorEvent_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that);case EditorEvent_Incoming() when incoming != null:
return incoming(_that);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that);case EditorEvent_Closed() when closed != null:
return closed(_that);case EditorEvent_Failed() when failed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String rev,  int atMs)?  saved,TResult Function( bool deleted)?  incoming,TResult Function( EditorSnapshot snapshot)?  reloaded,TResult Function()?  closed,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case EditorEvent_Incoming() when incoming != null:
return incoming(_that.deleted);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that.snapshot);case EditorEvent_Closed() when closed != null:
return closed();case EditorEvent_Failed() when failed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String rev,  int atMs)  saved,required TResult Function( bool deleted)  incoming,required TResult Function( EditorSnapshot snapshot)  reloaded,required TResult Function()  closed,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case EditorEvent_Saved():
return saved(_that.rev,_that.atMs);case EditorEvent_Incoming():
return incoming(_that.deleted);case EditorEvent_Reloaded():
return reloaded(_that.snapshot);case EditorEvent_Closed():
return closed();case EditorEvent_Failed():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String rev,  int atMs)?  saved,TResult? Function( bool deleted)?  incoming,TResult? Function( EditorSnapshot snapshot)?  reloaded,TResult? Function()?  closed,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case EditorEvent_Incoming() when incoming != null:
return incoming(_that.deleted);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that.snapshot);case EditorEvent_Closed() when closed != null:
return closed();case EditorEvent_Failed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class EditorEvent_Saved extends EditorEvent {
  const EditorEvent_Saved({required this.rev, required this.atMs}): super._();
  

 final  String rev;
 final  int atMs;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_SavedCopyWith<EditorEvent_Saved> get copyWith => _$EditorEvent_SavedCopyWithImpl<EditorEvent_Saved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Saved&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.atMs, atMs) || other.atMs == atMs));
}


@override
int get hashCode => Object.hash(runtimeType,rev,atMs);

@override
String toString() {
  return 'EditorEvent.saved(rev: $rev, atMs: $atMs)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_SavedCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_SavedCopyWith(EditorEvent_Saved value, $Res Function(EditorEvent_Saved) _then) = _$EditorEvent_SavedCopyWithImpl;
@useResult
$Res call({
 String rev, int atMs
});




}
/// @nodoc
class _$EditorEvent_SavedCopyWithImpl<$Res>
    implements $EditorEvent_SavedCopyWith<$Res> {
  _$EditorEvent_SavedCopyWithImpl(this._self, this._then);

  final EditorEvent_Saved _self;
  final $Res Function(EditorEvent_Saved) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,Object? atMs = null,}) {
  return _then(EditorEvent_Saved(
rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,atMs: null == atMs ? _self.atMs : atMs // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class EditorEvent_Incoming extends EditorEvent {
  const EditorEvent_Incoming({required this.deleted}): super._();
  

 final  bool deleted;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_IncomingCopyWith<EditorEvent_Incoming> get copyWith => _$EditorEvent_IncomingCopyWithImpl<EditorEvent_Incoming>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Incoming&&(identical(other.deleted, deleted) || other.deleted == deleted));
}


@override
int get hashCode => Object.hash(runtimeType,deleted);

@override
String toString() {
  return 'EditorEvent.incoming(deleted: $deleted)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_IncomingCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_IncomingCopyWith(EditorEvent_Incoming value, $Res Function(EditorEvent_Incoming) _then) = _$EditorEvent_IncomingCopyWithImpl;
@useResult
$Res call({
 bool deleted
});




}
/// @nodoc
class _$EditorEvent_IncomingCopyWithImpl<$Res>
    implements $EditorEvent_IncomingCopyWith<$Res> {
  _$EditorEvent_IncomingCopyWithImpl(this._self, this._then);

  final EditorEvent_Incoming _self;
  final $Res Function(EditorEvent_Incoming) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deleted = null,}) {
  return _then(EditorEvent_Incoming(
deleted: null == deleted ? _self.deleted : deleted // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class EditorEvent_Reloaded extends EditorEvent {
  const EditorEvent_Reloaded({required this.snapshot}): super._();
  

 final  EditorSnapshot snapshot;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_ReloadedCopyWith<EditorEvent_Reloaded> get copyWith => _$EditorEvent_ReloadedCopyWithImpl<EditorEvent_Reloaded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Reloaded&&(identical(other.snapshot, snapshot) || other.snapshot == snapshot));
}


@override
int get hashCode => Object.hash(runtimeType,snapshot);

@override
String toString() {
  return 'EditorEvent.reloaded(snapshot: $snapshot)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_ReloadedCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_ReloadedCopyWith(EditorEvent_Reloaded value, $Res Function(EditorEvent_Reloaded) _then) = _$EditorEvent_ReloadedCopyWithImpl;
@useResult
$Res call({
 EditorSnapshot snapshot
});




}
/// @nodoc
class _$EditorEvent_ReloadedCopyWithImpl<$Res>
    implements $EditorEvent_ReloadedCopyWith<$Res> {
  _$EditorEvent_ReloadedCopyWithImpl(this._self, this._then);

  final EditorEvent_Reloaded _self;
  final $Res Function(EditorEvent_Reloaded) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? snapshot = null,}) {
  return _then(EditorEvent_Reloaded(
snapshot: null == snapshot ? _self.snapshot : snapshot // ignore: cast_nullable_to_non_nullable
as EditorSnapshot,
  ));
}


}

/// @nodoc


class EditorEvent_Closed extends EditorEvent {
  const EditorEvent_Closed(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Closed);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.closed()';
}


}




/// @nodoc


class EditorEvent_Failed extends EditorEvent {
  const EditorEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_FailedCopyWith<EditorEvent_Failed> get copyWith => _$EditorEvent_FailedCopyWithImpl<EditorEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'EditorEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_FailedCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_FailedCopyWith(EditorEvent_Failed value, $Res Function(EditorEvent_Failed) _then) = _$EditorEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$EditorEvent_FailedCopyWithImpl<$Res>
    implements $EditorEvent_FailedCopyWith<$Res> {
  _$EditorEvent_FailedCopyWithImpl(this._self, this._then);

  final EditorEvent_Failed _self;
  final $Res Function(EditorEvent_Failed) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(EditorEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ImportEvent {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportEvent&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'ImportEvent(field0: $field0)';
}


}

/// @nodoc
class $ImportEventCopyWith<$Res>  {
$ImportEventCopyWith(ImportEvent _, $Res Function(ImportEvent) __);
}


/// Adds pattern-matching-related methods to [ImportEvent].
extension ImportEventPatterns on ImportEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ImportEvent_Progress value)?  progress,TResult Function( ImportEvent_Done value)?  done,TResult Function( ImportEvent_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ImportEvent_Progress() when progress != null:
return progress(_that);case ImportEvent_Done() when done != null:
return done(_that);case ImportEvent_Failed() when failed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ImportEvent_Progress value)  progress,required TResult Function( ImportEvent_Done value)  done,required TResult Function( ImportEvent_Failed value)  failed,}){
final _that = this;
switch (_that) {
case ImportEvent_Progress():
return progress(_that);case ImportEvent_Done():
return done(_that);case ImportEvent_Failed():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ImportEvent_Progress value)?  progress,TResult? Function( ImportEvent_Done value)?  done,TResult? Function( ImportEvent_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case ImportEvent_Progress() when progress != null:
return progress(_that);case ImportEvent_Done() when done != null:
return done(_that);case ImportEvent_Failed() when failed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ImportProgress field0)?  progress,TResult Function( ImportSummary field0)?  done,TResult Function( String field0)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ImportEvent_Progress() when progress != null:
return progress(_that.field0);case ImportEvent_Done() when done != null:
return done(_that.field0);case ImportEvent_Failed() when failed != null:
return failed(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ImportProgress field0)  progress,required TResult Function( ImportSummary field0)  done,required TResult Function( String field0)  failed,}) {final _that = this;
switch (_that) {
case ImportEvent_Progress():
return progress(_that.field0);case ImportEvent_Done():
return done(_that.field0);case ImportEvent_Failed():
return failed(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ImportProgress field0)?  progress,TResult? Function( ImportSummary field0)?  done,TResult? Function( String field0)?  failed,}) {final _that = this;
switch (_that) {
case ImportEvent_Progress() when progress != null:
return progress(_that.field0);case ImportEvent_Done() when done != null:
return done(_that.field0);case ImportEvent_Failed() when failed != null:
return failed(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class ImportEvent_Progress extends ImportEvent {
  const ImportEvent_Progress(this.field0): super._();
  

@override final  ImportProgress field0;

/// Create a copy of ImportEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportEvent_ProgressCopyWith<ImportEvent_Progress> get copyWith => _$ImportEvent_ProgressCopyWithImpl<ImportEvent_Progress>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportEvent_Progress&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'ImportEvent.progress(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ImportEvent_ProgressCopyWith<$Res> implements $ImportEventCopyWith<$Res> {
  factory $ImportEvent_ProgressCopyWith(ImportEvent_Progress value, $Res Function(ImportEvent_Progress) _then) = _$ImportEvent_ProgressCopyWithImpl;
@useResult
$Res call({
 ImportProgress field0
});




}
/// @nodoc
class _$ImportEvent_ProgressCopyWithImpl<$Res>
    implements $ImportEvent_ProgressCopyWith<$Res> {
  _$ImportEvent_ProgressCopyWithImpl(this._self, this._then);

  final ImportEvent_Progress _self;
  final $Res Function(ImportEvent_Progress) _then;

/// Create a copy of ImportEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ImportEvent_Progress(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as ImportProgress,
  ));
}


}

/// @nodoc


class ImportEvent_Done extends ImportEvent {
  const ImportEvent_Done(this.field0): super._();
  

@override final  ImportSummary field0;

/// Create a copy of ImportEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportEvent_DoneCopyWith<ImportEvent_Done> get copyWith => _$ImportEvent_DoneCopyWithImpl<ImportEvent_Done>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportEvent_Done&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'ImportEvent.done(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ImportEvent_DoneCopyWith<$Res> implements $ImportEventCopyWith<$Res> {
  factory $ImportEvent_DoneCopyWith(ImportEvent_Done value, $Res Function(ImportEvent_Done) _then) = _$ImportEvent_DoneCopyWithImpl;
@useResult
$Res call({
 ImportSummary field0
});




}
/// @nodoc
class _$ImportEvent_DoneCopyWithImpl<$Res>
    implements $ImportEvent_DoneCopyWith<$Res> {
  _$ImportEvent_DoneCopyWithImpl(this._self, this._then);

  final ImportEvent_Done _self;
  final $Res Function(ImportEvent_Done) _then;

/// Create a copy of ImportEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ImportEvent_Done(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as ImportSummary,
  ));
}


}

/// @nodoc


class ImportEvent_Failed extends ImportEvent {
  const ImportEvent_Failed(this.field0): super._();
  

@override final  String field0;

/// Create a copy of ImportEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ImportEvent_FailedCopyWith<ImportEvent_Failed> get copyWith => _$ImportEvent_FailedCopyWithImpl<ImportEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ImportEvent_Failed&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'ImportEvent.failed(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ImportEvent_FailedCopyWith<$Res> implements $ImportEventCopyWith<$Res> {
  factory $ImportEvent_FailedCopyWith(ImportEvent_Failed value, $Res Function(ImportEvent_Failed) _then) = _$ImportEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$ImportEvent_FailedCopyWithImpl<$Res>
    implements $ImportEvent_FailedCopyWith<$Res> {
  _$ImportEvent_FailedCopyWithImpl(this._self, this._then);

  final ImportEvent_Failed _self;
  final $Res Function(ImportEvent_Failed) _then;

/// Create a copy of ImportEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ImportEvent_Failed(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$KansoError {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError()';
}


}

/// @nodoc
class $KansoErrorCopyWith<$Res>  {
$KansoErrorCopyWith(KansoError _, $Res Function(KansoError) __);
}


/// Adds pattern-matching-related methods to [KansoError].
extension KansoErrorPatterns on KansoError {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( KansoError_NotFound value)?  notFound,TResult Function( KansoError_Locked value)?  locked,TResult Function( KansoError_Conflict value)?  conflict,TResult Function( KansoError_Invalid value)?  invalid,TResult Function( KansoError_Storage value)?  storage,TResult Function( KansoError_Network value)?  network,TResult Function( KansoError_SignedOut value)?  signedOut,TResult Function( KansoError_Auth value)?  auth,TResult Function( KansoError_Internal value)?  internal,required TResult orElse(),}){
final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound(_that);case KansoError_Locked() when locked != null:
return locked(_that);case KansoError_Conflict() when conflict != null:
return conflict(_that);case KansoError_Invalid() when invalid != null:
return invalid(_that);case KansoError_Storage() when storage != null:
return storage(_that);case KansoError_Network() when network != null:
return network(_that);case KansoError_SignedOut() when signedOut != null:
return signedOut(_that);case KansoError_Auth() when auth != null:
return auth(_that);case KansoError_Internal() when internal != null:
return internal(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( KansoError_NotFound value)  notFound,required TResult Function( KansoError_Locked value)  locked,required TResult Function( KansoError_Conflict value)  conflict,required TResult Function( KansoError_Invalid value)  invalid,required TResult Function( KansoError_Storage value)  storage,required TResult Function( KansoError_Network value)  network,required TResult Function( KansoError_SignedOut value)  signedOut,required TResult Function( KansoError_Auth value)  auth,required TResult Function( KansoError_Internal value)  internal,}){
final _that = this;
switch (_that) {
case KansoError_NotFound():
return notFound(_that);case KansoError_Locked():
return locked(_that);case KansoError_Conflict():
return conflict(_that);case KansoError_Invalid():
return invalid(_that);case KansoError_Storage():
return storage(_that);case KansoError_Network():
return network(_that);case KansoError_SignedOut():
return signedOut(_that);case KansoError_Auth():
return auth(_that);case KansoError_Internal():
return internal(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( KansoError_NotFound value)?  notFound,TResult? Function( KansoError_Locked value)?  locked,TResult? Function( KansoError_Conflict value)?  conflict,TResult? Function( KansoError_Invalid value)?  invalid,TResult? Function( KansoError_Storage value)?  storage,TResult? Function( KansoError_Network value)?  network,TResult? Function( KansoError_SignedOut value)?  signedOut,TResult? Function( KansoError_Auth value)?  auth,TResult? Function( KansoError_Internal value)?  internal,}){
final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound(_that);case KansoError_Locked() when locked != null:
return locked(_that);case KansoError_Conflict() when conflict != null:
return conflict(_that);case KansoError_Invalid() when invalid != null:
return invalid(_that);case KansoError_Storage() when storage != null:
return storage(_that);case KansoError_Network() when network != null:
return network(_that);case KansoError_SignedOut() when signedOut != null:
return signedOut(_that);case KansoError_Auth() when auth != null:
return auth(_that);case KansoError_Internal() when internal != null:
return internal(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  notFound,TResult Function()?  locked,TResult Function( String currentRev)?  conflict,TResult Function( String message)?  invalid,TResult Function( String message)?  storage,TResult Function( String message)?  network,TResult Function()?  signedOut,TResult Function( String message)?  auth,TResult Function( String message)?  internal,required TResult orElse(),}) {final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound();case KansoError_Locked() when locked != null:
return locked();case KansoError_Conflict() when conflict != null:
return conflict(_that.currentRev);case KansoError_Invalid() when invalid != null:
return invalid(_that.message);case KansoError_Storage() when storage != null:
return storage(_that.message);case KansoError_Network() when network != null:
return network(_that.message);case KansoError_SignedOut() when signedOut != null:
return signedOut();case KansoError_Auth() when auth != null:
return auth(_that.message);case KansoError_Internal() when internal != null:
return internal(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  notFound,required TResult Function()  locked,required TResult Function( String currentRev)  conflict,required TResult Function( String message)  invalid,required TResult Function( String message)  storage,required TResult Function( String message)  network,required TResult Function()  signedOut,required TResult Function( String message)  auth,required TResult Function( String message)  internal,}) {final _that = this;
switch (_that) {
case KansoError_NotFound():
return notFound();case KansoError_Locked():
return locked();case KansoError_Conflict():
return conflict(_that.currentRev);case KansoError_Invalid():
return invalid(_that.message);case KansoError_Storage():
return storage(_that.message);case KansoError_Network():
return network(_that.message);case KansoError_SignedOut():
return signedOut();case KansoError_Auth():
return auth(_that.message);case KansoError_Internal():
return internal(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  notFound,TResult? Function()?  locked,TResult? Function( String currentRev)?  conflict,TResult? Function( String message)?  invalid,TResult? Function( String message)?  storage,TResult? Function( String message)?  network,TResult? Function()?  signedOut,TResult? Function( String message)?  auth,TResult? Function( String message)?  internal,}) {final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound();case KansoError_Locked() when locked != null:
return locked();case KansoError_Conflict() when conflict != null:
return conflict(_that.currentRev);case KansoError_Invalid() when invalid != null:
return invalid(_that.message);case KansoError_Storage() when storage != null:
return storage(_that.message);case KansoError_Network() when network != null:
return network(_that.message);case KansoError_SignedOut() when signedOut != null:
return signedOut();case KansoError_Auth() when auth != null:
return auth(_that.message);case KansoError_Internal() when internal != null:
return internal(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class KansoError_NotFound extends KansoError {
  const KansoError_NotFound(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_NotFound);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.notFound()';
}


}




/// @nodoc


class KansoError_Locked extends KansoError {
  const KansoError_Locked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Locked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.locked()';
}


}




/// @nodoc


class KansoError_Conflict extends KansoError {
  const KansoError_Conflict({required this.currentRev}): super._();
  

 final  String currentRev;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_ConflictCopyWith<KansoError_Conflict> get copyWith => _$KansoError_ConflictCopyWithImpl<KansoError_Conflict>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Conflict&&(identical(other.currentRev, currentRev) || other.currentRev == currentRev));
}


@override
int get hashCode => Object.hash(runtimeType,currentRev);

@override
String toString() {
  return 'KansoError.conflict(currentRev: $currentRev)';
}


}

/// @nodoc
abstract mixin class $KansoError_ConflictCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_ConflictCopyWith(KansoError_Conflict value, $Res Function(KansoError_Conflict) _then) = _$KansoError_ConflictCopyWithImpl;
@useResult
$Res call({
 String currentRev
});




}
/// @nodoc
class _$KansoError_ConflictCopyWithImpl<$Res>
    implements $KansoError_ConflictCopyWith<$Res> {
  _$KansoError_ConflictCopyWithImpl(this._self, this._then);

  final KansoError_Conflict _self;
  final $Res Function(KansoError_Conflict) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? currentRev = null,}) {
  return _then(KansoError_Conflict(
currentRev: null == currentRev ? _self.currentRev : currentRev // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Invalid extends KansoError {
  const KansoError_Invalid({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_InvalidCopyWith<KansoError_Invalid> get copyWith => _$KansoError_InvalidCopyWithImpl<KansoError_Invalid>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Invalid&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.invalid(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_InvalidCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_InvalidCopyWith(KansoError_Invalid value, $Res Function(KansoError_Invalid) _then) = _$KansoError_InvalidCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_InvalidCopyWithImpl<$Res>
    implements $KansoError_InvalidCopyWith<$Res> {
  _$KansoError_InvalidCopyWithImpl(this._self, this._then);

  final KansoError_Invalid _self;
  final $Res Function(KansoError_Invalid) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Invalid(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Storage extends KansoError {
  const KansoError_Storage({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_StorageCopyWith<KansoError_Storage> get copyWith => _$KansoError_StorageCopyWithImpl<KansoError_Storage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Storage&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.storage(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_StorageCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_StorageCopyWith(KansoError_Storage value, $Res Function(KansoError_Storage) _then) = _$KansoError_StorageCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_StorageCopyWithImpl<$Res>
    implements $KansoError_StorageCopyWith<$Res> {
  _$KansoError_StorageCopyWithImpl(this._self, this._then);

  final KansoError_Storage _self;
  final $Res Function(KansoError_Storage) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Storage(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Network extends KansoError {
  const KansoError_Network({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_NetworkCopyWith<KansoError_Network> get copyWith => _$KansoError_NetworkCopyWithImpl<KansoError_Network>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Network&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.network(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_NetworkCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_NetworkCopyWith(KansoError_Network value, $Res Function(KansoError_Network) _then) = _$KansoError_NetworkCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_NetworkCopyWithImpl<$Res>
    implements $KansoError_NetworkCopyWith<$Res> {
  _$KansoError_NetworkCopyWithImpl(this._self, this._then);

  final KansoError_Network _self;
  final $Res Function(KansoError_Network) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Network(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_SignedOut extends KansoError {
  const KansoError_SignedOut(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_SignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.signedOut()';
}


}




/// @nodoc


class KansoError_Auth extends KansoError {
  const KansoError_Auth({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_AuthCopyWith<KansoError_Auth> get copyWith => _$KansoError_AuthCopyWithImpl<KansoError_Auth>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Auth&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.auth(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_AuthCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_AuthCopyWith(KansoError_Auth value, $Res Function(KansoError_Auth) _then) = _$KansoError_AuthCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_AuthCopyWithImpl<$Res>
    implements $KansoError_AuthCopyWith<$Res> {
  _$KansoError_AuthCopyWithImpl(this._self, this._then);

  final KansoError_Auth _self;
  final $Res Function(KansoError_Auth) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Auth(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Internal extends KansoError {
  const KansoError_Internal({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_InternalCopyWith<KansoError_Internal> get copyWith => _$KansoError_InternalCopyWithImpl<KansoError_Internal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Internal&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.internal(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_InternalCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_InternalCopyWith(KansoError_Internal value, $Res Function(KansoError_Internal) _then) = _$KansoError_InternalCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_InternalCopyWithImpl<$Res>
    implements $KansoError_InternalCopyWith<$Res> {
  _$KansoError_InternalCopyWithImpl(this._self, this._then);

  final KansoError_Internal _self;
  final $Res Function(KansoError_Internal) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Internal(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$LinkTarget {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LinkTarget()';
}


}

/// @nodoc
class $LinkTargetCopyWith<$Res>  {
$LinkTargetCopyWith(LinkTarget _, $Res Function(LinkTarget) __);
}


/// Adds pattern-matching-related methods to [LinkTarget].
extension LinkTargetPatterns on LinkTarget {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LinkTarget_Note value)?  note,TResult Function( LinkTarget_NewNote value)?  newNote,TResult Function( LinkTarget_Web value)?  web,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that);case LinkTarget_NewNote() when newNote != null:
return newNote(_that);case LinkTarget_Web() when web != null:
return web(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LinkTarget_Note value)  note,required TResult Function( LinkTarget_NewNote value)  newNote,required TResult Function( LinkTarget_Web value)  web,}){
final _that = this;
switch (_that) {
case LinkTarget_Note():
return note(_that);case LinkTarget_NewNote():
return newNote(_that);case LinkTarget_Web():
return web(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LinkTarget_Note value)?  note,TResult? Function( LinkTarget_NewNote value)?  newNote,TResult? Function( LinkTarget_Web value)?  web,}){
final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that);case LinkTarget_NewNote() when newNote != null:
return newNote(_that);case LinkTarget_Web() when web != null:
return web(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id)?  note,TResult Function( String title)?  newNote,TResult Function( String url)?  web,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that.id);case LinkTarget_NewNote() when newNote != null:
return newNote(_that.title);case LinkTarget_Web() when web != null:
return web(_that.url);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id)  note,required TResult Function( String title)  newNote,required TResult Function( String url)  web,}) {final _that = this;
switch (_that) {
case LinkTarget_Note():
return note(_that.id);case LinkTarget_NewNote():
return newNote(_that.title);case LinkTarget_Web():
return web(_that.url);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id)?  note,TResult? Function( String title)?  newNote,TResult? Function( String url)?  web,}) {final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that.id);case LinkTarget_NewNote() when newNote != null:
return newNote(_that.title);case LinkTarget_Web() when web != null:
return web(_that.url);case _:
  return null;

}
}

}

/// @nodoc


class LinkTarget_Note extends LinkTarget {
  const LinkTarget_Note({required this.id}): super._();
  

 final  String id;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkTarget_NoteCopyWith<LinkTarget_Note> get copyWith => _$LinkTarget_NoteCopyWithImpl<LinkTarget_Note>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget_Note&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'LinkTarget.note(id: $id)';
}


}

/// @nodoc
abstract mixin class $LinkTarget_NoteCopyWith<$Res> implements $LinkTargetCopyWith<$Res> {
  factory $LinkTarget_NoteCopyWith(LinkTarget_Note value, $Res Function(LinkTarget_Note) _then) = _$LinkTarget_NoteCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$LinkTarget_NoteCopyWithImpl<$Res>
    implements $LinkTarget_NoteCopyWith<$Res> {
  _$LinkTarget_NoteCopyWithImpl(this._self, this._then);

  final LinkTarget_Note _self;
  final $Res Function(LinkTarget_Note) _then;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(LinkTarget_Note(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LinkTarget_NewNote extends LinkTarget {
  const LinkTarget_NewNote({required this.title}): super._();
  

 final  String title;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkTarget_NewNoteCopyWith<LinkTarget_NewNote> get copyWith => _$LinkTarget_NewNoteCopyWithImpl<LinkTarget_NewNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget_NewNote&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'LinkTarget.newNote(title: $title)';
}


}

/// @nodoc
abstract mixin class $LinkTarget_NewNoteCopyWith<$Res> implements $LinkTargetCopyWith<$Res> {
  factory $LinkTarget_NewNoteCopyWith(LinkTarget_NewNote value, $Res Function(LinkTarget_NewNote) _then) = _$LinkTarget_NewNoteCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$LinkTarget_NewNoteCopyWithImpl<$Res>
    implements $LinkTarget_NewNoteCopyWith<$Res> {
  _$LinkTarget_NewNoteCopyWithImpl(this._self, this._then);

  final LinkTarget_NewNote _self;
  final $Res Function(LinkTarget_NewNote) _then;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(LinkTarget_NewNote(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LinkTarget_Web extends LinkTarget {
  const LinkTarget_Web({required this.url}): super._();
  

 final  String url;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkTarget_WebCopyWith<LinkTarget_Web> get copyWith => _$LinkTarget_WebCopyWithImpl<LinkTarget_Web>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget_Web&&(identical(other.url, url) || other.url == url));
}


@override
int get hashCode => Object.hash(runtimeType,url);

@override
String toString() {
  return 'LinkTarget.web(url: $url)';
}


}

/// @nodoc
abstract mixin class $LinkTarget_WebCopyWith<$Res> implements $LinkTargetCopyWith<$Res> {
  factory $LinkTarget_WebCopyWith(LinkTarget_Web value, $Res Function(LinkTarget_Web) _then) = _$LinkTarget_WebCopyWithImpl;
@useResult
$Res call({
 String url
});




}
/// @nodoc
class _$LinkTarget_WebCopyWithImpl<$Res>
    implements $LinkTarget_WebCopyWith<$Res> {
  _$LinkTarget_WebCopyWithImpl(this._self, this._then);

  final LinkTarget_Web _self;
  final $Res Function(LinkTarget_Web) _then;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,}) {
  return _then(LinkTarget_Web(
url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ListPlace {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace()';
}


}

/// @nodoc
class $ListPlaceCopyWith<$Res>  {
$ListPlaceCopyWith(ListPlace _, $Res Function(ListPlace) __);
}


/// Adds pattern-matching-related methods to [ListPlace].
extension ListPlacePatterns on ListPlace {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ListPlace_All value)?  all,TResult Function( ListPlace_Pinned value)?  pinned,TResult Function( ListPlace_Recent value)?  recent,TResult Function( ListPlace_Notebook value)?  notebook,TResult Function( ListPlace_Tag value)?  tag,TResult Function( ListPlace_Status value)?  status,TResult Function( ListPlace_TagPath value)?  tagPath,TResult Function( ListPlace_Trash value)?  trash,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all(_that);case ListPlace_Pinned() when pinned != null:
return pinned(_that);case ListPlace_Recent() when recent != null:
return recent(_that);case ListPlace_Notebook() when notebook != null:
return notebook(_that);case ListPlace_Tag() when tag != null:
return tag(_that);case ListPlace_Status() when status != null:
return status(_that);case ListPlace_TagPath() when tagPath != null:
return tagPath(_that);case ListPlace_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ListPlace_All value)  all,required TResult Function( ListPlace_Pinned value)  pinned,required TResult Function( ListPlace_Recent value)  recent,required TResult Function( ListPlace_Notebook value)  notebook,required TResult Function( ListPlace_Tag value)  tag,required TResult Function( ListPlace_Status value)  status,required TResult Function( ListPlace_TagPath value)  tagPath,required TResult Function( ListPlace_Trash value)  trash,}){
final _that = this;
switch (_that) {
case ListPlace_All():
return all(_that);case ListPlace_Pinned():
return pinned(_that);case ListPlace_Recent():
return recent(_that);case ListPlace_Notebook():
return notebook(_that);case ListPlace_Tag():
return tag(_that);case ListPlace_Status():
return status(_that);case ListPlace_TagPath():
return tagPath(_that);case ListPlace_Trash():
return trash(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ListPlace_All value)?  all,TResult? Function( ListPlace_Pinned value)?  pinned,TResult? Function( ListPlace_Recent value)?  recent,TResult? Function( ListPlace_Notebook value)?  notebook,TResult? Function( ListPlace_Tag value)?  tag,TResult? Function( ListPlace_Status value)?  status,TResult? Function( ListPlace_TagPath value)?  tagPath,TResult? Function( ListPlace_Trash value)?  trash,}){
final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all(_that);case ListPlace_Pinned() when pinned != null:
return pinned(_that);case ListPlace_Recent() when recent != null:
return recent(_that);case ListPlace_Notebook() when notebook != null:
return notebook(_that);case ListPlace_Tag() when tag != null:
return tag(_that);case ListPlace_Status() when status != null:
return status(_that);case ListPlace_TagPath() when tagPath != null:
return tagPath(_that);case ListPlace_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function()?  pinned,TResult Function()?  recent,TResult Function( String id)?  notebook,TResult Function( String id)?  tag,TResult Function( String status)?  status,TResult Function( String path)?  tagPath,TResult Function()?  trash,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all();case ListPlace_Pinned() when pinned != null:
return pinned();case ListPlace_Recent() when recent != null:
return recent();case ListPlace_Notebook() when notebook != null:
return notebook(_that.id);case ListPlace_Tag() when tag != null:
return tag(_that.id);case ListPlace_Status() when status != null:
return status(_that.status);case ListPlace_TagPath() when tagPath != null:
return tagPath(_that.path);case ListPlace_Trash() when trash != null:
return trash();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function()  pinned,required TResult Function()  recent,required TResult Function( String id)  notebook,required TResult Function( String id)  tag,required TResult Function( String status)  status,required TResult Function( String path)  tagPath,required TResult Function()  trash,}) {final _that = this;
switch (_that) {
case ListPlace_All():
return all();case ListPlace_Pinned():
return pinned();case ListPlace_Recent():
return recent();case ListPlace_Notebook():
return notebook(_that.id);case ListPlace_Tag():
return tag(_that.id);case ListPlace_Status():
return status(_that.status);case ListPlace_TagPath():
return tagPath(_that.path);case ListPlace_Trash():
return trash();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function()?  pinned,TResult? Function()?  recent,TResult? Function( String id)?  notebook,TResult? Function( String id)?  tag,TResult? Function( String status)?  status,TResult? Function( String path)?  tagPath,TResult? Function()?  trash,}) {final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all();case ListPlace_Pinned() when pinned != null:
return pinned();case ListPlace_Recent() when recent != null:
return recent();case ListPlace_Notebook() when notebook != null:
return notebook(_that.id);case ListPlace_Tag() when tag != null:
return tag(_that.id);case ListPlace_Status() when status != null:
return status(_that.status);case ListPlace_TagPath() when tagPath != null:
return tagPath(_that.path);case ListPlace_Trash() when trash != null:
return trash();case _:
  return null;

}
}

}

/// @nodoc


class ListPlace_All extends ListPlace {
  const ListPlace_All(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.all()';
}


}




/// @nodoc


class ListPlace_Pinned extends ListPlace {
  const ListPlace_Pinned(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Pinned);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.pinned()';
}


}




/// @nodoc


class ListPlace_Recent extends ListPlace {
  const ListPlace_Recent(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Recent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.recent()';
}


}




/// @nodoc


class ListPlace_Notebook extends ListPlace {
  const ListPlace_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_NotebookCopyWith<ListPlace_Notebook> get copyWith => _$ListPlace_NotebookCopyWithImpl<ListPlace_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ListPlace.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $ListPlace_NotebookCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_NotebookCopyWith(ListPlace_Notebook value, $Res Function(ListPlace_Notebook) _then) = _$ListPlace_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ListPlace_NotebookCopyWithImpl<$Res>
    implements $ListPlace_NotebookCopyWith<$Res> {
  _$ListPlace_NotebookCopyWithImpl(this._self, this._then);

  final ListPlace_Notebook _self;
  final $Res Function(ListPlace_Notebook) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ListPlace_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Tag extends ListPlace {
  const ListPlace_Tag({required this.id}): super._();
  

 final  String id;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_TagCopyWith<ListPlace_Tag> get copyWith => _$ListPlace_TagCopyWithImpl<ListPlace_Tag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Tag&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ListPlace.tag(id: $id)';
}


}

/// @nodoc
abstract mixin class $ListPlace_TagCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_TagCopyWith(ListPlace_Tag value, $Res Function(ListPlace_Tag) _then) = _$ListPlace_TagCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ListPlace_TagCopyWithImpl<$Res>
    implements $ListPlace_TagCopyWith<$Res> {
  _$ListPlace_TagCopyWithImpl(this._self, this._then);

  final ListPlace_Tag _self;
  final $Res Function(ListPlace_Tag) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ListPlace_Tag(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Status extends ListPlace {
  const ListPlace_Status({required this.status}): super._();
  

 final  String status;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_StatusCopyWith<ListPlace_Status> get copyWith => _$ListPlace_StatusCopyWithImpl<ListPlace_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Status&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,status);

@override
String toString() {
  return 'ListPlace.status(status: $status)';
}


}

/// @nodoc
abstract mixin class $ListPlace_StatusCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_StatusCopyWith(ListPlace_Status value, $Res Function(ListPlace_Status) _then) = _$ListPlace_StatusCopyWithImpl;
@useResult
$Res call({
 String status
});




}
/// @nodoc
class _$ListPlace_StatusCopyWithImpl<$Res>
    implements $ListPlace_StatusCopyWith<$Res> {
  _$ListPlace_StatusCopyWithImpl(this._self, this._then);

  final ListPlace_Status _self;
  final $Res Function(ListPlace_Status) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? status = null,}) {
  return _then(ListPlace_Status(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_TagPath extends ListPlace {
  const ListPlace_TagPath({required this.path}): super._();
  

 final  String path;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_TagPathCopyWith<ListPlace_TagPath> get copyWith => _$ListPlace_TagPathCopyWithImpl<ListPlace_TagPath>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_TagPath&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'ListPlace.tagPath(path: $path)';
}


}

/// @nodoc
abstract mixin class $ListPlace_TagPathCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_TagPathCopyWith(ListPlace_TagPath value, $Res Function(ListPlace_TagPath) _then) = _$ListPlace_TagPathCopyWithImpl;
@useResult
$Res call({
 String path
});




}
/// @nodoc
class _$ListPlace_TagPathCopyWithImpl<$Res>
    implements $ListPlace_TagPathCopyWith<$Res> {
  _$ListPlace_TagPathCopyWithImpl(this._self, this._then);

  final ListPlace_TagPath _self;
  final $Res Function(ListPlace_TagPath) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? path = null,}) {
  return _then(ListPlace_TagPath(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Trash extends ListPlace {
  const ListPlace_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.trash()';
}


}




/// @nodoc
mixin _$LockEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockEvent()';
}


}

/// @nodoc
class $LockEventCopyWith<$Res>  {
$LockEventCopyWith(LockEvent _, $Res Function(LockEvent) __);
}


/// Adds pattern-matching-related methods to [LockEvent].
extension LockEventPatterns on LockEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockEvent_Unlocked value)?  unlocked,TResult Function( LockEvent_Locked value)?  locked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked(_that);case LockEvent_Locked() when locked != null:
return locked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockEvent_Unlocked value)  unlocked,required TResult Function( LockEvent_Locked value)  locked,}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked():
return unlocked(_that);case LockEvent_Locked():
return locked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockEvent_Unlocked value)?  unlocked,TResult? Function( LockEvent_Locked value)?  locked,}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked(_that);case LockEvent_Locked() when locked != null:
return locked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  unlocked,TResult Function( LockReason reason)?  locked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked();case LockEvent_Locked() when locked != null:
return locked(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  unlocked,required TResult Function( LockReason reason)  locked,}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked():
return unlocked();case LockEvent_Locked():
return locked(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  unlocked,TResult? Function( LockReason reason)?  locked,}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked();case LockEvent_Locked() when locked != null:
return locked(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class LockEvent_Unlocked extends LockEvent {
  const LockEvent_Unlocked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent_Unlocked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockEvent.unlocked()';
}


}




/// @nodoc


class LockEvent_Locked extends LockEvent {
  const LockEvent_Locked({required this.reason}): super._();
  

 final  LockReason reason;

/// Create a copy of LockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockEvent_LockedCopyWith<LockEvent_Locked> get copyWith => _$LockEvent_LockedCopyWithImpl<LockEvent_Locked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent_Locked&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'LockEvent.locked(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $LockEvent_LockedCopyWith<$Res> implements $LockEventCopyWith<$Res> {
  factory $LockEvent_LockedCopyWith(LockEvent_Locked value, $Res Function(LockEvent_Locked) _then) = _$LockEvent_LockedCopyWithImpl;
@useResult
$Res call({
 LockReason reason
});




}
/// @nodoc
class _$LockEvent_LockedCopyWithImpl<$Res>
    implements $LockEvent_LockedCopyWith<$Res> {
  _$LockEvent_LockedCopyWithImpl(this._self, this._then);

  final LockEvent_Locked _self;
  final $Res Function(LockEvent_Locked) _then;

/// Create a copy of LockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(LockEvent_Locked(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as LockReason,
  ));
}


}

/// @nodoc
mixin _$LockState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState()';
}


}

/// @nodoc
class $LockStateCopyWith<$Res>  {
$LockStateCopyWith(LockState _, $Res Function(LockState) __);
}


/// Adds pattern-matching-related methods to [LockState].
extension LockStatePatterns on LockState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockState_Plain value)?  plain,TResult Function( LockState_Open value)?  open,TResult Function( LockState_Closed value)?  closed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain(_that);case LockState_Open() when open != null:
return open(_that);case LockState_Closed() when closed != null:
return closed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockState_Plain value)  plain,required TResult Function( LockState_Open value)  open,required TResult Function( LockState_Closed value)  closed,}){
final _that = this;
switch (_that) {
case LockState_Plain():
return plain(_that);case LockState_Open():
return open(_that);case LockState_Closed():
return closed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockState_Plain value)?  plain,TResult? Function( LockState_Open value)?  open,TResult? Function( LockState_Closed value)?  closed,}){
final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain(_that);case LockState_Open() when open != null:
return open(_that);case LockState_Closed() when closed != null:
return closed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  plain,TResult Function()?  open,TResult Function( String? reason)?  closed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain();case LockState_Open() when open != null:
return open();case LockState_Closed() when closed != null:
return closed(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  plain,required TResult Function()  open,required TResult Function( String? reason)  closed,}) {final _that = this;
switch (_that) {
case LockState_Plain():
return plain();case LockState_Open():
return open();case LockState_Closed():
return closed(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  plain,TResult? Function()?  open,TResult? Function( String? reason)?  closed,}) {final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain();case LockState_Open() when open != null:
return open();case LockState_Closed() when closed != null:
return closed(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class LockState_Plain extends LockState {
  const LockState_Plain(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Plain);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState.plain()';
}


}




/// @nodoc


class LockState_Open extends LockState {
  const LockState_Open(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Open);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState.open()';
}


}




/// @nodoc


class LockState_Closed extends LockState {
  const LockState_Closed({this.reason}): super._();
  

 final  String? reason;

/// Create a copy of LockState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockState_ClosedCopyWith<LockState_Closed> get copyWith => _$LockState_ClosedCopyWithImpl<LockState_Closed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Closed&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'LockState.closed(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $LockState_ClosedCopyWith<$Res> implements $LockStateCopyWith<$Res> {
  factory $LockState_ClosedCopyWith(LockState_Closed value, $Res Function(LockState_Closed) _then) = _$LockState_ClosedCopyWithImpl;
@useResult
$Res call({
 String? reason
});




}
/// @nodoc
class _$LockState_ClosedCopyWithImpl<$Res>
    implements $LockState_ClosedCopyWith<$Res> {
  _$LockState_ClosedCopyWithImpl(this._self, this._then);

  final LockState_Closed _self;
  final $Res Function(LockState_Closed) _then;

/// Create a copy of LockState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = freezed,}) {
  return _then(LockState_Closed(
reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$NoteChange {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange()';
}


}

/// @nodoc
class $NoteChangeCopyWith<$Res>  {
$NoteChangeCopyWith(NoteChange _, $Res Function(NoteChange) __);
}


/// Adds pattern-matching-related methods to [NoteChange].
extension NoteChangePatterns on NoteChange {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteChange_TogglePin value)?  togglePin,TResult Function( NoteChange_Move value)?  move,TResult Function( NoteChange_Trash value)?  trash,TResult Function( NoteChange_SetStatus value)?  setStatus,TResult Function( NoteChange_AddTag value)?  addTag,TResult Function( NoteChange_RemoveTag value)?  removeTag,TResult Function( NoteChange_Restore value)?  restore,TResult Function( NoteChange_DeleteForever value)?  deleteForever,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin(_that);case NoteChange_Move() when move != null:
return move(_that);case NoteChange_Trash() when trash != null:
return trash(_that);case NoteChange_SetStatus() when setStatus != null:
return setStatus(_that);case NoteChange_AddTag() when addTag != null:
return addTag(_that);case NoteChange_RemoveTag() when removeTag != null:
return removeTag(_that);case NoteChange_Restore() when restore != null:
return restore(_that);case NoteChange_DeleteForever() when deleteForever != null:
return deleteForever(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteChange_TogglePin value)  togglePin,required TResult Function( NoteChange_Move value)  move,required TResult Function( NoteChange_Trash value)  trash,required TResult Function( NoteChange_SetStatus value)  setStatus,required TResult Function( NoteChange_AddTag value)  addTag,required TResult Function( NoteChange_RemoveTag value)  removeTag,required TResult Function( NoteChange_Restore value)  restore,required TResult Function( NoteChange_DeleteForever value)  deleteForever,}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin():
return togglePin(_that);case NoteChange_Move():
return move(_that);case NoteChange_Trash():
return trash(_that);case NoteChange_SetStatus():
return setStatus(_that);case NoteChange_AddTag():
return addTag(_that);case NoteChange_RemoveTag():
return removeTag(_that);case NoteChange_Restore():
return restore(_that);case NoteChange_DeleteForever():
return deleteForever(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteChange_TogglePin value)?  togglePin,TResult? Function( NoteChange_Move value)?  move,TResult? Function( NoteChange_Trash value)?  trash,TResult? Function( NoteChange_SetStatus value)?  setStatus,TResult? Function( NoteChange_AddTag value)?  addTag,TResult? Function( NoteChange_RemoveTag value)?  removeTag,TResult? Function( NoteChange_Restore value)?  restore,TResult? Function( NoteChange_DeleteForever value)?  deleteForever,}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin(_that);case NoteChange_Move() when move != null:
return move(_that);case NoteChange_Trash() when trash != null:
return trash(_that);case NoteChange_SetStatus() when setStatus != null:
return setStatus(_that);case NoteChange_AddTag() when addTag != null:
return addTag(_that);case NoteChange_RemoveTag() when removeTag != null:
return removeTag(_that);case NoteChange_Restore() when restore != null:
return restore(_that);case NoteChange_DeleteForever() when deleteForever != null:
return deleteForever(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  togglePin,TResult Function( String notebookId)?  move,TResult Function()?  trash,TResult Function( String status)?  setStatus,TResult Function( String tagId)?  addTag,TResult Function( String tagId)?  removeTag,TResult Function()?  restore,TResult Function()?  deleteForever,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin();case NoteChange_Move() when move != null:
return move(_that.notebookId);case NoteChange_Trash() when trash != null:
return trash();case NoteChange_SetStatus() when setStatus != null:
return setStatus(_that.status);case NoteChange_AddTag() when addTag != null:
return addTag(_that.tagId);case NoteChange_RemoveTag() when removeTag != null:
return removeTag(_that.tagId);case NoteChange_Restore() when restore != null:
return restore();case NoteChange_DeleteForever() when deleteForever != null:
return deleteForever();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  togglePin,required TResult Function( String notebookId)  move,required TResult Function()  trash,required TResult Function( String status)  setStatus,required TResult Function( String tagId)  addTag,required TResult Function( String tagId)  removeTag,required TResult Function()  restore,required TResult Function()  deleteForever,}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin():
return togglePin();case NoteChange_Move():
return move(_that.notebookId);case NoteChange_Trash():
return trash();case NoteChange_SetStatus():
return setStatus(_that.status);case NoteChange_AddTag():
return addTag(_that.tagId);case NoteChange_RemoveTag():
return removeTag(_that.tagId);case NoteChange_Restore():
return restore();case NoteChange_DeleteForever():
return deleteForever();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  togglePin,TResult? Function( String notebookId)?  move,TResult? Function()?  trash,TResult? Function( String status)?  setStatus,TResult? Function( String tagId)?  addTag,TResult? Function( String tagId)?  removeTag,TResult? Function()?  restore,TResult? Function()?  deleteForever,}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin();case NoteChange_Move() when move != null:
return move(_that.notebookId);case NoteChange_Trash() when trash != null:
return trash();case NoteChange_SetStatus() when setStatus != null:
return setStatus(_that.status);case NoteChange_AddTag() when addTag != null:
return addTag(_that.tagId);case NoteChange_RemoveTag() when removeTag != null:
return removeTag(_that.tagId);case NoteChange_Restore() when restore != null:
return restore();case NoteChange_DeleteForever() when deleteForever != null:
return deleteForever();case _:
  return null;

}
}

}

/// @nodoc


class NoteChange_TogglePin extends NoteChange {
  const NoteChange_TogglePin(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_TogglePin);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.togglePin()';
}


}




/// @nodoc


class NoteChange_Move extends NoteChange {
  const NoteChange_Move({required this.notebookId}): super._();
  

 final  String notebookId;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChange_MoveCopyWith<NoteChange_Move> get copyWith => _$NoteChange_MoveCopyWithImpl<NoteChange_Move>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Move&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'NoteChange.move(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $NoteChange_MoveCopyWith<$Res> implements $NoteChangeCopyWith<$Res> {
  factory $NoteChange_MoveCopyWith(NoteChange_Move value, $Res Function(NoteChange_Move) _then) = _$NoteChange_MoveCopyWithImpl;
@useResult
$Res call({
 String notebookId
});




}
/// @nodoc
class _$NoteChange_MoveCopyWithImpl<$Res>
    implements $NoteChange_MoveCopyWith<$Res> {
  _$NoteChange_MoveCopyWithImpl(this._self, this._then);

  final NoteChange_Move _self;
  final $Res Function(NoteChange_Move) _then;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = null,}) {
  return _then(NoteChange_Move(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChange_Trash extends NoteChange {
  const NoteChange_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.trash()';
}


}




/// @nodoc


class NoteChange_SetStatus extends NoteChange {
  const NoteChange_SetStatus({required this.status}): super._();
  

 final  String status;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChange_SetStatusCopyWith<NoteChange_SetStatus> get copyWith => _$NoteChange_SetStatusCopyWithImpl<NoteChange_SetStatus>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_SetStatus&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,status);

@override
String toString() {
  return 'NoteChange.setStatus(status: $status)';
}


}

/// @nodoc
abstract mixin class $NoteChange_SetStatusCopyWith<$Res> implements $NoteChangeCopyWith<$Res> {
  factory $NoteChange_SetStatusCopyWith(NoteChange_SetStatus value, $Res Function(NoteChange_SetStatus) _then) = _$NoteChange_SetStatusCopyWithImpl;
@useResult
$Res call({
 String status
});




}
/// @nodoc
class _$NoteChange_SetStatusCopyWithImpl<$Res>
    implements $NoteChange_SetStatusCopyWith<$Res> {
  _$NoteChange_SetStatusCopyWithImpl(this._self, this._then);

  final NoteChange_SetStatus _self;
  final $Res Function(NoteChange_SetStatus) _then;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? status = null,}) {
  return _then(NoteChange_SetStatus(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChange_AddTag extends NoteChange {
  const NoteChange_AddTag({required this.tagId}): super._();
  

 final  String tagId;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChange_AddTagCopyWith<NoteChange_AddTag> get copyWith => _$NoteChange_AddTagCopyWithImpl<NoteChange_AddTag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_AddTag&&(identical(other.tagId, tagId) || other.tagId == tagId));
}


@override
int get hashCode => Object.hash(runtimeType,tagId);

@override
String toString() {
  return 'NoteChange.addTag(tagId: $tagId)';
}


}

/// @nodoc
abstract mixin class $NoteChange_AddTagCopyWith<$Res> implements $NoteChangeCopyWith<$Res> {
  factory $NoteChange_AddTagCopyWith(NoteChange_AddTag value, $Res Function(NoteChange_AddTag) _then) = _$NoteChange_AddTagCopyWithImpl;
@useResult
$Res call({
 String tagId
});




}
/// @nodoc
class _$NoteChange_AddTagCopyWithImpl<$Res>
    implements $NoteChange_AddTagCopyWith<$Res> {
  _$NoteChange_AddTagCopyWithImpl(this._self, this._then);

  final NoteChange_AddTag _self;
  final $Res Function(NoteChange_AddTag) _then;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? tagId = null,}) {
  return _then(NoteChange_AddTag(
tagId: null == tagId ? _self.tagId : tagId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChange_RemoveTag extends NoteChange {
  const NoteChange_RemoveTag({required this.tagId}): super._();
  

 final  String tagId;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChange_RemoveTagCopyWith<NoteChange_RemoveTag> get copyWith => _$NoteChange_RemoveTagCopyWithImpl<NoteChange_RemoveTag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_RemoveTag&&(identical(other.tagId, tagId) || other.tagId == tagId));
}


@override
int get hashCode => Object.hash(runtimeType,tagId);

@override
String toString() {
  return 'NoteChange.removeTag(tagId: $tagId)';
}


}

/// @nodoc
abstract mixin class $NoteChange_RemoveTagCopyWith<$Res> implements $NoteChangeCopyWith<$Res> {
  factory $NoteChange_RemoveTagCopyWith(NoteChange_RemoveTag value, $Res Function(NoteChange_RemoveTag) _then) = _$NoteChange_RemoveTagCopyWithImpl;
@useResult
$Res call({
 String tagId
});




}
/// @nodoc
class _$NoteChange_RemoveTagCopyWithImpl<$Res>
    implements $NoteChange_RemoveTagCopyWith<$Res> {
  _$NoteChange_RemoveTagCopyWithImpl(this._self, this._then);

  final NoteChange_RemoveTag _self;
  final $Res Function(NoteChange_RemoveTag) _then;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? tagId = null,}) {
  return _then(NoteChange_RemoveTag(
tagId: null == tagId ? _self.tagId : tagId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChange_Restore extends NoteChange {
  const NoteChange_Restore(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Restore);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.restore()';
}


}




/// @nodoc


class NoteChange_DeleteForever extends NoteChange {
  const NoteChange_DeleteForever(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_DeleteForever);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.deleteForever()';
}


}




/// @nodoc
mixin _$PersonLookup {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PersonLookup);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PersonLookup()';
}


}

/// @nodoc
class $PersonLookupCopyWith<$Res>  {
$PersonLookupCopyWith(PersonLookup _, $Res Function(PersonLookup) __);
}


/// Adds pattern-matching-related methods to [PersonLookup].
extension PersonLookupPatterns on PersonLookup {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PersonLookup_Found value)?  found,TResult Function( PersonLookup_Changed value)?  changed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that);case PersonLookup_Changed() when changed != null:
return changed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PersonLookup_Found value)  found,required TResult Function( PersonLookup_Changed value)  changed,}){
final _that = this;
switch (_that) {
case PersonLookup_Found():
return found(_that);case PersonLookup_Changed():
return changed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PersonLookup_Found value)?  found,TResult? Function( PersonLookup_Changed value)?  changed,}){
final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that);case PersonLookup_Changed() when changed != null:
return changed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PersonRow field0)?  found,TResult Function( PersonRow? pinned,  PersonRow offered)?  changed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that.field0);case PersonLookup_Changed() when changed != null:
return changed(_that.pinned,_that.offered);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PersonRow field0)  found,required TResult Function( PersonRow? pinned,  PersonRow offered)  changed,}) {final _that = this;
switch (_that) {
case PersonLookup_Found():
return found(_that.field0);case PersonLookup_Changed():
return changed(_that.pinned,_that.offered);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PersonRow field0)?  found,TResult? Function( PersonRow? pinned,  PersonRow offered)?  changed,}) {final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that.field0);case PersonLookup_Changed() when changed != null:
return changed(_that.pinned,_that.offered);case _:
  return null;

}
}

}

/// @nodoc


class PersonLookup_Found extends PersonLookup {
  const PersonLookup_Found(this.field0): super._();
  

 final  PersonRow field0;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PersonLookup_FoundCopyWith<PersonLookup_Found> get copyWith => _$PersonLookup_FoundCopyWithImpl<PersonLookup_Found>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PersonLookup_Found&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'PersonLookup.found(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PersonLookup_FoundCopyWith<$Res> implements $PersonLookupCopyWith<$Res> {
  factory $PersonLookup_FoundCopyWith(PersonLookup_Found value, $Res Function(PersonLookup_Found) _then) = _$PersonLookup_FoundCopyWithImpl;
@useResult
$Res call({
 PersonRow field0
});




}
/// @nodoc
class _$PersonLookup_FoundCopyWithImpl<$Res>
    implements $PersonLookup_FoundCopyWith<$Res> {
  _$PersonLookup_FoundCopyWithImpl(this._self, this._then);

  final PersonLookup_Found _self;
  final $Res Function(PersonLookup_Found) _then;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PersonLookup_Found(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as PersonRow,
  ));
}


}

/// @nodoc


class PersonLookup_Changed extends PersonLookup {
  const PersonLookup_Changed({this.pinned, required this.offered}): super._();
  

 final  PersonRow? pinned;
 final  PersonRow offered;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PersonLookup_ChangedCopyWith<PersonLookup_Changed> get copyWith => _$PersonLookup_ChangedCopyWithImpl<PersonLookup_Changed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PersonLookup_Changed&&(identical(other.pinned, pinned) || other.pinned == pinned)&&(identical(other.offered, offered) || other.offered == offered));
}


@override
int get hashCode => Object.hash(runtimeType,pinned,offered);

@override
String toString() {
  return 'PersonLookup.changed(pinned: $pinned, offered: $offered)';
}


}

/// @nodoc
abstract mixin class $PersonLookup_ChangedCopyWith<$Res> implements $PersonLookupCopyWith<$Res> {
  factory $PersonLookup_ChangedCopyWith(PersonLookup_Changed value, $Res Function(PersonLookup_Changed) _then) = _$PersonLookup_ChangedCopyWithImpl;
@useResult
$Res call({
 PersonRow? pinned, PersonRow offered
});




}
/// @nodoc
class _$PersonLookup_ChangedCopyWithImpl<$Res>
    implements $PersonLookup_ChangedCopyWith<$Res> {
  _$PersonLookup_ChangedCopyWithImpl(this._self, this._then);

  final PersonLookup_Changed _self;
  final $Res Function(PersonLookup_Changed) _then;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? pinned = freezed,Object? offered = null,}) {
  return _then(PersonLookup_Changed(
pinned: freezed == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as PersonRow?,offered: null == offered ? _self.offered : offered // ignore: cast_nullable_to_non_nullable
as PersonRow,
  ));
}


}

/// @nodoc
mixin _$PreviewLink {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewLink()';
}


}

/// @nodoc
class $PreviewLinkCopyWith<$Res>  {
$PreviewLinkCopyWith(PreviewLink _, $Res Function(PreviewLink) __);
}


/// Adds pattern-matching-related methods to [PreviewLink].
extension PreviewLinkPatterns on PreviewLink {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PreviewLink_Note value)?  note,TResult Function( PreviewLink_NewNote value)?  newNote,TResult Function( PreviewLink_Tag value)?  tag,TResult Function( PreviewLink_Heading value)?  heading,TResult Function( PreviewLink_Web value)?  web,TResult Function( PreviewLink_Attachment value)?  attachment,TResult Function( PreviewLink_Task value)?  task,TResult Function( PreviewLink_CopyCode value)?  copyCode,TResult Function( PreviewLink_Due value)?  due,TResult Function( PreviewLink_Nothing value)?  nothing,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PreviewLink_Note() when note != null:
return note(_that);case PreviewLink_NewNote() when newNote != null:
return newNote(_that);case PreviewLink_Tag() when tag != null:
return tag(_that);case PreviewLink_Heading() when heading != null:
return heading(_that);case PreviewLink_Web() when web != null:
return web(_that);case PreviewLink_Attachment() when attachment != null:
return attachment(_that);case PreviewLink_Task() when task != null:
return task(_that);case PreviewLink_CopyCode() when copyCode != null:
return copyCode(_that);case PreviewLink_Due() when due != null:
return due(_that);case PreviewLink_Nothing() when nothing != null:
return nothing(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PreviewLink_Note value)  note,required TResult Function( PreviewLink_NewNote value)  newNote,required TResult Function( PreviewLink_Tag value)  tag,required TResult Function( PreviewLink_Heading value)  heading,required TResult Function( PreviewLink_Web value)  web,required TResult Function( PreviewLink_Attachment value)  attachment,required TResult Function( PreviewLink_Task value)  task,required TResult Function( PreviewLink_CopyCode value)  copyCode,required TResult Function( PreviewLink_Due value)  due,required TResult Function( PreviewLink_Nothing value)  nothing,}){
final _that = this;
switch (_that) {
case PreviewLink_Note():
return note(_that);case PreviewLink_NewNote():
return newNote(_that);case PreviewLink_Tag():
return tag(_that);case PreviewLink_Heading():
return heading(_that);case PreviewLink_Web():
return web(_that);case PreviewLink_Attachment():
return attachment(_that);case PreviewLink_Task():
return task(_that);case PreviewLink_CopyCode():
return copyCode(_that);case PreviewLink_Due():
return due(_that);case PreviewLink_Nothing():
return nothing(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PreviewLink_Note value)?  note,TResult? Function( PreviewLink_NewNote value)?  newNote,TResult? Function( PreviewLink_Tag value)?  tag,TResult? Function( PreviewLink_Heading value)?  heading,TResult? Function( PreviewLink_Web value)?  web,TResult? Function( PreviewLink_Attachment value)?  attachment,TResult? Function( PreviewLink_Task value)?  task,TResult? Function( PreviewLink_CopyCode value)?  copyCode,TResult? Function( PreviewLink_Due value)?  due,TResult? Function( PreviewLink_Nothing value)?  nothing,}){
final _that = this;
switch (_that) {
case PreviewLink_Note() when note != null:
return note(_that);case PreviewLink_NewNote() when newNote != null:
return newNote(_that);case PreviewLink_Tag() when tag != null:
return tag(_that);case PreviewLink_Heading() when heading != null:
return heading(_that);case PreviewLink_Web() when web != null:
return web(_that);case PreviewLink_Attachment() when attachment != null:
return attachment(_that);case PreviewLink_Task() when task != null:
return task(_that);case PreviewLink_CopyCode() when copyCode != null:
return copyCode(_that);case PreviewLink_Due() when due != null:
return due(_that);case PreviewLink_Nothing() when nothing != null:
return nothing(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id,  String? heading,  String? anchor)?  note,TResult Function( String title)?  newNote,TResult Function( String name)?  tag,TResult Function( String id)?  heading,TResult Function( String url)?  web,TResult Function( String id,  String name,  String contentType,  int size)?  attachment,TResult Function( int index)?  task,TResult Function( int index)?  copyCode,TResult Function()?  due,TResult Function()?  nothing,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PreviewLink_Note() when note != null:
return note(_that.id,_that.heading,_that.anchor);case PreviewLink_NewNote() when newNote != null:
return newNote(_that.title);case PreviewLink_Tag() when tag != null:
return tag(_that.name);case PreviewLink_Heading() when heading != null:
return heading(_that.id);case PreviewLink_Web() when web != null:
return web(_that.url);case PreviewLink_Attachment() when attachment != null:
return attachment(_that.id,_that.name,_that.contentType,_that.size);case PreviewLink_Task() when task != null:
return task(_that.index);case PreviewLink_CopyCode() when copyCode != null:
return copyCode(_that.index);case PreviewLink_Due() when due != null:
return due();case PreviewLink_Nothing() when nothing != null:
return nothing();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id,  String? heading,  String? anchor)  note,required TResult Function( String title)  newNote,required TResult Function( String name)  tag,required TResult Function( String id)  heading,required TResult Function( String url)  web,required TResult Function( String id,  String name,  String contentType,  int size)  attachment,required TResult Function( int index)  task,required TResult Function( int index)  copyCode,required TResult Function()  due,required TResult Function()  nothing,}) {final _that = this;
switch (_that) {
case PreviewLink_Note():
return note(_that.id,_that.heading,_that.anchor);case PreviewLink_NewNote():
return newNote(_that.title);case PreviewLink_Tag():
return tag(_that.name);case PreviewLink_Heading():
return heading(_that.id);case PreviewLink_Web():
return web(_that.url);case PreviewLink_Attachment():
return attachment(_that.id,_that.name,_that.contentType,_that.size);case PreviewLink_Task():
return task(_that.index);case PreviewLink_CopyCode():
return copyCode(_that.index);case PreviewLink_Due():
return due();case PreviewLink_Nothing():
return nothing();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id,  String? heading,  String? anchor)?  note,TResult? Function( String title)?  newNote,TResult? Function( String name)?  tag,TResult? Function( String id)?  heading,TResult? Function( String url)?  web,TResult? Function( String id,  String name,  String contentType,  int size)?  attachment,TResult? Function( int index)?  task,TResult? Function( int index)?  copyCode,TResult? Function()?  due,TResult? Function()?  nothing,}) {final _that = this;
switch (_that) {
case PreviewLink_Note() when note != null:
return note(_that.id,_that.heading,_that.anchor);case PreviewLink_NewNote() when newNote != null:
return newNote(_that.title);case PreviewLink_Tag() when tag != null:
return tag(_that.name);case PreviewLink_Heading() when heading != null:
return heading(_that.id);case PreviewLink_Web() when web != null:
return web(_that.url);case PreviewLink_Attachment() when attachment != null:
return attachment(_that.id,_that.name,_that.contentType,_that.size);case PreviewLink_Task() when task != null:
return task(_that.index);case PreviewLink_CopyCode() when copyCode != null:
return copyCode(_that.index);case PreviewLink_Due() when due != null:
return due();case PreviewLink_Nothing() when nothing != null:
return nothing();case _:
  return null;

}
}

}

/// @nodoc


class PreviewLink_Note extends PreviewLink {
  const PreviewLink_Note({required this.id, this.heading, this.anchor}): super._();
  

 final  String id;
 final  String? heading;
 final  String? anchor;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_NoteCopyWith<PreviewLink_Note> get copyWith => _$PreviewLink_NoteCopyWithImpl<PreviewLink_Note>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Note&&(identical(other.id, id) || other.id == id)&&(identical(other.heading, heading) || other.heading == heading)&&(identical(other.anchor, anchor) || other.anchor == anchor));
}


@override
int get hashCode => Object.hash(runtimeType,id,heading,anchor);

@override
String toString() {
  return 'PreviewLink.note(id: $id, heading: $heading, anchor: $anchor)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_NoteCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_NoteCopyWith(PreviewLink_Note value, $Res Function(PreviewLink_Note) _then) = _$PreviewLink_NoteCopyWithImpl;
@useResult
$Res call({
 String id, String? heading, String? anchor
});




}
/// @nodoc
class _$PreviewLink_NoteCopyWithImpl<$Res>
    implements $PreviewLink_NoteCopyWith<$Res> {
  _$PreviewLink_NoteCopyWithImpl(this._self, this._then);

  final PreviewLink_Note _self;
  final $Res Function(PreviewLink_Note) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? heading = freezed,Object? anchor = freezed,}) {
  return _then(PreviewLink_Note(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,heading: freezed == heading ? _self.heading : heading // ignore: cast_nullable_to_non_nullable
as String?,anchor: freezed == anchor ? _self.anchor : anchor // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class PreviewLink_NewNote extends PreviewLink {
  const PreviewLink_NewNote({required this.title}): super._();
  

 final  String title;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_NewNoteCopyWith<PreviewLink_NewNote> get copyWith => _$PreviewLink_NewNoteCopyWithImpl<PreviewLink_NewNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_NewNote&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'PreviewLink.newNote(title: $title)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_NewNoteCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_NewNoteCopyWith(PreviewLink_NewNote value, $Res Function(PreviewLink_NewNote) _then) = _$PreviewLink_NewNoteCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$PreviewLink_NewNoteCopyWithImpl<$Res>
    implements $PreviewLink_NewNoteCopyWith<$Res> {
  _$PreviewLink_NewNoteCopyWithImpl(this._self, this._then);

  final PreviewLink_NewNote _self;
  final $Res Function(PreviewLink_NewNote) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(PreviewLink_NewNote(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewLink_Tag extends PreviewLink {
  const PreviewLink_Tag({required this.name}): super._();
  

 final  String name;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_TagCopyWith<PreviewLink_Tag> get copyWith => _$PreviewLink_TagCopyWithImpl<PreviewLink_Tag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Tag&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'PreviewLink.tag(name: $name)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_TagCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_TagCopyWith(PreviewLink_Tag value, $Res Function(PreviewLink_Tag) _then) = _$PreviewLink_TagCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$PreviewLink_TagCopyWithImpl<$Res>
    implements $PreviewLink_TagCopyWith<$Res> {
  _$PreviewLink_TagCopyWithImpl(this._self, this._then);

  final PreviewLink_Tag _self;
  final $Res Function(PreviewLink_Tag) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(PreviewLink_Tag(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewLink_Heading extends PreviewLink {
  const PreviewLink_Heading({required this.id}): super._();
  

 final  String id;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_HeadingCopyWith<PreviewLink_Heading> get copyWith => _$PreviewLink_HeadingCopyWithImpl<PreviewLink_Heading>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Heading&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'PreviewLink.heading(id: $id)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_HeadingCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_HeadingCopyWith(PreviewLink_Heading value, $Res Function(PreviewLink_Heading) _then) = _$PreviewLink_HeadingCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$PreviewLink_HeadingCopyWithImpl<$Res>
    implements $PreviewLink_HeadingCopyWith<$Res> {
  _$PreviewLink_HeadingCopyWithImpl(this._self, this._then);

  final PreviewLink_Heading _self;
  final $Res Function(PreviewLink_Heading) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(PreviewLink_Heading(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewLink_Web extends PreviewLink {
  const PreviewLink_Web({required this.url}): super._();
  

 final  String url;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_WebCopyWith<PreviewLink_Web> get copyWith => _$PreviewLink_WebCopyWithImpl<PreviewLink_Web>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Web&&(identical(other.url, url) || other.url == url));
}


@override
int get hashCode => Object.hash(runtimeType,url);

@override
String toString() {
  return 'PreviewLink.web(url: $url)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_WebCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_WebCopyWith(PreviewLink_Web value, $Res Function(PreviewLink_Web) _then) = _$PreviewLink_WebCopyWithImpl;
@useResult
$Res call({
 String url
});




}
/// @nodoc
class _$PreviewLink_WebCopyWithImpl<$Res>
    implements $PreviewLink_WebCopyWith<$Res> {
  _$PreviewLink_WebCopyWithImpl(this._self, this._then);

  final PreviewLink_Web _self;
  final $Res Function(PreviewLink_Web) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,}) {
  return _then(PreviewLink_Web(
url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewLink_Attachment extends PreviewLink {
  const PreviewLink_Attachment({required this.id, required this.name, required this.contentType, required this.size}): super._();
  

 final  String id;
 final  String name;
 final  String contentType;
 final  int size;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_AttachmentCopyWith<PreviewLink_Attachment> get copyWith => _$PreviewLink_AttachmentCopyWithImpl<PreviewLink_Attachment>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Attachment&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.contentType, contentType) || other.contentType == contentType)&&(identical(other.size, size) || other.size == size));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,contentType,size);

@override
String toString() {
  return 'PreviewLink.attachment(id: $id, name: $name, contentType: $contentType, size: $size)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_AttachmentCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_AttachmentCopyWith(PreviewLink_Attachment value, $Res Function(PreviewLink_Attachment) _then) = _$PreviewLink_AttachmentCopyWithImpl;
@useResult
$Res call({
 String id, String name, String contentType, int size
});




}
/// @nodoc
class _$PreviewLink_AttachmentCopyWithImpl<$Res>
    implements $PreviewLink_AttachmentCopyWith<$Res> {
  _$PreviewLink_AttachmentCopyWithImpl(this._self, this._then);

  final PreviewLink_Attachment _self;
  final $Res Function(PreviewLink_Attachment) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? contentType = null,Object? size = null,}) {
  return _then(PreviewLink_Attachment(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,contentType: null == contentType ? _self.contentType : contentType // ignore: cast_nullable_to_non_nullable
as String,size: null == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PreviewLink_Task extends PreviewLink {
  const PreviewLink_Task({required this.index}): super._();
  

 final  int index;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_TaskCopyWith<PreviewLink_Task> get copyWith => _$PreviewLink_TaskCopyWithImpl<PreviewLink_Task>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Task&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'PreviewLink.task(index: $index)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_TaskCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_TaskCopyWith(PreviewLink_Task value, $Res Function(PreviewLink_Task) _then) = _$PreviewLink_TaskCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$PreviewLink_TaskCopyWithImpl<$Res>
    implements $PreviewLink_TaskCopyWith<$Res> {
  _$PreviewLink_TaskCopyWithImpl(this._self, this._then);

  final PreviewLink_Task _self;
  final $Res Function(PreviewLink_Task) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(PreviewLink_Task(
index: null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PreviewLink_CopyCode extends PreviewLink {
  const PreviewLink_CopyCode({required this.index}): super._();
  

 final  int index;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLink_CopyCodeCopyWith<PreviewLink_CopyCode> get copyWith => _$PreviewLink_CopyCodeCopyWithImpl<PreviewLink_CopyCode>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_CopyCode&&(identical(other.index, index) || other.index == index));
}


@override
int get hashCode => Object.hash(runtimeType,index);

@override
String toString() {
  return 'PreviewLink.copyCode(index: $index)';
}


}

/// @nodoc
abstract mixin class $PreviewLink_CopyCodeCopyWith<$Res> implements $PreviewLinkCopyWith<$Res> {
  factory $PreviewLink_CopyCodeCopyWith(PreviewLink_CopyCode value, $Res Function(PreviewLink_CopyCode) _then) = _$PreviewLink_CopyCodeCopyWithImpl;
@useResult
$Res call({
 int index
});




}
/// @nodoc
class _$PreviewLink_CopyCodeCopyWithImpl<$Res>
    implements $PreviewLink_CopyCodeCopyWith<$Res> {
  _$PreviewLink_CopyCodeCopyWithImpl(this._self, this._then);

  final PreviewLink_CopyCode _self;
  final $Res Function(PreviewLink_CopyCode) _then;

/// Create a copy of PreviewLink
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,}) {
  return _then(PreviewLink_CopyCode(
index: null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PreviewLink_Due extends PreviewLink {
  const PreviewLink_Due(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Due);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewLink.due()';
}


}




/// @nodoc


class PreviewLink_Nothing extends PreviewLink {
  const PreviewLink_Nothing(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLink_Nothing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewLink.nothing()';
}


}




/// @nodoc
mixin _$ProviderEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ProviderEvent()';
}


}

/// @nodoc
class $ProviderEventCopyWith<$Res>  {
$ProviderEventCopyWith(ProviderEvent _, $Res Function(ProviderEvent) __);
}


/// Adds pattern-matching-related methods to [ProviderEvent].
extension ProviderEventPatterns on ProviderEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ProviderEvent_OpenPage value)?  openPage,TResult Function( ProviderEvent_ShowCode value)?  showCode,TResult Function( ProviderEvent_ClosePage value)?  closePage,TResult Function( ProviderEvent_NeedsUsername value)?  needsUsername,TResult Function( ProviderEvent_Finished value)?  finished,TResult Function( ProviderEvent_Failed value)?  failed,TResult Function( ProviderEvent_Cancelled value)?  cancelled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that);case ProviderEvent_ClosePage() when closePage != null:
return closePage(_that);case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that);case ProviderEvent_Finished() when finished != null:
return finished(_that);case ProviderEvent_Failed() when failed != null:
return failed(_that);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ProviderEvent_OpenPage value)  openPage,required TResult Function( ProviderEvent_ShowCode value)  showCode,required TResult Function( ProviderEvent_ClosePage value)  closePage,required TResult Function( ProviderEvent_NeedsUsername value)  needsUsername,required TResult Function( ProviderEvent_Finished value)  finished,required TResult Function( ProviderEvent_Failed value)  failed,required TResult Function( ProviderEvent_Cancelled value)  cancelled,}){
final _that = this;
switch (_that) {
case ProviderEvent_OpenPage():
return openPage(_that);case ProviderEvent_ShowCode():
return showCode(_that);case ProviderEvent_ClosePage():
return closePage(_that);case ProviderEvent_NeedsUsername():
return needsUsername(_that);case ProviderEvent_Finished():
return finished(_that);case ProviderEvent_Failed():
return failed(_that);case ProviderEvent_Cancelled():
return cancelled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ProviderEvent_OpenPage value)?  openPage,TResult? Function( ProviderEvent_ShowCode value)?  showCode,TResult? Function( ProviderEvent_ClosePage value)?  closePage,TResult? Function( ProviderEvent_NeedsUsername value)?  needsUsername,TResult? Function( ProviderEvent_Finished value)?  finished,TResult? Function( ProviderEvent_Failed value)?  failed,TResult? Function( ProviderEvent_Cancelled value)?  cancelled,}){
final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that);case ProviderEvent_ClosePage() when closePage != null:
return closePage(_that);case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that);case ProviderEvent_Finished() when finished != null:
return finished(_that);case ProviderEvent_Failed() when failed != null:
return failed(_that);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String url,  String? callbackScheme)?  openPage,TResult Function( String userCode,  String verificationUri)?  showCode,TResult Function()?  closePage,TResult Function( String suggested)?  needsUsername,TResult Function( AccountState field0)?  finished,TResult Function( String message)?  failed,TResult Function()?  cancelled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that.url,_that.callbackScheme);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that.userCode,_that.verificationUri);case ProviderEvent_ClosePage() when closePage != null:
return closePage();case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that.suggested);case ProviderEvent_Finished() when finished != null:
return finished(_that.field0);case ProviderEvent_Failed() when failed != null:
return failed(_that.message);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String url,  String? callbackScheme)  openPage,required TResult Function( String userCode,  String verificationUri)  showCode,required TResult Function()  closePage,required TResult Function( String suggested)  needsUsername,required TResult Function( AccountState field0)  finished,required TResult Function( String message)  failed,required TResult Function()  cancelled,}) {final _that = this;
switch (_that) {
case ProviderEvent_OpenPage():
return openPage(_that.url,_that.callbackScheme);case ProviderEvent_ShowCode():
return showCode(_that.userCode,_that.verificationUri);case ProviderEvent_ClosePage():
return closePage();case ProviderEvent_NeedsUsername():
return needsUsername(_that.suggested);case ProviderEvent_Finished():
return finished(_that.field0);case ProviderEvent_Failed():
return failed(_that.message);case ProviderEvent_Cancelled():
return cancelled();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String url,  String? callbackScheme)?  openPage,TResult? Function( String userCode,  String verificationUri)?  showCode,TResult? Function()?  closePage,TResult? Function( String suggested)?  needsUsername,TResult? Function( AccountState field0)?  finished,TResult? Function( String message)?  failed,TResult? Function()?  cancelled,}) {final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that.url,_that.callbackScheme);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that.userCode,_that.verificationUri);case ProviderEvent_ClosePage() when closePage != null:
return closePage();case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that.suggested);case ProviderEvent_Finished() when finished != null:
return finished(_that.field0);case ProviderEvent_Failed() when failed != null:
return failed(_that.message);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled();case _:
  return null;

}
}

}

/// @nodoc


class ProviderEvent_OpenPage extends ProviderEvent {
  const ProviderEvent_OpenPage({required this.url, this.callbackScheme}): super._();
  

 final  String url;
 final  String? callbackScheme;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_OpenPageCopyWith<ProviderEvent_OpenPage> get copyWith => _$ProviderEvent_OpenPageCopyWithImpl<ProviderEvent_OpenPage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_OpenPage&&(identical(other.url, url) || other.url == url)&&(identical(other.callbackScheme, callbackScheme) || other.callbackScheme == callbackScheme));
}


@override
int get hashCode => Object.hash(runtimeType,url,callbackScheme);

@override
String toString() {
  return 'ProviderEvent.openPage(url: $url, callbackScheme: $callbackScheme)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_OpenPageCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_OpenPageCopyWith(ProviderEvent_OpenPage value, $Res Function(ProviderEvent_OpenPage) _then) = _$ProviderEvent_OpenPageCopyWithImpl;
@useResult
$Res call({
 String url, String? callbackScheme
});




}
/// @nodoc
class _$ProviderEvent_OpenPageCopyWithImpl<$Res>
    implements $ProviderEvent_OpenPageCopyWith<$Res> {
  _$ProviderEvent_OpenPageCopyWithImpl(this._self, this._then);

  final ProviderEvent_OpenPage _self;
  final $Res Function(ProviderEvent_OpenPage) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,Object? callbackScheme = freezed,}) {
  return _then(ProviderEvent_OpenPage(
url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,callbackScheme: freezed == callbackScheme ? _self.callbackScheme : callbackScheme // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class ProviderEvent_ShowCode extends ProviderEvent {
  const ProviderEvent_ShowCode({required this.userCode, required this.verificationUri}): super._();
  

 final  String userCode;
 final  String verificationUri;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_ShowCodeCopyWith<ProviderEvent_ShowCode> get copyWith => _$ProviderEvent_ShowCodeCopyWithImpl<ProviderEvent_ShowCode>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_ShowCode&&(identical(other.userCode, userCode) || other.userCode == userCode)&&(identical(other.verificationUri, verificationUri) || other.verificationUri == verificationUri));
}


@override
int get hashCode => Object.hash(runtimeType,userCode,verificationUri);

@override
String toString() {
  return 'ProviderEvent.showCode(userCode: $userCode, verificationUri: $verificationUri)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_ShowCodeCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_ShowCodeCopyWith(ProviderEvent_ShowCode value, $Res Function(ProviderEvent_ShowCode) _then) = _$ProviderEvent_ShowCodeCopyWithImpl;
@useResult
$Res call({
 String userCode, String verificationUri
});




}
/// @nodoc
class _$ProviderEvent_ShowCodeCopyWithImpl<$Res>
    implements $ProviderEvent_ShowCodeCopyWith<$Res> {
  _$ProviderEvent_ShowCodeCopyWithImpl(this._self, this._then);

  final ProviderEvent_ShowCode _self;
  final $Res Function(ProviderEvent_ShowCode) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? userCode = null,Object? verificationUri = null,}) {
  return _then(ProviderEvent_ShowCode(
userCode: null == userCode ? _self.userCode : userCode // ignore: cast_nullable_to_non_nullable
as String,verificationUri: null == verificationUri ? _self.verificationUri : verificationUri // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ProviderEvent_ClosePage extends ProviderEvent {
  const ProviderEvent_ClosePage(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_ClosePage);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ProviderEvent.closePage()';
}


}




/// @nodoc


class ProviderEvent_NeedsUsername extends ProviderEvent {
  const ProviderEvent_NeedsUsername({required this.suggested}): super._();
  

 final  String suggested;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_NeedsUsernameCopyWith<ProviderEvent_NeedsUsername> get copyWith => _$ProviderEvent_NeedsUsernameCopyWithImpl<ProviderEvent_NeedsUsername>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_NeedsUsername&&(identical(other.suggested, suggested) || other.suggested == suggested));
}


@override
int get hashCode => Object.hash(runtimeType,suggested);

@override
String toString() {
  return 'ProviderEvent.needsUsername(suggested: $suggested)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_NeedsUsernameCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_NeedsUsernameCopyWith(ProviderEvent_NeedsUsername value, $Res Function(ProviderEvent_NeedsUsername) _then) = _$ProviderEvent_NeedsUsernameCopyWithImpl;
@useResult
$Res call({
 String suggested
});




}
/// @nodoc
class _$ProviderEvent_NeedsUsernameCopyWithImpl<$Res>
    implements $ProviderEvent_NeedsUsernameCopyWith<$Res> {
  _$ProviderEvent_NeedsUsernameCopyWithImpl(this._self, this._then);

  final ProviderEvent_NeedsUsername _self;
  final $Res Function(ProviderEvent_NeedsUsername) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? suggested = null,}) {
  return _then(ProviderEvent_NeedsUsername(
suggested: null == suggested ? _self.suggested : suggested // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ProviderEvent_Finished extends ProviderEvent {
  const ProviderEvent_Finished(this.field0): super._();
  

 final  AccountState field0;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_FinishedCopyWith<ProviderEvent_Finished> get copyWith => _$ProviderEvent_FinishedCopyWithImpl<ProviderEvent_Finished>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_Finished&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'ProviderEvent.finished(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_FinishedCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_FinishedCopyWith(ProviderEvent_Finished value, $Res Function(ProviderEvent_Finished) _then) = _$ProviderEvent_FinishedCopyWithImpl;
@useResult
$Res call({
 AccountState field0
});


$AccountStateCopyWith<$Res> get field0;

}
/// @nodoc
class _$ProviderEvent_FinishedCopyWithImpl<$Res>
    implements $ProviderEvent_FinishedCopyWith<$Res> {
  _$ProviderEvent_FinishedCopyWithImpl(this._self, this._then);

  final ProviderEvent_Finished _self;
  final $Res Function(ProviderEvent_Finished) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ProviderEvent_Finished(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as AccountState,
  ));
}

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AccountStateCopyWith<$Res> get field0 {
  
  return $AccountStateCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class ProviderEvent_Failed extends ProviderEvent {
  const ProviderEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_FailedCopyWith<ProviderEvent_Failed> get copyWith => _$ProviderEvent_FailedCopyWithImpl<ProviderEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ProviderEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_FailedCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_FailedCopyWith(ProviderEvent_Failed value, $Res Function(ProviderEvent_Failed) _then) = _$ProviderEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ProviderEvent_FailedCopyWithImpl<$Res>
    implements $ProviderEvent_FailedCopyWith<$Res> {
  _$ProviderEvent_FailedCopyWithImpl(this._self, this._then);

  final ProviderEvent_Failed _self;
  final $Res Function(ProviderEvent_Failed) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ProviderEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ProviderEvent_Cancelled extends ProviderEvent {
  const ProviderEvent_Cancelled(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_Cancelled);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ProviderEvent.cancelled()';
}


}




/// @nodoc
mixin _$SaveState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SaveState()';
}


}

/// @nodoc
class $SaveStateCopyWith<$Res>  {
$SaveStateCopyWith(SaveState _, $Res Function(SaveState) __);
}


/// Adds pattern-matching-related methods to [SaveState].
extension SaveStatePatterns on SaveState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SaveState_Saved value)?  saved,TResult Function( SaveState_Pending value)?  pending,TResult Function( SaveState_Conflict value)?  conflict,TResult Function( SaveState_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that);case SaveState_Pending() when pending != null:
return pending(_that);case SaveState_Conflict() when conflict != null:
return conflict(_that);case SaveState_Failed() when failed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SaveState_Saved value)  saved,required TResult Function( SaveState_Pending value)  pending,required TResult Function( SaveState_Conflict value)  conflict,required TResult Function( SaveState_Failed value)  failed,}){
final _that = this;
switch (_that) {
case SaveState_Saved():
return saved(_that);case SaveState_Pending():
return pending(_that);case SaveState_Conflict():
return conflict(_that);case SaveState_Failed():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SaveState_Saved value)?  saved,TResult? Function( SaveState_Pending value)?  pending,TResult? Function( SaveState_Conflict value)?  conflict,TResult? Function( SaveState_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that);case SaveState_Pending() when pending != null:
return pending(_that);case SaveState_Conflict() when conflict != null:
return conflict(_that);case SaveState_Failed() when failed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String rev,  int atMs)?  saved,TResult Function()?  pending,TResult Function( bool deleted)?  conflict,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case SaveState_Pending() when pending != null:
return pending();case SaveState_Conflict() when conflict != null:
return conflict(_that.deleted);case SaveState_Failed() when failed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String rev,  int atMs)  saved,required TResult Function()  pending,required TResult Function( bool deleted)  conflict,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case SaveState_Saved():
return saved(_that.rev,_that.atMs);case SaveState_Pending():
return pending();case SaveState_Conflict():
return conflict(_that.deleted);case SaveState_Failed():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String rev,  int atMs)?  saved,TResult? Function()?  pending,TResult? Function( bool deleted)?  conflict,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case SaveState_Pending() when pending != null:
return pending();case SaveState_Conflict() when conflict != null:
return conflict(_that.deleted);case SaveState_Failed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class SaveState_Saved extends SaveState {
  const SaveState_Saved({required this.rev, required this.atMs}): super._();
  

 final  String rev;
 final  int atMs;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SaveState_SavedCopyWith<SaveState_Saved> get copyWith => _$SaveState_SavedCopyWithImpl<SaveState_Saved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Saved&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.atMs, atMs) || other.atMs == atMs));
}


@override
int get hashCode => Object.hash(runtimeType,rev,atMs);

@override
String toString() {
  return 'SaveState.saved(rev: $rev, atMs: $atMs)';
}


}

/// @nodoc
abstract mixin class $SaveState_SavedCopyWith<$Res> implements $SaveStateCopyWith<$Res> {
  factory $SaveState_SavedCopyWith(SaveState_Saved value, $Res Function(SaveState_Saved) _then) = _$SaveState_SavedCopyWithImpl;
@useResult
$Res call({
 String rev, int atMs
});




}
/// @nodoc
class _$SaveState_SavedCopyWithImpl<$Res>
    implements $SaveState_SavedCopyWith<$Res> {
  _$SaveState_SavedCopyWithImpl(this._self, this._then);

  final SaveState_Saved _self;
  final $Res Function(SaveState_Saved) _then;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,Object? atMs = null,}) {
  return _then(SaveState_Saved(
rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,atMs: null == atMs ? _self.atMs : atMs // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class SaveState_Pending extends SaveState {
  const SaveState_Pending(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Pending);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SaveState.pending()';
}


}




/// @nodoc


class SaveState_Conflict extends SaveState {
  const SaveState_Conflict({required this.deleted}): super._();
  

 final  bool deleted;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SaveState_ConflictCopyWith<SaveState_Conflict> get copyWith => _$SaveState_ConflictCopyWithImpl<SaveState_Conflict>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Conflict&&(identical(other.deleted, deleted) || other.deleted == deleted));
}


@override
int get hashCode => Object.hash(runtimeType,deleted);

@override
String toString() {
  return 'SaveState.conflict(deleted: $deleted)';
}


}

/// @nodoc
abstract mixin class $SaveState_ConflictCopyWith<$Res> implements $SaveStateCopyWith<$Res> {
  factory $SaveState_ConflictCopyWith(SaveState_Conflict value, $Res Function(SaveState_Conflict) _then) = _$SaveState_ConflictCopyWithImpl;
@useResult
$Res call({
 bool deleted
});




}
/// @nodoc
class _$SaveState_ConflictCopyWithImpl<$Res>
    implements $SaveState_ConflictCopyWith<$Res> {
  _$SaveState_ConflictCopyWithImpl(this._self, this._then);

  final SaveState_Conflict _self;
  final $Res Function(SaveState_Conflict) _then;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deleted = null,}) {
  return _then(SaveState_Conflict(
deleted: null == deleted ? _self.deleted : deleted // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class SaveState_Failed extends SaveState {
  const SaveState_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SaveState_FailedCopyWith<SaveState_Failed> get copyWith => _$SaveState_FailedCopyWithImpl<SaveState_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'SaveState.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $SaveState_FailedCopyWith<$Res> implements $SaveStateCopyWith<$Res> {
  factory $SaveState_FailedCopyWith(SaveState_Failed value, $Res Function(SaveState_Failed) _then) = _$SaveState_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$SaveState_FailedCopyWithImpl<$Res>
    implements $SaveState_FailedCopyWith<$Res> {
  _$SaveState_FailedCopyWithImpl(this._self, this._then);

  final SaveState_Failed _self;
  final $Res Function(SaveState_Failed) _then;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(SaveState_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$SearchScope {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope()';
}


}

/// @nodoc
class $SearchScopeCopyWith<$Res>  {
$SearchScopeCopyWith(SearchScope _, $Res Function(SearchScope) __);
}


/// Adds pattern-matching-related methods to [SearchScope].
extension SearchScopePatterns on SearchScope {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SearchScope_All value)?  all,TResult Function( SearchScope_Titles value)?  titles,TResult Function( SearchScope_Tags value)?  tags,TResult Function( SearchScope_Notebook value)?  notebook,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all(_that);case SearchScope_Titles() when titles != null:
return titles(_that);case SearchScope_Tags() when tags != null:
return tags(_that);case SearchScope_Notebook() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SearchScope_All value)  all,required TResult Function( SearchScope_Titles value)  titles,required TResult Function( SearchScope_Tags value)  tags,required TResult Function( SearchScope_Notebook value)  notebook,}){
final _that = this;
switch (_that) {
case SearchScope_All():
return all(_that);case SearchScope_Titles():
return titles(_that);case SearchScope_Tags():
return tags(_that);case SearchScope_Notebook():
return notebook(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SearchScope_All value)?  all,TResult? Function( SearchScope_Titles value)?  titles,TResult? Function( SearchScope_Tags value)?  tags,TResult? Function( SearchScope_Notebook value)?  notebook,}){
final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all(_that);case SearchScope_Titles() when titles != null:
return titles(_that);case SearchScope_Tags() when tags != null:
return tags(_that);case SearchScope_Notebook() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function()?  titles,TResult Function()?  tags,TResult Function( String id)?  notebook,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all();case SearchScope_Titles() when titles != null:
return titles();case SearchScope_Tags() when tags != null:
return tags();case SearchScope_Notebook() when notebook != null:
return notebook(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function()  titles,required TResult Function()  tags,required TResult Function( String id)  notebook,}) {final _that = this;
switch (_that) {
case SearchScope_All():
return all();case SearchScope_Titles():
return titles();case SearchScope_Tags():
return tags();case SearchScope_Notebook():
return notebook(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function()?  titles,TResult? Function()?  tags,TResult? Function( String id)?  notebook,}) {final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all();case SearchScope_Titles() when titles != null:
return titles();case SearchScope_Tags() when tags != null:
return tags();case SearchScope_Notebook() when notebook != null:
return notebook(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class SearchScope_All extends SearchScope {
  const SearchScope_All(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.all()';
}


}




/// @nodoc


class SearchScope_Titles extends SearchScope {
  const SearchScope_Titles(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Titles);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.titles()';
}


}




/// @nodoc


class SearchScope_Tags extends SearchScope {
  const SearchScope_Tags(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Tags);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.tags()';
}


}




/// @nodoc


class SearchScope_Notebook extends SearchScope {
  const SearchScope_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of SearchScope
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchScope_NotebookCopyWith<SearchScope_Notebook> get copyWith => _$SearchScope_NotebookCopyWithImpl<SearchScope_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'SearchScope.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $SearchScope_NotebookCopyWith<$Res> implements $SearchScopeCopyWith<$Res> {
  factory $SearchScope_NotebookCopyWith(SearchScope_Notebook value, $Res Function(SearchScope_Notebook) _then) = _$SearchScope_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$SearchScope_NotebookCopyWithImpl<$Res>
    implements $SearchScope_NotebookCopyWith<$Res> {
  _$SearchScope_NotebookCopyWithImpl(this._self, this._then);

  final SearchScope_Notebook _self;
  final $Res Function(SearchScope_Notebook) _then;

/// Create a copy of SearchScope
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(SearchScope_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$SyncAccount {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncAccount);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncAccount()';
}


}

/// @nodoc
class $SyncAccountCopyWith<$Res>  {
$SyncAccountCopyWith(SyncAccount _, $Res Function(SyncAccount) __);
}


/// Adds pattern-matching-related methods to [SyncAccount].
extension SyncAccountPatterns on SyncAccount {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SyncAccount_SignedOut value)?  signedOut,TResult Function( SyncAccount_SignedIn value)?  signedIn,TResult Function( SyncAccount_Locked value)?  locked,TResult Function( SyncAccount_SessionEnded value)?  sessionEnded,TResult Function( SyncAccount_Busy value)?  busy,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SyncAccount_SignedOut() when signedOut != null:
return signedOut(_that);case SyncAccount_SignedIn() when signedIn != null:
return signedIn(_that);case SyncAccount_Locked() when locked != null:
return locked(_that);case SyncAccount_SessionEnded() when sessionEnded != null:
return sessionEnded(_that);case SyncAccount_Busy() when busy != null:
return busy(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SyncAccount_SignedOut value)  signedOut,required TResult Function( SyncAccount_SignedIn value)  signedIn,required TResult Function( SyncAccount_Locked value)  locked,required TResult Function( SyncAccount_SessionEnded value)  sessionEnded,required TResult Function( SyncAccount_Busy value)  busy,}){
final _that = this;
switch (_that) {
case SyncAccount_SignedOut():
return signedOut(_that);case SyncAccount_SignedIn():
return signedIn(_that);case SyncAccount_Locked():
return locked(_that);case SyncAccount_SessionEnded():
return sessionEnded(_that);case SyncAccount_Busy():
return busy(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SyncAccount_SignedOut value)?  signedOut,TResult? Function( SyncAccount_SignedIn value)?  signedIn,TResult? Function( SyncAccount_Locked value)?  locked,TResult? Function( SyncAccount_SessionEnded value)?  sessionEnded,TResult? Function( SyncAccount_Busy value)?  busy,}){
final _that = this;
switch (_that) {
case SyncAccount_SignedOut() when signedOut != null:
return signedOut(_that);case SyncAccount_SignedIn() when signedIn != null:
return signedIn(_that);case SyncAccount_Locked() when locked != null:
return locked(_that);case SyncAccount_SessionEnded() when sessionEnded != null:
return sessionEnded(_that);case SyncAccount_Busy() when busy != null:
return busy(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  signedOut,TResult Function()?  signedIn,TResult Function()?  locked,TResult Function()?  sessionEnded,TResult Function( AccountStep field0)?  busy,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SyncAccount_SignedOut() when signedOut != null:
return signedOut();case SyncAccount_SignedIn() when signedIn != null:
return signedIn();case SyncAccount_Locked() when locked != null:
return locked();case SyncAccount_SessionEnded() when sessionEnded != null:
return sessionEnded();case SyncAccount_Busy() when busy != null:
return busy(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  signedOut,required TResult Function()  signedIn,required TResult Function()  locked,required TResult Function()  sessionEnded,required TResult Function( AccountStep field0)  busy,}) {final _that = this;
switch (_that) {
case SyncAccount_SignedOut():
return signedOut();case SyncAccount_SignedIn():
return signedIn();case SyncAccount_Locked():
return locked();case SyncAccount_SessionEnded():
return sessionEnded();case SyncAccount_Busy():
return busy(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  signedOut,TResult? Function()?  signedIn,TResult? Function()?  locked,TResult? Function()?  sessionEnded,TResult? Function( AccountStep field0)?  busy,}) {final _that = this;
switch (_that) {
case SyncAccount_SignedOut() when signedOut != null:
return signedOut();case SyncAccount_SignedIn() when signedIn != null:
return signedIn();case SyncAccount_Locked() when locked != null:
return locked();case SyncAccount_SessionEnded() when sessionEnded != null:
return sessionEnded();case SyncAccount_Busy() when busy != null:
return busy(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class SyncAccount_SignedOut extends SyncAccount {
  const SyncAccount_SignedOut(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncAccount_SignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncAccount.signedOut()';
}


}




/// @nodoc


class SyncAccount_SignedIn extends SyncAccount {
  const SyncAccount_SignedIn(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncAccount_SignedIn);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncAccount.signedIn()';
}


}




/// @nodoc


class SyncAccount_Locked extends SyncAccount {
  const SyncAccount_Locked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncAccount_Locked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncAccount.locked()';
}


}




/// @nodoc


class SyncAccount_SessionEnded extends SyncAccount {
  const SyncAccount_SessionEnded(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncAccount_SessionEnded);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SyncAccount.sessionEnded()';
}


}




/// @nodoc


class SyncAccount_Busy extends SyncAccount {
  const SyncAccount_Busy(this.field0): super._();
  

 final  AccountStep field0;

/// Create a copy of SyncAccount
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncAccount_BusyCopyWith<SyncAccount_Busy> get copyWith => _$SyncAccount_BusyCopyWithImpl<SyncAccount_Busy>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncAccount_Busy&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'SyncAccount.busy(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $SyncAccount_BusyCopyWith<$Res> implements $SyncAccountCopyWith<$Res> {
  factory $SyncAccount_BusyCopyWith(SyncAccount_Busy value, $Res Function(SyncAccount_Busy) _then) = _$SyncAccount_BusyCopyWithImpl;
@useResult
$Res call({
 AccountStep field0
});




}
/// @nodoc
class _$SyncAccount_BusyCopyWithImpl<$Res>
    implements $SyncAccount_BusyCopyWith<$Res> {
  _$SyncAccount_BusyCopyWithImpl(this._self, this._then);

  final SyncAccount_Busy _self;
  final $Res Function(SyncAccount_Busy) _then;

/// Create a copy of SyncAccount
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(SyncAccount_Busy(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as AccountStep,
  ));
}


}

/// @nodoc
mixin _$Ticked {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Ticked()';
}


}

/// @nodoc
class $TickedCopyWith<$Res>  {
$TickedCopyWith(Ticked _, $Res Function(Ticked) __);
}


/// Adds pattern-matching-related methods to [Ticked].
extension TickedPatterns on Ticked {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( Ticked_Changed value)?  changed,TResult Function( Ticked_Unchanged value)?  unchanged,TResult Function( Ticked_Missing value)?  missing,required TResult orElse(),}){
final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that);case Ticked_Unchanged() when unchanged != null:
return unchanged(_that);case Ticked_Missing() when missing != null:
return missing(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( Ticked_Changed value)  changed,required TResult Function( Ticked_Unchanged value)  unchanged,required TResult Function( Ticked_Missing value)  missing,}){
final _that = this;
switch (_that) {
case Ticked_Changed():
return changed(_that);case Ticked_Unchanged():
return unchanged(_that);case Ticked_Missing():
return missing(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( Ticked_Changed value)?  changed,TResult? Function( Ticked_Unchanged value)?  unchanged,TResult? Function( Ticked_Missing value)?  missing,}){
final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that);case Ticked_Unchanged() when unchanged != null:
return unchanged(_that);case Ticked_Missing() when missing != null:
return missing(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String rev,  int undo,  String message)?  changed,TResult Function()?  unchanged,TResult Function()?  missing,required TResult orElse(),}) {final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that.rev,_that.undo,_that.message);case Ticked_Unchanged() when unchanged != null:
return unchanged();case Ticked_Missing() when missing != null:
return missing();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String rev,  int undo,  String message)  changed,required TResult Function()  unchanged,required TResult Function()  missing,}) {final _that = this;
switch (_that) {
case Ticked_Changed():
return changed(_that.rev,_that.undo,_that.message);case Ticked_Unchanged():
return unchanged();case Ticked_Missing():
return missing();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String rev,  int undo,  String message)?  changed,TResult? Function()?  unchanged,TResult? Function()?  missing,}) {final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that.rev,_that.undo,_that.message);case Ticked_Unchanged() when unchanged != null:
return unchanged();case Ticked_Missing() when missing != null:
return missing();case _:
  return null;

}
}

}

/// @nodoc


class Ticked_Changed extends Ticked {
  const Ticked_Changed({required this.rev, required this.undo, required this.message}): super._();
  

 final  String rev;
 final  int undo;
 final  String message;

/// Create a copy of Ticked
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$Ticked_ChangedCopyWith<Ticked_Changed> get copyWith => _$Ticked_ChangedCopyWithImpl<Ticked_Changed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked_Changed&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.undo, undo) || other.undo == undo)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,rev,undo,message);

@override
String toString() {
  return 'Ticked.changed(rev: $rev, undo: $undo, message: $message)';
}


}

/// @nodoc
abstract mixin class $Ticked_ChangedCopyWith<$Res> implements $TickedCopyWith<$Res> {
  factory $Ticked_ChangedCopyWith(Ticked_Changed value, $Res Function(Ticked_Changed) _then) = _$Ticked_ChangedCopyWithImpl;
@useResult
$Res call({
 String rev, int undo, String message
});




}
/// @nodoc
class _$Ticked_ChangedCopyWithImpl<$Res>
    implements $Ticked_ChangedCopyWith<$Res> {
  _$Ticked_ChangedCopyWithImpl(this._self, this._then);

  final Ticked_Changed _self;
  final $Res Function(Ticked_Changed) _then;

/// Create a copy of Ticked
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,Object? undo = null,Object? message = null,}) {
  return _then(Ticked_Changed(
rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,undo: null == undo ? _self.undo : undo // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class Ticked_Unchanged extends Ticked {
  const Ticked_Unchanged(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked_Unchanged);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Ticked.unchanged()';
}


}




/// @nodoc


class Ticked_Missing extends Ticked {
  const Ticked_Missing(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked_Missing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Ticked.missing()';
}


}




// dart format on
