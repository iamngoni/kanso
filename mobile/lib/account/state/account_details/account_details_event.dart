// Kanso — account_details_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'account_details_bloc.dart';

@freezed
sealed class AccountDetailsEvent with _$AccountDetailsEvent {
  /// The page opened, or the person asked to try again: read the account.
  const factory AccountDetailsEvent.started() = AccountDetailsStartedEvent;

  /// Signing a device out was confirmed.
  const factory AccountDetailsEvent.deviceRevoked(String deviceId) =
      AccountDetailsDeviceRevokedEvent;

  /// Signing this phone out was confirmed.
  const factory AccountDetailsEvent.signedOut() = AccountDetailsSignedOutEvent;

  /// The change password form was submitted.
  const factory AccountDetailsEvent.passwordChanged({
    required String current,
    required String newPassword,
    required String repeat,
  }) = AccountDetailsPasswordChangedEvent;

  /// A new recovery key was asked for, with the current password.
  const factory AccountDetailsEvent.recoveryKeyRequested({
    required String currentPassword,
  }) = AccountDetailsRecoveryKeyRequestedEvent;

  /// Deleting the account was confirmed with its password and its name.
  const factory AccountDetailsEvent.accountDeleted({
    required String password,
    required String typedUsername,
  }) = AccountDetailsAccountDeletedEvent;

  /// Adding a passkey was confirmed with the current password.
  const factory AccountDetailsEvent.passkeyAdded({
    required String currentPassword,
  }) = AccountDetailsPasskeyAddedEvent;

  /// Removing the passkey [id] was confirmed with the current password.
  const factory AccountDetailsEvent.passkeyRemoved({
    required String id,
    required String currentPassword,
  }) = AccountDetailsPasskeyRemovedEvent;

  /// Linking [provider] was confirmed with the current password.
  const factory AccountDetailsEvent.providerLinked({
    required ProviderKind provider,
    required String currentPassword,
  }) = AccountDetailsProviderLinkedEvent;

  /// Cancel was pressed while a provider is being linked.
  const factory AccountDetailsEvent.providerLinkCancelled() =
      AccountDetailsProviderLinkCancelledEvent;

  /// Unlinking [provider] was confirmed with the current password.
  const factory AccountDetailsEvent.providerUnlinked({
    required ProviderKind provider,
    required String currentPassword,
  }) = AccountDetailsProviderUnlinkedEvent;

  /// The page took the new recovery key to show it.
  const factory AccountDetailsEvent.keyShown() = AccountDetailsKeyShownEvent;

  /// The message at the top of the page was put away.
  const factory AccountDetailsEvent.messageDismissed() =
      AccountDetailsMessageDismissedEvent;
}
