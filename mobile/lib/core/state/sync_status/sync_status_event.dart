// Kanso — sync_status_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'sync_status_bloc.dart';

@freezed
sealed class SyncStatusEvent with _$SyncStatusEvent {
  /// The app started: follow the sync status.
  const factory SyncStatusEvent.started() = SyncStatusStartedEvent;

  /// The person asked to sync now (a pull to refresh).
  const factory SyncStatusEvent.syncRequested() = SyncStatusSyncRequestedEvent;

  /// The app came in front ([foreground] true) or went behind: the library
  /// syncs in front and stops behind.
  const factory SyncStatusEvent.foregroundChanged({required bool foreground}) =
      SyncStatusForegroundChangedEvent;
}
