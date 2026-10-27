// Kanso — share_note_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'share_note_bloc.dart';

@freezed
sealed class ShareNoteEvent with _$ShareNoteEvent {
  /// The sheet was opened for the note [noteId]: read its sharing.
  const factory ShareNoteEvent.started(String noteId) = ShareNoteStartedEvent;

  /// The person asked to read the sharing again.
  const factory ShareNoteEvent.retried() = ShareNoteRetriedEvent;

  /// The role the next invitation gives was chosen.
  const factory ShareNoteEvent.inviteRoleChosen(ShareRole role) =
      ShareNoteInviteRoleChosenEvent;

  /// The person pressed Invite with [username] typed: look them up, and
  /// invite them (or ask about their keys first).
  const factory ShareNoteEvent.invited(String username) = ShareNoteInvitedEvent;

  /// The fingerprint of the person in a verify prompt was compared and
  /// matches: mark it so, then invite them.
  const factory ShareNoteEvent.fingerprintConfirmed() =
      ShareNoteFingerprintConfirmedEvent;

  /// The person in a verify prompt is invited without their fingerprint
  /// being compared: it stays unverified.
  const factory ShareNoteEvent.invitedUnverified() =
      ShareNoteInvitedUnverifiedEvent;

  /// The person trusts the new keys of the person in a key-changed prompt,
  /// having compared the fingerprint: pin them, then, if they were being
  /// invited, ask for the new fingerprint to be compared as for someone
  /// new.
  const factory ShareNoteEvent.newKeyAccepted() = ShareNoteNewKeyAcceptedEvent;

  /// The prompt was left: nobody is invited, nothing is trusted.
  const factory ShareNoteEvent.promptDismissed() =
      ShareNotePromptDismissedEvent;

  /// The warning on the member [username] was pressed: look up the keys the
  /// server gives for them now.
  const factory ShareNoteEvent.keyReviewed(String username) =
      ShareNoteKeyReviewedEvent;

  /// A member was given another role.
  const factory ShareNoteEvent.memberRoleChanged(
    String accountId,
    ShareRole role,
  ) = ShareNoteMemberRoleChangedEvent;

  /// A member was removed, or an invitation cancelled.
  const factory ShareNoteEvent.memberRemoved(String accountId) =
      ShareNoteMemberRemovedEvent;

  /// Someone who asked through a link was let in.
  const factory ShareNoteEvent.requestApproved(String accountId) =
      ShareNoteRequestApprovedEvent;

  /// Someone who asked through a link was turned away.
  const factory ShareNoteEvent.requestDeclined(String accountId) =
      ShareNoteRequestDeclinedEvent;

  /// The role the next link gives was chosen.
  const factory ShareNoteEvent.linkRoleChosen(ShareRole role) =
      ShareNoteLinkRoleChosenEvent;

  /// How many days the next link lasts was chosen.
  const factory ShareNoteEvent.linkDaysChosen(int days) =
      ShareNoteLinkDaysChosenEvent;

  /// Make Link was pressed.
  const factory ShareNoteEvent.linkMade() = ShareNoteLinkMadeEvent;

  /// A link was cancelled.
  const factory ShareNoteEvent.linkCancelled(String linkId) =
      ShareNoteLinkCancelledEvent;

  /// Copy was pressed on the link just made.
  const factory ShareNoteEvent.linkCopied() = ShareNoteLinkCopiedEvent;

  /// Send was pressed on the link just made: offer it to the system's share
  /// sheet, which on an iPad points at [origin].
  const factory ShareNoteEvent.linkSent({Rect? origin}) =
      ShareNoteLinkSentEvent;

  /// Stop sharing (or Leave) was pressed: ask first.
  const factory ShareNoteEvent.endAsked() = ShareNoteEndAskedEvent;

  /// The question was answered no.
  const factory ShareNoteEvent.endDismissed() = ShareNoteEndDismissedEvent;

  /// The question was answered yes: the owner stops sharing, anyone else
  /// leaves.
  const factory ShareNoteEvent.endConfirmed() = ShareNoteEndConfirmedEvent;
}
