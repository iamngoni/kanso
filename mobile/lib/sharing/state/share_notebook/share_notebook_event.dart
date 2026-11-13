// Kanso — share_notebook_event.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'share_notebook_bloc.dart';

@freezed
sealed class ShareNotebookEvent with _$ShareNotebookEvent {
  /// The sheet opened for the notebook [notebookId].
  const factory ShareNotebookEvent.started(String notebookId) =
      ShareNotebookStartedEvent;

  /// Read the notebook's sharing again, after it failed.
  const factory ShareNotebookEvent.retried() = ShareNotebookRetriedEvent;

  /// The role the next person is invited with.
  const factory ShareNotebookEvent.inviteRoleChosen(BookRole role) =
      ShareNotebookInviteRoleChosenEvent;

  /// Invite [username]: the first one shares the notebook.
  const factory ShareNotebookEvent.invited(String username) =
      ShareNotebookInvitedEvent;

  /// The fingerprints matched: mark the person verified, then invite them.
  const factory ShareNotebookEvent.fingerprintConfirmed() =
      ShareNotebookFingerprintConfirmedEvent;

  /// Invite them without comparing.
  const factory ShareNotebookEvent.invitedUnverified() =
      ShareNotebookInvitedUnverifiedEvent;

  /// Trust the new keys of someone whose keys changed.
  const factory ShareNotebookEvent.newKeyAccepted() =
      ShareNotebookNewKeyAcceptedEvent;

  /// The question was put away: nobody is invited.
  const factory ShareNotebookEvent.promptDismissed() =
      ShareNotebookPromptDismissedEvent;

  /// A member gets another role.
  const factory ShareNotebookEvent.memberRoleChanged(
    String accountId,
    BookRole role,
  ) = ShareNotebookMemberRoleChangedEvent;

  /// A member is removed (the person already said yes).
  const factory ShareNotebookEvent.memberRemoved(BookMemberRow member) =
      ShareNotebookMemberRemovedEvent;

  /// Leave or stop sharing (the person already said yes).
  const factory ShareNotebookEvent.ended() = ShareNotebookEndedEvent;

  /// Sharing the notebook got this far.
  const factory ShareNotebookEvent.progressed(BookShareProgress progress) =
      ShareNotebookProgressedEvent;
}
