// Kanso — lock_remove_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_remove_bloc.dart';

@freezed
sealed class LockRemoveEvent with _$LockRemoveEvent {
  /// The person confirmed taking the lock off [target].
  const factory LockRemoveEvent.confirmed(LockTarget target) =
      LockRemoveConfirmedEvent;
}
