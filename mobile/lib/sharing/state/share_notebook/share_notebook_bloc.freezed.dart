// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'share_notebook_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ShareNotebookEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent()';
}


}

/// @nodoc
class $ShareNotebookEventCopyWith<$Res>  {
$ShareNotebookEventCopyWith(ShareNotebookEvent _, $Res Function(ShareNotebookEvent) __);
}


/// Adds pattern-matching-related methods to [ShareNotebookEvent].
extension ShareNotebookEventPatterns on ShareNotebookEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ShareNotebookStartedEvent value)?  started,TResult Function( ShareNotebookRetriedEvent value)?  retried,TResult Function( ShareNotebookInviteRoleChosenEvent value)?  inviteRoleChosen,TResult Function( ShareNotebookInvitedEvent value)?  invited,TResult Function( ShareNotebookFingerprintConfirmedEvent value)?  fingerprintConfirmed,TResult Function( ShareNotebookInvitedUnverifiedEvent value)?  invitedUnverified,TResult Function( ShareNotebookNewKeyAcceptedEvent value)?  newKeyAccepted,TResult Function( ShareNotebookPromptDismissedEvent value)?  promptDismissed,TResult Function( ShareNotebookMemberRoleChangedEvent value)?  memberRoleChanged,TResult Function( ShareNotebookMemberRemovedEvent value)?  memberRemoved,TResult Function( ShareNotebookEndedEvent value)?  ended,TResult Function( ShareNotebookProgressedEvent value)?  progressed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ShareNotebookStartedEvent() when started != null:
return started(_that);case ShareNotebookRetriedEvent() when retried != null:
return retried(_that);case ShareNotebookInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that);case ShareNotebookInvitedEvent() when invited != null:
return invited(_that);case ShareNotebookFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed(_that);case ShareNotebookInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified(_that);case ShareNotebookNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted(_that);case ShareNotebookPromptDismissedEvent() when promptDismissed != null:
return promptDismissed(_that);case ShareNotebookMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that);case ShareNotebookMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that);case ShareNotebookEndedEvent() when ended != null:
return ended(_that);case ShareNotebookProgressedEvent() when progressed != null:
return progressed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ShareNotebookStartedEvent value)  started,required TResult Function( ShareNotebookRetriedEvent value)  retried,required TResult Function( ShareNotebookInviteRoleChosenEvent value)  inviteRoleChosen,required TResult Function( ShareNotebookInvitedEvent value)  invited,required TResult Function( ShareNotebookFingerprintConfirmedEvent value)  fingerprintConfirmed,required TResult Function( ShareNotebookInvitedUnverifiedEvent value)  invitedUnverified,required TResult Function( ShareNotebookNewKeyAcceptedEvent value)  newKeyAccepted,required TResult Function( ShareNotebookPromptDismissedEvent value)  promptDismissed,required TResult Function( ShareNotebookMemberRoleChangedEvent value)  memberRoleChanged,required TResult Function( ShareNotebookMemberRemovedEvent value)  memberRemoved,required TResult Function( ShareNotebookEndedEvent value)  ended,required TResult Function( ShareNotebookProgressedEvent value)  progressed,}){
final _that = this;
switch (_that) {
case ShareNotebookStartedEvent():
return started(_that);case ShareNotebookRetriedEvent():
return retried(_that);case ShareNotebookInviteRoleChosenEvent():
return inviteRoleChosen(_that);case ShareNotebookInvitedEvent():
return invited(_that);case ShareNotebookFingerprintConfirmedEvent():
return fingerprintConfirmed(_that);case ShareNotebookInvitedUnverifiedEvent():
return invitedUnverified(_that);case ShareNotebookNewKeyAcceptedEvent():
return newKeyAccepted(_that);case ShareNotebookPromptDismissedEvent():
return promptDismissed(_that);case ShareNotebookMemberRoleChangedEvent():
return memberRoleChanged(_that);case ShareNotebookMemberRemovedEvent():
return memberRemoved(_that);case ShareNotebookEndedEvent():
return ended(_that);case ShareNotebookProgressedEvent():
return progressed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ShareNotebookStartedEvent value)?  started,TResult? Function( ShareNotebookRetriedEvent value)?  retried,TResult? Function( ShareNotebookInviteRoleChosenEvent value)?  inviteRoleChosen,TResult? Function( ShareNotebookInvitedEvent value)?  invited,TResult? Function( ShareNotebookFingerprintConfirmedEvent value)?  fingerprintConfirmed,TResult? Function( ShareNotebookInvitedUnverifiedEvent value)?  invitedUnverified,TResult? Function( ShareNotebookNewKeyAcceptedEvent value)?  newKeyAccepted,TResult? Function( ShareNotebookPromptDismissedEvent value)?  promptDismissed,TResult? Function( ShareNotebookMemberRoleChangedEvent value)?  memberRoleChanged,TResult? Function( ShareNotebookMemberRemovedEvent value)?  memberRemoved,TResult? Function( ShareNotebookEndedEvent value)?  ended,TResult? Function( ShareNotebookProgressedEvent value)?  progressed,}){
final _that = this;
switch (_that) {
case ShareNotebookStartedEvent() when started != null:
return started(_that);case ShareNotebookRetriedEvent() when retried != null:
return retried(_that);case ShareNotebookInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that);case ShareNotebookInvitedEvent() when invited != null:
return invited(_that);case ShareNotebookFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed(_that);case ShareNotebookInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified(_that);case ShareNotebookNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted(_that);case ShareNotebookPromptDismissedEvent() when promptDismissed != null:
return promptDismissed(_that);case ShareNotebookMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that);case ShareNotebookMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that);case ShareNotebookEndedEvent() when ended != null:
return ended(_that);case ShareNotebookProgressedEvent() when progressed != null:
return progressed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String notebookId)?  started,TResult Function()?  retried,TResult Function( BookRole role)?  inviteRoleChosen,TResult Function( String username)?  invited,TResult Function()?  fingerprintConfirmed,TResult Function()?  invitedUnverified,TResult Function()?  newKeyAccepted,TResult Function()?  promptDismissed,TResult Function( String accountId,  BookRole role)?  memberRoleChanged,TResult Function( BookMemberRow member)?  memberRemoved,TResult Function()?  ended,TResult Function( BookShareProgress progress)?  progressed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ShareNotebookStartedEvent() when started != null:
return started(_that.notebookId);case ShareNotebookRetriedEvent() when retried != null:
return retried();case ShareNotebookInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that.role);case ShareNotebookInvitedEvent() when invited != null:
return invited(_that.username);case ShareNotebookFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed();case ShareNotebookInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified();case ShareNotebookNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted();case ShareNotebookPromptDismissedEvent() when promptDismissed != null:
return promptDismissed();case ShareNotebookMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that.accountId,_that.role);case ShareNotebookMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that.member);case ShareNotebookEndedEvent() when ended != null:
return ended();case ShareNotebookProgressedEvent() when progressed != null:
return progressed(_that.progress);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String notebookId)  started,required TResult Function()  retried,required TResult Function( BookRole role)  inviteRoleChosen,required TResult Function( String username)  invited,required TResult Function()  fingerprintConfirmed,required TResult Function()  invitedUnverified,required TResult Function()  newKeyAccepted,required TResult Function()  promptDismissed,required TResult Function( String accountId,  BookRole role)  memberRoleChanged,required TResult Function( BookMemberRow member)  memberRemoved,required TResult Function()  ended,required TResult Function( BookShareProgress progress)  progressed,}) {final _that = this;
switch (_that) {
case ShareNotebookStartedEvent():
return started(_that.notebookId);case ShareNotebookRetriedEvent():
return retried();case ShareNotebookInviteRoleChosenEvent():
return inviteRoleChosen(_that.role);case ShareNotebookInvitedEvent():
return invited(_that.username);case ShareNotebookFingerprintConfirmedEvent():
return fingerprintConfirmed();case ShareNotebookInvitedUnverifiedEvent():
return invitedUnverified();case ShareNotebookNewKeyAcceptedEvent():
return newKeyAccepted();case ShareNotebookPromptDismissedEvent():
return promptDismissed();case ShareNotebookMemberRoleChangedEvent():
return memberRoleChanged(_that.accountId,_that.role);case ShareNotebookMemberRemovedEvent():
return memberRemoved(_that.member);case ShareNotebookEndedEvent():
return ended();case ShareNotebookProgressedEvent():
return progressed(_that.progress);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String notebookId)?  started,TResult? Function()?  retried,TResult? Function( BookRole role)?  inviteRoleChosen,TResult? Function( String username)?  invited,TResult? Function()?  fingerprintConfirmed,TResult? Function()?  invitedUnverified,TResult? Function()?  newKeyAccepted,TResult? Function()?  promptDismissed,TResult? Function( String accountId,  BookRole role)?  memberRoleChanged,TResult? Function( BookMemberRow member)?  memberRemoved,TResult? Function()?  ended,TResult? Function( BookShareProgress progress)?  progressed,}) {final _that = this;
switch (_that) {
case ShareNotebookStartedEvent() when started != null:
return started(_that.notebookId);case ShareNotebookRetriedEvent() when retried != null:
return retried();case ShareNotebookInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that.role);case ShareNotebookInvitedEvent() when invited != null:
return invited(_that.username);case ShareNotebookFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed();case ShareNotebookInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified();case ShareNotebookNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted();case ShareNotebookPromptDismissedEvent() when promptDismissed != null:
return promptDismissed();case ShareNotebookMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that.accountId,_that.role);case ShareNotebookMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that.member);case ShareNotebookEndedEvent() when ended != null:
return ended();case ShareNotebookProgressedEvent() when progressed != null:
return progressed(_that.progress);case _:
  return null;

}
}

}

