// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'share_note_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ShareNoteEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent()';
}


}

/// @nodoc
class $ShareNoteEventCopyWith<$Res>  {
$ShareNoteEventCopyWith(ShareNoteEvent _, $Res Function(ShareNoteEvent) __);
}


/// Adds pattern-matching-related methods to [ShareNoteEvent].
extension ShareNoteEventPatterns on ShareNoteEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ShareNoteStartedEvent value)?  started,TResult Function( ShareNoteRetriedEvent value)?  retried,TResult Function( ShareNoteInviteRoleChosenEvent value)?  inviteRoleChosen,TResult Function( ShareNoteInvitedEvent value)?  invited,TResult Function( ShareNoteFingerprintConfirmedEvent value)?  fingerprintConfirmed,TResult Function( ShareNoteInvitedUnverifiedEvent value)?  invitedUnverified,TResult Function( ShareNoteNewKeyAcceptedEvent value)?  newKeyAccepted,TResult Function( ShareNotePromptDismissedEvent value)?  promptDismissed,TResult Function( ShareNoteKeyReviewedEvent value)?  keyReviewed,TResult Function( ShareNoteMemberRoleChangedEvent value)?  memberRoleChanged,TResult Function( ShareNoteMemberRemovedEvent value)?  memberRemoved,TResult Function( ShareNoteRequestApprovedEvent value)?  requestApproved,TResult Function( ShareNoteRequestDeclinedEvent value)?  requestDeclined,TResult Function( ShareNoteLinkRoleChosenEvent value)?  linkRoleChosen,TResult Function( ShareNoteLinkDaysChosenEvent value)?  linkDaysChosen,TResult Function( ShareNoteLinkMadeEvent value)?  linkMade,TResult Function( ShareNoteLinkCancelledEvent value)?  linkCancelled,TResult Function( ShareNoteLinkCopiedEvent value)?  linkCopied,TResult Function( ShareNoteLinkSentEvent value)?  linkSent,TResult Function( ShareNoteEndAskedEvent value)?  endAsked,TResult Function( ShareNoteEndDismissedEvent value)?  endDismissed,TResult Function( ShareNoteEndConfirmedEvent value)?  endConfirmed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ShareNoteStartedEvent() when started != null:
return started(_that);case ShareNoteRetriedEvent() when retried != null:
return retried(_that);case ShareNoteInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that);case ShareNoteInvitedEvent() when invited != null:
return invited(_that);case ShareNoteFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed(_that);case ShareNoteInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified(_that);case ShareNoteNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted(_that);case ShareNotePromptDismissedEvent() when promptDismissed != null:
return promptDismissed(_that);case ShareNoteKeyReviewedEvent() when keyReviewed != null:
return keyReviewed(_that);case ShareNoteMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that);case ShareNoteMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that);case ShareNoteRequestApprovedEvent() when requestApproved != null:
return requestApproved(_that);case ShareNoteRequestDeclinedEvent() when requestDeclined != null:
return requestDeclined(_that);case ShareNoteLinkRoleChosenEvent() when linkRoleChosen != null:
return linkRoleChosen(_that);case ShareNoteLinkDaysChosenEvent() when linkDaysChosen != null:
return linkDaysChosen(_that);case ShareNoteLinkMadeEvent() when linkMade != null:
return linkMade(_that);case ShareNoteLinkCancelledEvent() when linkCancelled != null:
return linkCancelled(_that);case ShareNoteLinkCopiedEvent() when linkCopied != null:
return linkCopied(_that);case ShareNoteLinkSentEvent() when linkSent != null:
return linkSent(_that);case ShareNoteEndAskedEvent() when endAsked != null:
return endAsked(_that);case ShareNoteEndDismissedEvent() when endDismissed != null:
return endDismissed(_that);case ShareNoteEndConfirmedEvent() when endConfirmed != null:
return endConfirmed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ShareNoteStartedEvent value)  started,required TResult Function( ShareNoteRetriedEvent value)  retried,required TResult Function( ShareNoteInviteRoleChosenEvent value)  inviteRoleChosen,required TResult Function( ShareNoteInvitedEvent value)  invited,required TResult Function( ShareNoteFingerprintConfirmedEvent value)  fingerprintConfirmed,required TResult Function( ShareNoteInvitedUnverifiedEvent value)  invitedUnverified,required TResult Function( ShareNoteNewKeyAcceptedEvent value)  newKeyAccepted,required TResult Function( ShareNotePromptDismissedEvent value)  promptDismissed,required TResult Function( ShareNoteKeyReviewedEvent value)  keyReviewed,required TResult Function( ShareNoteMemberRoleChangedEvent value)  memberRoleChanged,required TResult Function( ShareNoteMemberRemovedEvent value)  memberRemoved,required TResult Function( ShareNoteRequestApprovedEvent value)  requestApproved,required TResult Function( ShareNoteRequestDeclinedEvent value)  requestDeclined,required TResult Function( ShareNoteLinkRoleChosenEvent value)  linkRoleChosen,required TResult Function( ShareNoteLinkDaysChosenEvent value)  linkDaysChosen,required TResult Function( ShareNoteLinkMadeEvent value)  linkMade,required TResult Function( ShareNoteLinkCancelledEvent value)  linkCancelled,required TResult Function( ShareNoteLinkCopiedEvent value)  linkCopied,required TResult Function( ShareNoteLinkSentEvent value)  linkSent,required TResult Function( ShareNoteEndAskedEvent value)  endAsked,required TResult Function( ShareNoteEndDismissedEvent value)  endDismissed,required TResult Function( ShareNoteEndConfirmedEvent value)  endConfirmed,}){
final _that = this;
switch (_that) {
case ShareNoteStartedEvent():
return started(_that);case ShareNoteRetriedEvent():
return retried(_that);case ShareNoteInviteRoleChosenEvent():
return inviteRoleChosen(_that);case ShareNoteInvitedEvent():
return invited(_that);case ShareNoteFingerprintConfirmedEvent():
return fingerprintConfirmed(_that);case ShareNoteInvitedUnverifiedEvent():
return invitedUnverified(_that);case ShareNoteNewKeyAcceptedEvent():
return newKeyAccepted(_that);case ShareNotePromptDismissedEvent():
return promptDismissed(_that);case ShareNoteKeyReviewedEvent():
return keyReviewed(_that);case ShareNoteMemberRoleChangedEvent():
return memberRoleChanged(_that);case ShareNoteMemberRemovedEvent():
return memberRemoved(_that);case ShareNoteRequestApprovedEvent():
return requestApproved(_that);case ShareNoteRequestDeclinedEvent():
return requestDeclined(_that);case ShareNoteLinkRoleChosenEvent():
return linkRoleChosen(_that);case ShareNoteLinkDaysChosenEvent():
return linkDaysChosen(_that);case ShareNoteLinkMadeEvent():
return linkMade(_that);case ShareNoteLinkCancelledEvent():
return linkCancelled(_that);case ShareNoteLinkCopiedEvent():
return linkCopied(_that);case ShareNoteLinkSentEvent():
return linkSent(_that);case ShareNoteEndAskedEvent():
return endAsked(_that);case ShareNoteEndDismissedEvent():
return endDismissed(_that);case ShareNoteEndConfirmedEvent():
return endConfirmed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ShareNoteStartedEvent value)?  started,TResult? Function( ShareNoteRetriedEvent value)?  retried,TResult? Function( ShareNoteInviteRoleChosenEvent value)?  inviteRoleChosen,TResult? Function( ShareNoteInvitedEvent value)?  invited,TResult? Function( ShareNoteFingerprintConfirmedEvent value)?  fingerprintConfirmed,TResult? Function( ShareNoteInvitedUnverifiedEvent value)?  invitedUnverified,TResult? Function( ShareNoteNewKeyAcceptedEvent value)?  newKeyAccepted,TResult? Function( ShareNotePromptDismissedEvent value)?  promptDismissed,TResult? Function( ShareNoteKeyReviewedEvent value)?  keyReviewed,TResult? Function( ShareNoteMemberRoleChangedEvent value)?  memberRoleChanged,TResult? Function( ShareNoteMemberRemovedEvent value)?  memberRemoved,TResult? Function( ShareNoteRequestApprovedEvent value)?  requestApproved,TResult? Function( ShareNoteRequestDeclinedEvent value)?  requestDeclined,TResult? Function( ShareNoteLinkRoleChosenEvent value)?  linkRoleChosen,TResult? Function( ShareNoteLinkDaysChosenEvent value)?  linkDaysChosen,TResult? Function( ShareNoteLinkMadeEvent value)?  linkMade,TResult? Function( ShareNoteLinkCancelledEvent value)?  linkCancelled,TResult? Function( ShareNoteLinkCopiedEvent value)?  linkCopied,TResult? Function( ShareNoteLinkSentEvent value)?  linkSent,TResult? Function( ShareNoteEndAskedEvent value)?  endAsked,TResult? Function( ShareNoteEndDismissedEvent value)?  endDismissed,TResult? Function( ShareNoteEndConfirmedEvent value)?  endConfirmed,}){
final _that = this;
switch (_that) {
case ShareNoteStartedEvent() when started != null:
return started(_that);case ShareNoteRetriedEvent() when retried != null:
return retried(_that);case ShareNoteInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that);case ShareNoteInvitedEvent() when invited != null:
return invited(_that);case ShareNoteFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed(_that);case ShareNoteInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified(_that);case ShareNoteNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted(_that);case ShareNotePromptDismissedEvent() when promptDismissed != null:
return promptDismissed(_that);case ShareNoteKeyReviewedEvent() when keyReviewed != null:
return keyReviewed(_that);case ShareNoteMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that);case ShareNoteMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that);case ShareNoteRequestApprovedEvent() when requestApproved != null:
return requestApproved(_that);case ShareNoteRequestDeclinedEvent() when requestDeclined != null:
return requestDeclined(_that);case ShareNoteLinkRoleChosenEvent() when linkRoleChosen != null:
return linkRoleChosen(_that);case ShareNoteLinkDaysChosenEvent() when linkDaysChosen != null:
return linkDaysChosen(_that);case ShareNoteLinkMadeEvent() when linkMade != null:
return linkMade(_that);case ShareNoteLinkCancelledEvent() when linkCancelled != null:
return linkCancelled(_that);case ShareNoteLinkCopiedEvent() when linkCopied != null:
return linkCopied(_that);case ShareNoteLinkSentEvent() when linkSent != null:
return linkSent(_that);case ShareNoteEndAskedEvent() when endAsked != null:
return endAsked(_that);case ShareNoteEndDismissedEvent() when endDismissed != null:
return endDismissed(_that);case ShareNoteEndConfirmedEvent() when endConfirmed != null:
return endConfirmed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function()?  retried,TResult Function( ShareRole role)?  inviteRoleChosen,TResult Function( String username)?  invited,TResult Function()?  fingerprintConfirmed,TResult Function()?  invitedUnverified,TResult Function()?  newKeyAccepted,TResult Function()?  promptDismissed,TResult Function( String username)?  keyReviewed,TResult Function( String accountId,  ShareRole role)?  memberRoleChanged,TResult Function( String accountId)?  memberRemoved,TResult Function( String accountId)?  requestApproved,TResult Function( String accountId)?  requestDeclined,TResult Function( ShareRole role)?  linkRoleChosen,TResult Function( int days)?  linkDaysChosen,TResult Function()?  linkMade,TResult Function( String linkId)?  linkCancelled,TResult Function()?  linkCopied,TResult Function( Rect? origin)?  linkSent,TResult Function()?  endAsked,TResult Function()?  endDismissed,TResult Function()?  endConfirmed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ShareNoteStartedEvent() when started != null:
return started(_that.noteId);case ShareNoteRetriedEvent() when retried != null:
return retried();case ShareNoteInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that.role);case ShareNoteInvitedEvent() when invited != null:
return invited(_that.username);case ShareNoteFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed();case ShareNoteInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified();case ShareNoteNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted();case ShareNotePromptDismissedEvent() when promptDismissed != null:
return promptDismissed();case ShareNoteKeyReviewedEvent() when keyReviewed != null:
return keyReviewed(_that.username);case ShareNoteMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that.accountId,_that.role);case ShareNoteMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that.accountId);case ShareNoteRequestApprovedEvent() when requestApproved != null:
return requestApproved(_that.accountId);case ShareNoteRequestDeclinedEvent() when requestDeclined != null:
return requestDeclined(_that.accountId);case ShareNoteLinkRoleChosenEvent() when linkRoleChosen != null:
return linkRoleChosen(_that.role);case ShareNoteLinkDaysChosenEvent() when linkDaysChosen != null:
return linkDaysChosen(_that.days);case ShareNoteLinkMadeEvent() when linkMade != null:
return linkMade();case ShareNoteLinkCancelledEvent() when linkCancelled != null:
return linkCancelled(_that.linkId);case ShareNoteLinkCopiedEvent() when linkCopied != null:
return linkCopied();case ShareNoteLinkSentEvent() when linkSent != null:
return linkSent(_that.origin);case ShareNoteEndAskedEvent() when endAsked != null:
return endAsked();case ShareNoteEndDismissedEvent() when endDismissed != null:
return endDismissed();case ShareNoteEndConfirmedEvent() when endConfirmed != null:
return endConfirmed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function()  retried,required TResult Function( ShareRole role)  inviteRoleChosen,required TResult Function( String username)  invited,required TResult Function()  fingerprintConfirmed,required TResult Function()  invitedUnverified,required TResult Function()  newKeyAccepted,required TResult Function()  promptDismissed,required TResult Function( String username)  keyReviewed,required TResult Function( String accountId,  ShareRole role)  memberRoleChanged,required TResult Function( String accountId)  memberRemoved,required TResult Function( String accountId)  requestApproved,required TResult Function( String accountId)  requestDeclined,required TResult Function( ShareRole role)  linkRoleChosen,required TResult Function( int days)  linkDaysChosen,required TResult Function()  linkMade,required TResult Function( String linkId)  linkCancelled,required TResult Function()  linkCopied,required TResult Function( Rect? origin)  linkSent,required TResult Function()  endAsked,required TResult Function()  endDismissed,required TResult Function()  endConfirmed,}) {final _that = this;
switch (_that) {
case ShareNoteStartedEvent():
return started(_that.noteId);case ShareNoteRetriedEvent():
return retried();case ShareNoteInviteRoleChosenEvent():
return inviteRoleChosen(_that.role);case ShareNoteInvitedEvent():
return invited(_that.username);case ShareNoteFingerprintConfirmedEvent():
return fingerprintConfirmed();case ShareNoteInvitedUnverifiedEvent():
return invitedUnverified();case ShareNoteNewKeyAcceptedEvent():
return newKeyAccepted();case ShareNotePromptDismissedEvent():
return promptDismissed();case ShareNoteKeyReviewedEvent():
return keyReviewed(_that.username);case ShareNoteMemberRoleChangedEvent():
return memberRoleChanged(_that.accountId,_that.role);case ShareNoteMemberRemovedEvent():
return memberRemoved(_that.accountId);case ShareNoteRequestApprovedEvent():
return requestApproved(_that.accountId);case ShareNoteRequestDeclinedEvent():
return requestDeclined(_that.accountId);case ShareNoteLinkRoleChosenEvent():
return linkRoleChosen(_that.role);case ShareNoteLinkDaysChosenEvent():
return linkDaysChosen(_that.days);case ShareNoteLinkMadeEvent():
return linkMade();case ShareNoteLinkCancelledEvent():
return linkCancelled(_that.linkId);case ShareNoteLinkCopiedEvent():
return linkCopied();case ShareNoteLinkSentEvent():
return linkSent(_that.origin);case ShareNoteEndAskedEvent():
return endAsked();case ShareNoteEndDismissedEvent():
return endDismissed();case ShareNoteEndConfirmedEvent():
return endConfirmed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function()?  retried,TResult? Function( ShareRole role)?  inviteRoleChosen,TResult? Function( String username)?  invited,TResult? Function()?  fingerprintConfirmed,TResult? Function()?  invitedUnverified,TResult? Function()?  newKeyAccepted,TResult? Function()?  promptDismissed,TResult? Function( String username)?  keyReviewed,TResult? Function( String accountId,  ShareRole role)?  memberRoleChanged,TResult? Function( String accountId)?  memberRemoved,TResult? Function( String accountId)?  requestApproved,TResult? Function( String accountId)?  requestDeclined,TResult? Function( ShareRole role)?  linkRoleChosen,TResult? Function( int days)?  linkDaysChosen,TResult? Function()?  linkMade,TResult? Function( String linkId)?  linkCancelled,TResult? Function()?  linkCopied,TResult? Function( Rect? origin)?  linkSent,TResult? Function()?  endAsked,TResult? Function()?  endDismissed,TResult? Function()?  endConfirmed,}) {final _that = this;
switch (_that) {
case ShareNoteStartedEvent() when started != null:
return started(_that.noteId);case ShareNoteRetriedEvent() when retried != null:
return retried();case ShareNoteInviteRoleChosenEvent() when inviteRoleChosen != null:
return inviteRoleChosen(_that.role);case ShareNoteInvitedEvent() when invited != null:
return invited(_that.username);case ShareNoteFingerprintConfirmedEvent() when fingerprintConfirmed != null:
return fingerprintConfirmed();case ShareNoteInvitedUnverifiedEvent() when invitedUnverified != null:
return invitedUnverified();case ShareNoteNewKeyAcceptedEvent() when newKeyAccepted != null:
return newKeyAccepted();case ShareNotePromptDismissedEvent() when promptDismissed != null:
return promptDismissed();case ShareNoteKeyReviewedEvent() when keyReviewed != null:
return keyReviewed(_that.username);case ShareNoteMemberRoleChangedEvent() when memberRoleChanged != null:
return memberRoleChanged(_that.accountId,_that.role);case ShareNoteMemberRemovedEvent() when memberRemoved != null:
return memberRemoved(_that.accountId);case ShareNoteRequestApprovedEvent() when requestApproved != null:
return requestApproved(_that.accountId);case ShareNoteRequestDeclinedEvent() when requestDeclined != null:
return requestDeclined(_that.accountId);case ShareNoteLinkRoleChosenEvent() when linkRoleChosen != null:
return linkRoleChosen(_that.role);case ShareNoteLinkDaysChosenEvent() when linkDaysChosen != null:
return linkDaysChosen(_that.days);case ShareNoteLinkMadeEvent() when linkMade != null:
return linkMade();case ShareNoteLinkCancelledEvent() when linkCancelled != null:
return linkCancelled(_that.linkId);case ShareNoteLinkCopiedEvent() when linkCopied != null:
return linkCopied();case ShareNoteLinkSentEvent() when linkSent != null:
return linkSent(_that.origin);case ShareNoteEndAskedEvent() when endAsked != null:
return endAsked();case ShareNoteEndDismissedEvent() when endDismissed != null:
return endDismissed();case ShareNoteEndConfirmedEvent() when endConfirmed != null:
return endConfirmed();case _:
  return null;

}
}

}

