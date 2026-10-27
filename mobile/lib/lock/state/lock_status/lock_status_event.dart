// Kanso — lock_status_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_status_bloc.dart';

@freezed
sealed class LockStatusEvent with _$LockStatusEvent {
  /// Read the status, and keep it read as the notes open and close.
  const factory LockStatusEvent.started() = LockStatusStartedEvent;

  /// Read the status again.
  const factory LockStatusEvent.refreshed() = LockStatusRefreshedEvent;

  /// The person asked to lock the notes now.
  const factory LockStatusEvent.lockedNow() = LockStatusLockedNowEvent;

  /// The app moved to [lifecycle].
  const factory LockStatusEvent.lifecycleChanged(AppLifecycleState lifecycle) =
      LockStatusLifecycleChangedEvent;
}