/// @nodoc


class ShareNotebookStartedEvent implements ShareNotebookEvent {
  const ShareNotebookStartedEvent(this.notebookId);
  

 final  String notebookId;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookStartedEventCopyWith<ShareNotebookStartedEvent> get copyWith => _$ShareNotebookStartedEventCopyWithImpl<ShareNotebookStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookStartedEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'ShareNotebookEvent.started(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookStartedEventCopyWith<$Res> implements $ShareNotebookEventCopyWith<$Res> {
  factory $ShareNotebookStartedEventCopyWith(ShareNotebookStartedEvent value, $Res Function(ShareNotebookStartedEvent) _then) = _$ShareNotebookStartedEventCopyWithImpl;
@useResult
$Res call({
 String notebookId
});




}
/// @nodoc
class _$ShareNotebookStartedEventCopyWithImpl<$Res>
    implements $ShareNotebookStartedEventCopyWith<$Res> {
  _$ShareNotebookStartedEventCopyWithImpl(this._self, this._then);

  final ShareNotebookStartedEvent _self;
  final $Res Function(ShareNotebookStartedEvent) _then;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = null,}) {
  return _then(ShareNotebookStartedEvent(
null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNotebookRetriedEvent implements ShareNotebookEvent {
  const ShareNotebookRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent.retried()';
}


}




/// @nodoc


class ShareNotebookInviteRoleChosenEvent implements ShareNotebookEvent {
  const ShareNotebookInviteRoleChosenEvent(this.role);
  

 final  BookRole role;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookInviteRoleChosenEventCopyWith<ShareNotebookInviteRoleChosenEvent> get copyWith => _$ShareNotebookInviteRoleChosenEventCopyWithImpl<ShareNotebookInviteRoleChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookInviteRoleChosenEvent&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,role);

@override
String toString() {
  return 'ShareNotebookEvent.inviteRoleChosen(role: $role)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookInviteRoleChosenEventCopyWith<$Res> implements $ShareNotebookEventCopyWith<$Res> {
  factory $ShareNotebookInviteRoleChosenEventCopyWith(ShareNotebookInviteRoleChosenEvent value, $Res Function(ShareNotebookInviteRoleChosenEvent) _then) = _$ShareNotebookInviteRoleChosenEventCopyWithImpl;
@useResult
$Res call({
 BookRole role
});




}
/// @nodoc
class _$ShareNotebookInviteRoleChosenEventCopyWithImpl<$Res>
    implements $ShareNotebookInviteRoleChosenEventCopyWith<$Res> {
  _$ShareNotebookInviteRoleChosenEventCopyWithImpl(this._self, this._then);

  final ShareNotebookInviteRoleChosenEvent _self;
  final $Res Function(ShareNotebookInviteRoleChosenEvent) _then;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? role = null,}) {
  return _then(ShareNotebookInviteRoleChosenEvent(
null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as BookRole,
  ));
}


}