/// @nodoc


class ShareNoteStartedEvent implements ShareNoteEvent {
  const ShareNoteStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteStartedEventCopyWith<ShareNoteStartedEvent> get copyWith => _$ShareNoteStartedEventCopyWithImpl<ShareNoteStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'ShareNoteEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $ShareNoteStartedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteStartedEventCopyWith(ShareNoteStartedEvent value, $Res Function(ShareNoteStartedEvent) _then) = _$ShareNoteStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$ShareNoteStartedEventCopyWithImpl<$Res>
    implements $ShareNoteStartedEventCopyWith<$Res> {
  _$ShareNoteStartedEventCopyWithImpl(this._self, this._then);

  final ShareNoteStartedEvent _self;
  final $Res Function(ShareNoteStartedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(ShareNoteStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteRetriedEvent implements ShareNoteEvent {
  const ShareNoteRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.retried()';
}


}




/// @nodoc


class ShareNoteInviteRoleChosenEvent implements ShareNoteEvent {
  const ShareNoteInviteRoleChosenEvent(this.role);
  

 final  ShareRole role;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteInviteRoleChosenEventCopyWith<ShareNoteInviteRoleChosenEvent> get copyWith => _$ShareNoteInviteRoleChosenEventCopyWithImpl<ShareNoteInviteRoleChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteInviteRoleChosenEvent&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,role);

@override
String toString() {
  return 'ShareNoteEvent.inviteRoleChosen(role: $role)';
}


}

/// @nodoc
abstract mixin class $ShareNoteInviteRoleChosenEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteInviteRoleChosenEventCopyWith(ShareNoteInviteRoleChosenEvent value, $Res Function(ShareNoteInviteRoleChosenEvent) _then) = _$ShareNoteInviteRoleChosenEventCopyWithImpl;
@useResult
$Res call({
 ShareRole role
});




}
/// @nodoc
class _$ShareNoteInviteRoleChosenEventCopyWithImpl<$Res>
    implements $ShareNoteInviteRoleChosenEventCopyWith<$Res> {
  _$ShareNoteInviteRoleChosenEventCopyWithImpl(this._self, this._then);

  final ShareNoteInviteRoleChosenEvent _self;
  final $Res Function(ShareNoteInviteRoleChosenEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? role = null,}) {
  return _then(ShareNoteInviteRoleChosenEvent(
null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole,
  ));
}


}

