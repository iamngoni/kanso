// Kanso — device_approval_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'device_approval_bloc.dart';

@freezed
sealed class DeviceApprovalEvent with _$DeviceApprovalEvent {
  /// Whether an account is signed in, or whether the app is in front, changed:
  /// requests are watched only while both are so.
  const factory DeviceApprovalEvent.conditionsChanged({
    required bool signedIn,
    required bool foreground,
  }) = DeviceApprovalConditionsChangedEvent;

  /// Approve was pressed for the request [id].
  const factory DeviceApprovalEvent.approved(String id) =
      DeviceApprovalApprovedEvent;

  /// Don't approve was pressed for the request [id].
  const factory DeviceApprovalEvent.declined(String id) =
      DeviceApprovalDeclinedEvent;
}