/// @nodoc


class ShareNotebookInvitedEvent implements ShareNotebookEvent {
  const ShareNotebookInvitedEvent(this.username);
  

 final  String username;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookInvitedEventCopyWith<ShareNotebookInvitedEvent> get copyWith => _$ShareNotebookInvitedEventCopyWithImpl<ShareNotebookInvitedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookInvitedEvent&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'ShareNotebookEvent.invited(username: $username)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookInvitedEventCopyWith<$Res> implements $ShareNotebookEventCopyWith<$Res> {
  factory $ShareNotebookInvitedEventCopyWith(ShareNotebookInvitedEvent value, $Res Function(ShareNotebookInvitedEvent) _then) = _$ShareNotebookInvitedEventCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$ShareNotebookInvitedEventCopyWithImpl<$Res>
    implements $ShareNotebookInvitedEventCopyWith<$Res> {
  _$ShareNotebookInvitedEventCopyWithImpl(this._self, this._then);

  final ShareNotebookInvitedEvent _self;
  final $Res Function(ShareNotebookInvitedEvent) _then;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(ShareNotebookInvitedEvent(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNotebookFingerprintConfirmedEvent implements ShareNotebookEvent {
  const ShareNotebookFingerprintConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookFingerprintConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent.fingerprintConfirmed()';
}


}




/// @nodoc


class ShareNotebookInvitedUnverifiedEvent implements ShareNotebookEvent {
  const ShareNotebookInvitedUnverifiedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookInvitedUnverifiedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent.invitedUnverified()';
}


}




/// @nodoc


class ShareNotebookNewKeyAcceptedEvent implements ShareNotebookEvent {
  const ShareNotebookNewKeyAcceptedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookNewKeyAcceptedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent.newKeyAccepted()';
}


}




/// @nodoc


class ShareNotebookPromptDismissedEvent implements ShareNotebookEvent {
  const ShareNotebookPromptDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookPromptDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent.promptDismissed()';
}


}