/// @nodoc


class ShareNoteInvitedEvent implements ShareNoteEvent {
  const ShareNoteInvitedEvent(this.username);
  

 final  String username;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteInvitedEventCopyWith<ShareNoteInvitedEvent> get copyWith => _$ShareNoteInvitedEventCopyWithImpl<ShareNoteInvitedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteInvitedEvent&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'ShareNoteEvent.invited(username: $username)';
}


}

/// @nodoc
abstract mixin class $ShareNoteInvitedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteInvitedEventCopyWith(ShareNoteInvitedEvent value, $Res Function(ShareNoteInvitedEvent) _then) = _$ShareNoteInvitedEventCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$ShareNoteInvitedEventCopyWithImpl<$Res>
    implements $ShareNoteInvitedEventCopyWith<$Res> {
  _$ShareNoteInvitedEventCopyWithImpl(this._self, this._then);

  final ShareNoteInvitedEvent _self;
  final $Res Function(ShareNoteInvitedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(ShareNoteInvitedEvent(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteFingerprintConfirmedEvent implements ShareNoteEvent {
  const ShareNoteFingerprintConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteFingerprintConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.fingerprintConfirmed()';
}


}




/// @nodoc


class ShareNoteInvitedUnverifiedEvent implements ShareNoteEvent {
  const ShareNoteInvitedUnverifiedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteInvitedUnverifiedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.invitedUnverified()';
}


}




/// @nodoc


class ShareNoteNewKeyAcceptedEvent implements ShareNoteEvent {
  const ShareNoteNewKeyAcceptedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteNewKeyAcceptedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.newKeyAccepted()';
}


}




/// @nodoc


class ShareNotePromptDismissedEvent implements ShareNoteEvent {
  const ShareNotePromptDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNotePromptDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.promptDismissed()';
}


}