/// @nodoc


class ShareNotebookMemberRoleChangedEvent implements ShareNotebookEvent {
  const ShareNotebookMemberRoleChangedEvent(this.accountId, this.role);
  

 final  String accountId;
 final  BookRole role;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookMemberRoleChangedEventCopyWith<ShareNotebookMemberRoleChangedEvent> get copyWith => _$ShareNotebookMemberRoleChangedEventCopyWithImpl<ShareNotebookMemberRoleChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookMemberRoleChangedEvent&&(identical(other.accountId, accountId) || other.accountId == accountId)&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,accountId,role);

@override
String toString() {
  return 'ShareNotebookEvent.memberRoleChanged(accountId: $accountId, role: $role)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookMemberRoleChangedEventCopyWith<$Res> implements $ShareNotebookEventCopyWith<$Res> {
  factory $ShareNotebookMemberRoleChangedEventCopyWith(ShareNotebookMemberRoleChangedEvent value, $Res Function(ShareNotebookMemberRoleChangedEvent) _then) = _$ShareNotebookMemberRoleChangedEventCopyWithImpl;
@useResult
$Res call({
 String accountId, BookRole role
});




}
/// @nodoc
class _$ShareNotebookMemberRoleChangedEventCopyWithImpl<$Res>
    implements $ShareNotebookMemberRoleChangedEventCopyWith<$Res> {
  _$ShareNotebookMemberRoleChangedEventCopyWithImpl(this._self, this._then);

  final ShareNotebookMemberRoleChangedEvent _self;
  final $Res Function(ShareNotebookMemberRoleChangedEvent) _then;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? accountId = null,Object? role = null,}) {
  return _then(ShareNotebookMemberRoleChangedEvent(
null == accountId ? _self.accountId : accountId // ignore: cast_nullable_to_non_nullable
as String,null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as BookRole,
  ));
}


}

/// @nodoc


class ShareNotebookMemberRemovedEvent implements ShareNotebookEvent {
  const ShareNotebookMemberRemovedEvent(this.member);
  

 final  BookMemberRow member;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookMemberRemovedEventCopyWith<ShareNotebookMemberRemovedEvent> get copyWith => _$ShareNotebookMemberRemovedEventCopyWithImpl<ShareNotebookMemberRemovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookMemberRemovedEvent&&(identical(other.member, member) || other.member == member));
}


@override
int get hashCode => Object.hash(runtimeType,member);