/// @nodoc


class ShareNoteKeyReviewedEvent implements ShareNoteEvent {
  const ShareNoteKeyReviewedEvent(this.username);
  

 final  String username;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteKeyReviewedEventCopyWith<ShareNoteKeyReviewedEvent> get copyWith => _$ShareNoteKeyReviewedEventCopyWithImpl<ShareNoteKeyReviewedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteKeyReviewedEvent&&(identical(other.username, username) || other.username == username));
}


@override
int get hashCode => Object.hash(runtimeType,username);

@override
String toString() {
  return 'ShareNoteEvent.keyReviewed(username: $username)';
}


}

/// @nodoc
abstract mixin class $ShareNoteKeyReviewedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteKeyReviewedEventCopyWith(ShareNoteKeyReviewedEvent value, $Res Function(ShareNoteKeyReviewedEvent) _then) = _$ShareNoteKeyReviewedEventCopyWithImpl;
@useResult
$Res call({
 String username
});




}
/// @nodoc
class _$ShareNoteKeyReviewedEventCopyWithImpl<$Res>
    implements $ShareNoteKeyReviewedEventCopyWith<$Res> {
  _$ShareNoteKeyReviewedEventCopyWithImpl(this._self, this._then);

  final ShareNoteKeyReviewedEvent _self;
  final $Res Function(ShareNoteKeyReviewedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,}) {
  return _then(ShareNoteKeyReviewedEvent(
null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteMemberRoleChangedEvent implements ShareNoteEvent {
  const ShareNoteMemberRoleChangedEvent(this.accountId, this.role);
  

 final  String accountId;
 final  ShareRole role;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteMemberRoleChangedEventCopyWith<ShareNoteMemberRoleChangedEvent> get copyWith => _$ShareNoteMemberRoleChangedEventCopyWithImpl<ShareNoteMemberRoleChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteMemberRoleChangedEvent&&(identical(other.accountId, accountId) || other.accountId == accountId)&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,accountId,role);

@override
String toString() {
  return 'ShareNoteEvent.memberRoleChanged(accountId: $accountId, role: $role)';
}


}

/// @nodoc
abstract mixin class $ShareNoteMemberRoleChangedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteMemberRoleChangedEventCopyWith(ShareNoteMemberRoleChangedEvent value, $Res Function(ShareNoteMemberRoleChangedEvent) _then) = _$ShareNoteMemberRoleChangedEventCopyWithImpl;
@useResult
$Res call({
 String accountId, ShareRole role
});




}
/// @nodoc
class _$ShareNoteMemberRoleChangedEventCopyWithImpl<$Res>
    implements $ShareNoteMemberRoleChangedEventCopyWith<$Res> {
  _$ShareNoteMemberRoleChangedEventCopyWithImpl(this._self, this._then);

  final ShareNoteMemberRoleChangedEvent _self;
  final $Res Function(ShareNoteMemberRoleChangedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? accountId = null,Object? role = null,}) {
  return _then(ShareNoteMemberRoleChangedEvent(
null == accountId ? _self.accountId : accountId // ignore: cast_nullable_to_non_nullable
as String,null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole,
  ));
}


}