@override
String toString() {
  return 'ShareNotebookEvent.memberRemoved(member: $member)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookMemberRemovedEventCopyWith<$Res> implements $ShareNotebookEventCopyWith<$Res> {
  factory $ShareNotebookMemberRemovedEventCopyWith(ShareNotebookMemberRemovedEvent value, $Res Function(ShareNotebookMemberRemovedEvent) _then) = _$ShareNotebookMemberRemovedEventCopyWithImpl;
@useResult
$Res call({
 BookMemberRow member
});




}
/// @nodoc
class _$ShareNotebookMemberRemovedEventCopyWithImpl<$Res>
    implements $ShareNotebookMemberRemovedEventCopyWith<$Res> {
  _$ShareNotebookMemberRemovedEventCopyWithImpl(this._self, this._then);

  final ShareNotebookMemberRemovedEvent _self;
  final $Res Function(ShareNotebookMemberRemovedEvent) _then;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? member = null,}) {
  return _then(ShareNotebookMemberRemovedEvent(
null == member ? _self.member : member // ignore: cast_nullable_to_non_nullable
as BookMemberRow,
  ));
}


}

/// @nodoc


class ShareNotebookEndedEvent implements ShareNotebookEvent {
  const ShareNotebookEndedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookEndedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNotebookEvent.ended()';
}


}




/// @nodoc


class ShareNotebookProgressedEvent implements ShareNotebookEvent {
  const ShareNotebookProgressedEvent(this.progress);
  

 final  BookShareProgress progress;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookProgressedEventCopyWith<ShareNotebookProgressedEvent> get copyWith => _$ShareNotebookProgressedEventCopyWithImpl<ShareNotebookProgressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookProgressedEvent&&(identical(other.progress, progress) || other.progress == progress));
}


@override
int get hashCode => Object.hash(runtimeType,progress);

@override
String toString() {
  return 'ShareNotebookEvent.progressed(progress: $progress)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookProgressedEventCopyWith<$Res> implements $ShareNotebookEventCopyWith<$Res> {
  factory $ShareNotebookProgressedEventCopyWith(ShareNotebookProgressedEvent value, $Res Function(ShareNotebookProgressedEvent) _then) = _$ShareNotebookProgressedEventCopyWithImpl;
@useResult
$Res call({
 BookShareProgress progress
});




}
/// @nodoc
class _$ShareNotebookProgressedEventCopyWithImpl<$Res>
    implements $ShareNotebookProgressedEventCopyWith<$Res> {
  _$ShareNotebookProgressedEventCopyWithImpl(this._self, this._then);

  final ShareNotebookProgressedEvent _self;
  final $Res Function(ShareNotebookProgressedEvent) _then;

/// Create a copy of ShareNotebookEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? progress = null,}) {
  return _then(ShareNotebookProgressedEvent(
null == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as BookShareProgress,
  ));
}


}

/// @nodoc
mixin _$ShareNotebookState {

 String get notebookId; ShareSheetStatus get status;/// Why the notebook can't be shared, in the library's words.
 String? get reason;/// What sharing it would do, while it isn't shared.
 BookShareCheck? get check;/// Its sharing, once it is shared.
 BookShareInfo? get info; BookRole get inviteRole;/// A step is running: nothing can be pressed.
 bool get busy;/// How far sharing it has got, while it is being shared.
 BookShareProgress? get progress; BookPrompt? get prompt; ShareProblem? get problem; BookDone? get done;
/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNotebookStateCopyWith<ShareNotebookState> get copyWith => _$ShareNotebookStateCopyWithImpl<ShareNotebookState>(this as ShareNotebookState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotebookState&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.status, status) || other.status == status)&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.check, check) || other.check == check)&&(identical(other.info, info) || other.info == info)&&(identical(other.inviteRole, inviteRole) || other.inviteRole == inviteRole)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.prompt, prompt) || other.prompt == prompt)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.done, done) || other.done == done));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId,status,reason,check,info,inviteRole,busy,progress,prompt,problem,done);

@override
String toString() {
  return 'ShareNotebookState(notebookId: $notebookId, status: $status, reason: $reason, check: $check, info: $info, inviteRole: $inviteRole, busy: $busy, progress: $progress, prompt: $prompt, problem: $problem, done: $done)';
}


}