/// @nodoc


class ShareNoteMemberRemovedEvent implements ShareNoteEvent {
  const ShareNoteMemberRemovedEvent(this.accountId);
  

 final  String accountId;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteMemberRemovedEventCopyWith<ShareNoteMemberRemovedEvent> get copyWith => _$ShareNoteMemberRemovedEventCopyWithImpl<ShareNoteMemberRemovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteMemberRemovedEvent&&(identical(other.accountId, accountId) || other.accountId == accountId));
}


@override
int get hashCode => Object.hash(runtimeType,accountId);

@override
String toString() {
  return 'ShareNoteEvent.memberRemoved(accountId: $accountId)';
}


}

/// @nodoc
abstract mixin class $ShareNoteMemberRemovedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteMemberRemovedEventCopyWith(ShareNoteMemberRemovedEvent value, $Res Function(ShareNoteMemberRemovedEvent) _then) = _$ShareNoteMemberRemovedEventCopyWithImpl;
@useResult
$Res call({
 String accountId
});




}
/// @nodoc
class _$ShareNoteMemberRemovedEventCopyWithImpl<$Res>
    implements $ShareNoteMemberRemovedEventCopyWith<$Res> {
  _$ShareNoteMemberRemovedEventCopyWithImpl(this._self, this._then);

  final ShareNoteMemberRemovedEvent _self;
  final $Res Function(ShareNoteMemberRemovedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? accountId = null,}) {
  return _then(ShareNoteMemberRemovedEvent(
null == accountId ? _self.accountId : accountId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteRequestApprovedEvent implements ShareNoteEvent {
  const ShareNoteRequestApprovedEvent(this.accountId);
  

 final  String accountId;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteRequestApprovedEventCopyWith<ShareNoteRequestApprovedEvent> get copyWith => _$ShareNoteRequestApprovedEventCopyWithImpl<ShareNoteRequestApprovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteRequestApprovedEvent&&(identical(other.accountId, accountId) || other.accountId == accountId));
}


@override
int get hashCode => Object.hash(runtimeType,accountId);

@override
String toString() {
  return 'ShareNoteEvent.requestApproved(accountId: $accountId)';
}


}

/// @nodoc
abstract mixin class $ShareNoteRequestApprovedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteRequestApprovedEventCopyWith(ShareNoteRequestApprovedEvent value, $Res Function(ShareNoteRequestApprovedEvent) _then) = _$ShareNoteRequestApprovedEventCopyWithImpl;
@useResult
$Res call({
 String accountId
});




}
/// @nodoc
class _$ShareNoteRequestApprovedEventCopyWithImpl<$Res>
    implements $ShareNoteRequestApprovedEventCopyWith<$Res> {
  _$ShareNoteRequestApprovedEventCopyWithImpl(this._self, this._then);

  final ShareNoteRequestApprovedEvent _self;
  final $Res Function(ShareNoteRequestApprovedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? accountId = null,}) {
  return _then(ShareNoteRequestApprovedEvent(
null == accountId ? _self.accountId : accountId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteRequestDeclinedEvent implements ShareNoteEvent {
  const ShareNoteRequestDeclinedEvent(this.accountId);
  

 final  String accountId;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteRequestDeclinedEventCopyWith<ShareNoteRequestDeclinedEvent> get copyWith => _$ShareNoteRequestDeclinedEventCopyWithImpl<ShareNoteRequestDeclinedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteRequestDeclinedEvent&&(identical(other.accountId, accountId) || other.accountId == accountId));
}


@override
int get hashCode => Object.hash(runtimeType,accountId);

@override
String toString() {
  return 'ShareNoteEvent.requestDeclined(accountId: $accountId)';
}


}

/// @nodoc
abstract mixin class $ShareNoteRequestDeclinedEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteRequestDeclinedEventCopyWith(ShareNoteRequestDeclinedEvent value, $Res Function(ShareNoteRequestDeclinedEvent) _then) = _$ShareNoteRequestDeclinedEventCopyWithImpl;
@useResult
$Res call({
 String accountId
});




}
/// @nodoc
class _$ShareNoteRequestDeclinedEventCopyWithImpl<$Res>
    implements $ShareNoteRequestDeclinedEventCopyWith<$Res> {
  _$ShareNoteRequestDeclinedEventCopyWithImpl(this._self, this._then);

  final ShareNoteRequestDeclinedEvent _self;
  final $Res Function(ShareNoteRequestDeclinedEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? accountId = null,}) {
  return _then(ShareNoteRequestDeclinedEvent(
null == accountId ? _self.accountId : accountId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteLinkRoleChosenEvent implements ShareNoteEvent {
  const ShareNoteLinkRoleChosenEvent(this.role);
  

 final  ShareRole role;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteLinkRoleChosenEventCopyWith<ShareNoteLinkRoleChosenEvent> get copyWith => _$ShareNoteLinkRoleChosenEventCopyWithImpl<ShareNoteLinkRoleChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteLinkRoleChosenEvent&&(identical(other.role, role) || other.role == role));
}


@override
int get hashCode => Object.hash(runtimeType,role);

@override
String toString() {
  return 'ShareNoteEvent.linkRoleChosen(role: $role)';
}


}

/// @nodoc
abstract mixin class $ShareNoteLinkRoleChosenEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteLinkRoleChosenEventCopyWith(ShareNoteLinkRoleChosenEvent value, $Res Function(ShareNoteLinkRoleChosenEvent) _then) = _$ShareNoteLinkRoleChosenEventCopyWithImpl;
@useResult
$Res call({
 ShareRole role
});




}
/// @nodoc
class _$ShareNoteLinkRoleChosenEventCopyWithImpl<$Res>
    implements $ShareNoteLinkRoleChosenEventCopyWith<$Res> {
  _$ShareNoteLinkRoleChosenEventCopyWithImpl(this._self, this._then);

  final ShareNoteLinkRoleChosenEvent _self;
  final $Res Function(ShareNoteLinkRoleChosenEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? role = null,}) {
  return _then(ShareNoteLinkRoleChosenEvent(
null == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole,
  ));
}


}

/// @nodoc


class ShareNoteLinkDaysChosenEvent implements ShareNoteEvent {
  const ShareNoteLinkDaysChosenEvent(this.days);
  

 final  int days;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteLinkDaysChosenEventCopyWith<ShareNoteLinkDaysChosenEvent> get copyWith => _$ShareNoteLinkDaysChosenEventCopyWithImpl<ShareNoteLinkDaysChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteLinkDaysChosenEvent&&(identical(other.days, days) || other.days == days));
}


@override
int get hashCode => Object.hash(runtimeType,days);

@override
String toString() {
  return 'ShareNoteEvent.linkDaysChosen(days: $days)';
}


}

/// @nodoc
abstract mixin class $ShareNoteLinkDaysChosenEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteLinkDaysChosenEventCopyWith(ShareNoteLinkDaysChosenEvent value, $Res Function(ShareNoteLinkDaysChosenEvent) _then) = _$ShareNoteLinkDaysChosenEventCopyWithImpl;
@useResult
$Res call({
 int days
});




}
/// @nodoc
class _$ShareNoteLinkDaysChosenEventCopyWithImpl<$Res>
    implements $ShareNoteLinkDaysChosenEventCopyWith<$Res> {
  _$ShareNoteLinkDaysChosenEventCopyWithImpl(this._self, this._then);

  final ShareNoteLinkDaysChosenEvent _self;
  final $Res Function(ShareNoteLinkDaysChosenEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? days = null,}) {
  return _then(ShareNoteLinkDaysChosenEvent(
null == days ? _self.days : days // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class ShareNoteLinkMadeEvent implements ShareNoteEvent {
  const ShareNoteLinkMadeEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteLinkMadeEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.linkMade()';
}


}




/// @nodoc


class ShareNoteLinkCancelledEvent implements ShareNoteEvent {
  const ShareNoteLinkCancelledEvent(this.linkId);
  

 final  String linkId;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteLinkCancelledEventCopyWith<ShareNoteLinkCancelledEvent> get copyWith => _$ShareNoteLinkCancelledEventCopyWithImpl<ShareNoteLinkCancelledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteLinkCancelledEvent&&(identical(other.linkId, linkId) || other.linkId == linkId));
}


@override
int get hashCode => Object.hash(runtimeType,linkId);

@override
String toString() {
  return 'ShareNoteEvent.linkCancelled(linkId: $linkId)';
}


}

/// @nodoc
abstract mixin class $ShareNoteLinkCancelledEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteLinkCancelledEventCopyWith(ShareNoteLinkCancelledEvent value, $Res Function(ShareNoteLinkCancelledEvent) _then) = _$ShareNoteLinkCancelledEventCopyWithImpl;
@useResult
$Res call({
 String linkId
});




}
/// @nodoc
class _$ShareNoteLinkCancelledEventCopyWithImpl<$Res>
    implements $ShareNoteLinkCancelledEventCopyWith<$Res> {
  _$ShareNoteLinkCancelledEventCopyWithImpl(this._self, this._then);

  final ShareNoteLinkCancelledEvent _self;
  final $Res Function(ShareNoteLinkCancelledEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? linkId = null,}) {
  return _then(ShareNoteLinkCancelledEvent(
null == linkId ? _self.linkId : linkId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ShareNoteLinkCopiedEvent implements ShareNoteEvent {
  const ShareNoteLinkCopiedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteLinkCopiedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.linkCopied()';
}


}




/// @nodoc


class ShareNoteLinkSentEvent implements ShareNoteEvent {
  const ShareNoteLinkSentEvent({this.origin});
  

 final  Rect? origin;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteLinkSentEventCopyWith<ShareNoteLinkSentEvent> get copyWith => _$ShareNoteLinkSentEventCopyWithImpl<ShareNoteLinkSentEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteLinkSentEvent&&(identical(other.origin, origin) || other.origin == origin));
}


@override
int get hashCode => Object.hash(runtimeType,origin);

@override
String toString() {
  return 'ShareNoteEvent.linkSent(origin: $origin)';
}


}

/// @nodoc
abstract mixin class $ShareNoteLinkSentEventCopyWith<$Res> implements $ShareNoteEventCopyWith<$Res> {
  factory $ShareNoteLinkSentEventCopyWith(ShareNoteLinkSentEvent value, $Res Function(ShareNoteLinkSentEvent) _then) = _$ShareNoteLinkSentEventCopyWithImpl;
@useResult
$Res call({
 Rect? origin
});




}
/// @nodoc
class _$ShareNoteLinkSentEventCopyWithImpl<$Res>
    implements $ShareNoteLinkSentEventCopyWith<$Res> {
  _$ShareNoteLinkSentEventCopyWithImpl(this._self, this._then);

  final ShareNoteLinkSentEvent _self;
  final $Res Function(ShareNoteLinkSentEvent) _then;

/// Create a copy of ShareNoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? origin = freezed,}) {
  return _then(ShareNoteLinkSentEvent(
origin: freezed == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as Rect?,
  ));
}


}

/// @nodoc


class ShareNoteEndAskedEvent implements ShareNoteEvent {
  const ShareNoteEndAskedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteEndAskedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.endAsked()';
}


}




/// @nodoc


class ShareNoteEndDismissedEvent implements ShareNoteEvent {
  const ShareNoteEndDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteEndDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.endDismissed()';
}


}




/// @nodoc


class ShareNoteEndConfirmedEvent implements ShareNoteEvent {
  const ShareNoteEndConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteEndConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ShareNoteEvent.endConfirmed()';
}


}