/// @nodoc
abstract mixin class $ShareNotebookStateCopyWith<$Res>  {
  factory $ShareNotebookStateCopyWith(ShareNotebookState value, $Res Function(ShareNotebookState) _then) = _$ShareNotebookStateCopyWithImpl;
@useResult
$Res call({
 String notebookId, ShareSheetStatus status, String? reason, BookShareCheck? check, BookShareInfo? info, BookRole inviteRole, bool busy, BookShareProgress? progress, BookPrompt? prompt, ShareProblem? problem, BookDone? done
});


$BookPromptCopyWith<$Res>? get prompt;$ShareProblemCopyWith<$Res>? get problem;$BookDoneCopyWith<$Res>? get done;

}
/// @nodoc
class _$ShareNotebookStateCopyWithImpl<$Res>
    implements $ShareNotebookStateCopyWith<$Res> {
  _$ShareNotebookStateCopyWithImpl(this._self, this._then);

  final ShareNotebookState _self;
  final $Res Function(ShareNotebookState) _then;

/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? notebookId = null,Object? status = null,Object? reason = freezed,Object? check = freezed,Object? info = freezed,Object? inviteRole = null,Object? busy = null,Object? progress = freezed,Object? prompt = freezed,Object? problem = freezed,Object? done = freezed,}) {
  return _then(_self.copyWith(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as ShareSheetStatus,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,check: freezed == check ? _self.check : check // ignore: cast_nullable_to_non_nullable
as BookShareCheck?,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as BookShareInfo?,inviteRole: null == inviteRole ? _self.inviteRole : inviteRole // ignore: cast_nullable_to_non_nullable
as BookRole,busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as BookShareProgress?,prompt: freezed == prompt ? _self.prompt : prompt // ignore: cast_nullable_to_non_nullable
as BookPrompt?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,done: freezed == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as BookDone?,
  ));
}
/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BookPromptCopyWith<$Res>? get prompt {
    if (_self.prompt == null) {
    return null;
  }

  return $BookPromptCopyWith<$Res>(_self.prompt!, (value) {
    return _then(_self.copyWith(prompt: value));
  });
}/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ShareProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $ShareProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BookDoneCopyWith<$Res>? get done {
    if (_self.done == null) {
    return null;
  }

  return $BookDoneCopyWith<$Res>(_self.done!, (value) {
    return _then(_self.copyWith(done: value));
  });
}
}


/// Adds pattern-matching-related methods to [ShareNotebookState].
extension ShareNotebookStatePatterns on ShareNotebookState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ShareNotebookState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ShareNotebookState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ShareNotebookState value)  $default,){
final _that = this;
switch (_that) {
case _ShareNotebookState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ShareNotebookState value)?  $default,){
final _that = this;
switch (_that) {
case _ShareNotebookState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String notebookId,  ShareSheetStatus status,  String? reason,  BookShareCheck? check,  BookShareInfo? info,  BookRole inviteRole,  bool busy,  BookShareProgress? progress,  BookPrompt? prompt,  ShareProblem? problem,  BookDone? done)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ShareNotebookState() when $default != null:
return $default(_that.notebookId,_that.status,_that.reason,_that.check,_that.info,_that.inviteRole,_that.busy,_that.progress,_that.prompt,_that.problem,_that.done);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String notebookId,  ShareSheetStatus status,  String? reason,  BookShareCheck? check,  BookShareInfo? info,  BookRole inviteRole,  bool busy,  BookShareProgress? progress,  BookPrompt? prompt,  ShareProblem? problem,  BookDone? done)  $default,) {final _that = this;
switch (_that) {
case _ShareNotebookState():
return $default(_that.notebookId,_that.status,_that.reason,_that.check,_that.info,_that.inviteRole,_that.busy,_that.progress,_that.prompt,_that.problem,_that.done);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String notebookId,  ShareSheetStatus status,  String? reason,  BookShareCheck? check,  BookShareInfo? info,  BookRole inviteRole,  bool busy,  BookShareProgress? progress,  BookPrompt? prompt,  ShareProblem? problem,  BookDone? done)?  $default,) {final _that = this;
switch (_that) {
case _ShareNotebookState() when $default != null:
return $default(_that.notebookId,_that.status,_that.reason,_that.check,_that.info,_that.inviteRole,_that.busy,_that.progress,_that.prompt,_that.problem,_that.done);case _:
  return null;

}
}

}

/// @nodoc