/// @nodoc
mixin _$ShareNoteState {

/// The note the sheet is for.
 String get noteId; ShareSheetStatus get status;/// Why the note can't be shared, in the library's words, while the
/// status is `notShareable`.
 String? get reason;/// The note's sharing; null while it is not shared.
 ShareInfo? get info;/// A step is running.
 bool get busy;/// A question before someone's keys are trusted.
 SharePrompt? get prompt;/// The address of the link just made. It is known only to the call that
/// made it, so it shows until the sheet closes.
 String? get madeLink;/// The link was copied.
 bool get linkCopied;/// Why the last step did not happen.
 ShareProblem? get problem;/// What the last step did.
 ShareDone? get done;/// Stop sharing (or Leave) was pressed once; the next press does it.
 bool get confirmEnd;/// The role the next invitation gives.
 ShareRole get inviteRole;/// The role and the days of the next link.
 ShareRole get linkRole; int get linkDays;
/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ShareNoteStateCopyWith<ShareNoteState> get copyWith => _$ShareNoteStateCopyWithImpl<ShareNoteState>(this as ShareNoteState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ShareNoteState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.info, info) || other.info == info)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.prompt, prompt) || other.prompt == prompt)&&(identical(other.madeLink, madeLink) || other.madeLink == madeLink)&&(identical(other.linkCopied, linkCopied) || other.linkCopied == linkCopied)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.done, done) || other.done == done)&&(identical(other.confirmEnd, confirmEnd) || other.confirmEnd == confirmEnd)&&(identical(other.inviteRole, inviteRole) || other.inviteRole == inviteRole)&&(identical(other.linkRole, linkRole) || other.linkRole == linkRole)&&(identical(other.linkDays, linkDays) || other.linkDays == linkDays));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,reason,info,busy,prompt,madeLink,linkCopied,problem,done,confirmEnd,inviteRole,linkRole,linkDays);

@override
String toString() {
  return 'ShareNoteState(noteId: $noteId, status: $status, reason: $reason, info: $info, busy: $busy, prompt: $prompt, madeLink: $madeLink, linkCopied: $linkCopied, problem: $problem, done: $done, confirmEnd: $confirmEnd, inviteRole: $inviteRole, linkRole: $linkRole, linkDays: $linkDays)';
}


}

/// @nodoc
abstract mixin class $ShareNoteStateCopyWith<$Res>  {
  factory $ShareNoteStateCopyWith(ShareNoteState value, $Res Function(ShareNoteState) _then) = _$ShareNoteStateCopyWithImpl;
@useResult
$Res call({
 String noteId, ShareSheetStatus status, String? reason, ShareInfo? info, bool busy, SharePrompt? prompt, String? madeLink, bool linkCopied, ShareProblem? problem, ShareDone? done, bool confirmEnd, ShareRole inviteRole, ShareRole linkRole, int linkDays
});


$SharePromptCopyWith<$Res>? get prompt;$ShareProblemCopyWith<$Res>? get problem;$ShareDoneCopyWith<$Res>? get done;

}
/// @nodoc
class _$ShareNoteStateCopyWithImpl<$Res>
    implements $ShareNoteStateCopyWith<$Res> {
  _$ShareNoteStateCopyWithImpl(this._self, this._then);

  final ShareNoteState _self;
  final $Res Function(ShareNoteState) _then;

/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? status = null,Object? reason = freezed,Object? info = freezed,Object? busy = null,Object? prompt = freezed,Object? madeLink = freezed,Object? linkCopied = null,Object? problem = freezed,Object? done = freezed,Object? confirmEnd = null,Object? inviteRole = null,Object? linkRole = null,Object? linkDays = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as ShareSheetStatus,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as ShareInfo?,busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,prompt: freezed == prompt ? _self.prompt : prompt // ignore: cast_nullable_to_non_nullable
as SharePrompt?,madeLink: freezed == madeLink ? _self.madeLink : madeLink // ignore: cast_nullable_to_non_nullable
as String?,linkCopied: null == linkCopied ? _self.linkCopied : linkCopied // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,done: freezed == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as ShareDone?,confirmEnd: null == confirmEnd ? _self.confirmEnd : confirmEnd // ignore: cast_nullable_to_non_nullable
as bool,inviteRole: null == inviteRole ? _self.inviteRole : inviteRole // ignore: cast_nullable_to_non_nullable
as ShareRole,linkRole: null == linkRole ? _self.linkRole : linkRole // ignore: cast_nullable_to_non_nullable
as ShareRole,linkDays: null == linkDays ? _self.linkDays : linkDays // ignore: cast_nullable_to_non_nullable
as int,
  ));
}
/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$SharePromptCopyWith<$Res>? get prompt {
    if (_self.prompt == null) {
    return null;
  }

  return $SharePromptCopyWith<$Res>(_self.prompt!, (value) {
    return _then(_self.copyWith(prompt: value));
  });
}/// Create a copy of ShareNoteState
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
}/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ShareDoneCopyWith<$Res>? get done {
    if (_self.done == null) {
    return null;
  }

  return $ShareDoneCopyWith<$Res>(_self.done!, (value) {
    return _then(_self.copyWith(done: value));
  });
}
}


/// Adds pattern-matching-related methods to [ShareNoteState].
extension ShareNoteStatePatterns on ShareNoteState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ShareNoteState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ShareNoteState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ShareNoteState value)  $default,){
final _that = this;
switch (_that) {
case _ShareNoteState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ShareNoteState value)?  $default,){
final _that = this;
switch (_that) {
case _ShareNoteState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  ShareSheetStatus status,  String? reason,  ShareInfo? info,  bool busy,  SharePrompt? prompt,  String? madeLink,  bool linkCopied,  ShareProblem? problem,  ShareDone? done,  bool confirmEnd,  ShareRole inviteRole,  ShareRole linkRole,  int linkDays)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ShareNoteState() when $default != null:
return $default(_that.noteId,_that.status,_that.reason,_that.info,_that.busy,_that.prompt,_that.madeLink,_that.linkCopied,_that.problem,_that.done,_that.confirmEnd,_that.inviteRole,_that.linkRole,_that.linkDays);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  ShareSheetStatus status,  String? reason,  ShareInfo? info,  bool busy,  SharePrompt? prompt,  String? madeLink,  bool linkCopied,  ShareProblem? problem,  ShareDone? done,  bool confirmEnd,  ShareRole inviteRole,  ShareRole linkRole,  int linkDays)  $default,) {final _that = this;
switch (_that) {
case _ShareNoteState():
return $default(_that.noteId,_that.status,_that.reason,_that.info,_that.busy,_that.prompt,_that.madeLink,_that.linkCopied,_that.problem,_that.done,_that.confirmEnd,_that.inviteRole,_that.linkRole,_that.linkDays);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  ShareSheetStatus status,  String? reason,  ShareInfo? info,  bool busy,  SharePrompt? prompt,  String? madeLink,  bool linkCopied,  ShareProblem? problem,  ShareDone? done,  bool confirmEnd,  ShareRole inviteRole,  ShareRole linkRole,  int linkDays)?  $default,) {final _that = this;
switch (_that) {
case _ShareNoteState() when $default != null:
return $default(_that.noteId,_that.status,_that.reason,_that.info,_that.busy,_that.prompt,_that.madeLink,_that.linkCopied,_that.problem,_that.done,_that.confirmEnd,_that.inviteRole,_that.linkRole,_that.linkDays);case _:
  return null;

}
}

}