class _ShareNotebookState extends ShareNotebookState {
  const _ShareNotebookState({this.notebookId = '', this.status = ShareSheetStatus.loading, this.reason, this.check, this.info, this.inviteRole = BookRole.editor, this.busy = false, this.progress, this.prompt, this.problem, this.done}): super._();
  

@override@JsonKey() final  String notebookId;
@override@JsonKey() final  ShareSheetStatus status;
/// Why the notebook can't be shared, in the library's words.
@override final  String? reason;
/// What sharing it would do, while it isn't shared.
@override final  BookShareCheck? check;
/// Its sharing, once it is shared.
@override final  BookShareInfo? info;
@override@JsonKey() final  BookRole inviteRole;
/// A step is running: nothing can be pressed.
@override@JsonKey() final  bool busy;
/// How far sharing it has got, while it is being shared.
@override final  BookShareProgress? progress;
@override final  BookPrompt? prompt;
@override final  ShareProblem? problem;
@override final  BookDone? done;

/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ShareNotebookStateCopyWith<_ShareNotebookState> get copyWith => __$ShareNotebookStateCopyWithImpl<_ShareNotebookState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ShareNotebookState&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.status, status) || other.status == status)&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.check, check) || other.check == check)&&(identical(other.info, info) || other.info == info)&&(identical(other.inviteRole, inviteRole) || other.inviteRole == inviteRole)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.prompt, prompt) || other.prompt == prompt)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.done, done) || other.done == done));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId,status,reason,check,info,inviteRole,busy,progress,prompt,problem,done);

@override
String toString() {
  return 'ShareNotebookState(notebookId: $notebookId, status: $status, reason: $reason, check: $check, info: $info, inviteRole: $inviteRole, busy: $busy, progress: $progress, prompt: $prompt, problem: $problem, done: $done)';
}


}

/// @nodoc
abstract mixin class _$ShareNotebookStateCopyWith<$Res> implements $ShareNotebookStateCopyWith<$Res> {
  factory _$ShareNotebookStateCopyWith(_ShareNotebookState value, $Res Function(_ShareNotebookState) _then) = __$ShareNotebookStateCopyWithImpl;
@override @useResult
$Res call({
 String notebookId, ShareSheetStatus status, String? reason, BookShareCheck? check, BookShareInfo? info, BookRole inviteRole, bool busy, BookShareProgress? progress, BookPrompt? prompt, ShareProblem? problem, BookDone? done
});


@override $BookPromptCopyWith<$Res>? get prompt;@override $ShareProblemCopyWith<$Res>? get problem;@override $BookDoneCopyWith<$Res>? get done;

}
/// @nodoc
class __$ShareNotebookStateCopyWithImpl<$Res>
    implements _$ShareNotebookStateCopyWith<$Res> {
  __$ShareNotebookStateCopyWithImpl(this._self, this._then);

  final _ShareNotebookState _self;
  final $Res Function(_ShareNotebookState) _then;

/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? notebookId = null,Object? status = null,Object? reason = freezed,Object? check = freezed,Object? info = freezed,Object? inviteRole = null,Object? busy = null,Object? progress = freezed,Object? prompt = freezed,Object? problem = freezed,Object? done = freezed,}) {
  return _then(_ShareNotebookState(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as ShareSheetStatus,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,check: freezed == check ? _self.check : check // ignore: cast_nullable_to_non_nullable
as BookShareCheck?,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as BookShareInfo?,inviteRole: null == inviteRole ? _self.inviteRole : inviteRole // ignore: cast_nullable_to_non_nullable
as BookRole,busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as BookShareProgress?,prompt: freezed == prompt ? _self.prompt : prompt // ignore: cast_nullable_to_non_nullable
as BookPrompt?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,done: freezed == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as BookDone?,
  ));
}

/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BookPromptCopyWith<$Res>? get prompt {
    if (_self.prompt == null) {
    return null;
  }

  return $BookPromptCopyWith<$Res>(_self.prompt!, (value) {
    return _then(_self.copyWith(prompt: value));
  });
}/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ShareProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $ShareProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}/// Create a copy of ShareNotebookState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BookDoneCopyWith<$Res>? get done {
    if (_self.done == null) {
    return null;
  }

  return $BookDoneCopyWith<$Res>(_self.done!, (value) {
    return _then(_self.copyWith(done: value));
  });
}
}

// dart format on