/// @nodoc


class _ShareNoteState extends ShareNoteState {
  const _ShareNoteState({this.noteId = '', this.status = ShareSheetStatus.loading, this.reason, this.info, this.busy = false, this.prompt, this.madeLink, this.linkCopied = false, this.problem, this.done, this.confirmEnd = false, this.inviteRole = ShareRole.editor, this.linkRole = ShareRole.commenter, this.linkDays = 7}): super._();
  

/// The note the sheet is for.
@override@JsonKey() final  String noteId;
@override@JsonKey() final  ShareSheetStatus status;
/// Why the note can't be shared, in the library's words, while the
/// status is `notShareable`.
@override final  String? reason;
/// The note's sharing; null while it is not shared.
@override final  ShareInfo? info;
/// A step is running.
@override@JsonKey() final  bool busy;
/// A question before someone's keys are trusted.
@override final  SharePrompt? prompt;
/// The address of the link just made. It is known only to the call that
/// made it, so it shows until the sheet closes.
@override final  String? madeLink;
/// The link was copied.
@override@JsonKey() final  bool linkCopied;
/// Why the last step did not happen.
@override final  ShareProblem? problem;
/// What the last step did.
@override final  ShareDone? done;
/// Stop sharing (or Leave) was pressed once; the next press does it.
@override@JsonKey() final  bool confirmEnd;
/// The role the next invitation gives.
@override@JsonKey() final  ShareRole inviteRole;
/// The role and the days of the next link.
@override@JsonKey() final  ShareRole linkRole;
@override@JsonKey() final  int linkDays;

/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ShareNoteStateCopyWith<_ShareNoteState> get copyWith => __$ShareNoteStateCopyWithImpl<_ShareNoteState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ShareNoteState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.reason, reason) || other.reason == reason)&&(identical(other.info, info) || other.info == info)&&(identical(other.busy, busy) || other.busy == busy)&&(identical(other.prompt, prompt) || other.prompt == prompt)&&(identical(other.madeLink, madeLink) || other.madeLink == madeLink)&&(identical(other.linkCopied, linkCopied) || other.linkCopied == linkCopied)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.done, done) || other.done == done)&&(identical(other.confirmEnd, confirmEnd) || other.confirmEnd == confirmEnd)&&(identical(other.inviteRole, inviteRole) || other.inviteRole == inviteRole)&&(identical(other.linkRole, linkRole) || other.linkRole == linkRole)&&(identical(other.linkDays, linkDays) || other.linkDays == linkDays));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,reason,info,busy,prompt,madeLink,linkCopied,problem,done,confirmEnd,inviteRole,linkRole,linkDays);

@override
String toString() {
  return 'ShareNoteState(noteId: $noteId, status: $status, reason: $reason, info: $info, busy: $busy, prompt: $prompt, madeLink: $madeLink, linkCopied: $linkCopied, problem: $problem, done: $done, confirmEnd: $confirmEnd, inviteRole: $inviteRole, linkRole: $linkRole, linkDays: $linkDays)';
}


}

/// @nodoc
abstract mixin class _$ShareNoteStateCopyWith<$Res> implements $ShareNoteStateCopyWith<$Res> {
  factory _$ShareNoteStateCopyWith(_ShareNoteState value, $Res Function(_ShareNoteState) _then) = __$ShareNoteStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, ShareSheetStatus status, String? reason, ShareInfo? info, bool busy, SharePrompt? prompt, String? madeLink, bool linkCopied, ShareProblem? problem, ShareDone? done, bool confirmEnd, ShareRole inviteRole, ShareRole linkRole, int linkDays
});


@override $SharePromptCopyWith<$Res>? get prompt;@override $ShareProblemCopyWith<$Res>? get problem;@override $ShareDoneCopyWith<$Res>? get done;

}
/// @nodoc
class __$ShareNoteStateCopyWithImpl<$Res>
    implements _$ShareNoteStateCopyWith<$Res> {
  __$ShareNoteStateCopyWithImpl(this._self, this._then);

  final _ShareNoteState _self;
  final $Res Function(_ShareNoteState) _then;

/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? status = null,Object? reason = freezed,Object? info = freezed,Object? busy = null,Object? prompt = freezed,Object? madeLink = freezed,Object? linkCopied = null,Object? problem = freezed,Object? done = freezed,Object? confirmEnd = null,Object? inviteRole = null,Object? linkRole = null,Object? linkDays = null,}) {
  return _then(_ShareNoteState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as ShareSheetStatus,reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as ShareInfo?,busy: null == busy ? _self.busy : busy // ignore: cast_nullable_to_non_nullable
as bool,prompt: freezed == prompt ? _self.prompt : prompt // ignore: cast_nullable_to_non_nullable
as SharePrompt?,madeLink: freezed == madeLink ? _self.madeLink : madeLink // ignore: cast_nullable_to_non_nullable
as String?,linkCopied: null == linkCopied ? _self.linkCopied : linkCopied // ignore: cast_nullable_to_non_nullable
as bool,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,done: freezed == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as ShareDone?,confirmEnd: null == confirmEnd ? _self.confirmEnd : confirmEnd // ignore: cast_nullable_to_non_nullable
as bool,inviteRole: null == inviteRole ? _self.inviteRole : inviteRole // ignore: cast_nullable_to_non_nullable
as ShareRole,linkRole: null == linkRole ? _self.linkRole : linkRole // ignore: cast_nullable_to_non_nullable
as ShareRole,linkDays: null == linkDays ? _self.linkDays : linkDays // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$SharePromptCopyWith<$Res>? get prompt {
    if (_self.prompt == null) {
    return null;
  }

  return $SharePromptCopyWith<$Res>(_self.prompt!, (value) {
    return _then(_self.copyWith(prompt: value));
  });
}/// Create a copy of ShareNoteState
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
}/// Create a copy of ShareNoteState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ShareDoneCopyWith<$Res>? get done {
    if (_self.done == null) {
    return null;
  }

  return $ShareDoneCopyWith<$Res>(_self.done!, (value) {
    return _then(_self.copyWith(done: value));
  });
}
}

// dart format on
