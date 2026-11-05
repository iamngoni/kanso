// Kanso — lock_remove_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_remove_bloc.dart';

@freezed
sealed class LockRemoveEvent with _$LockRemoveEvent {
  /// The person confirmed taking the lock off [target].
  const factory LockRemoveEvent.confirmed(LockTarget target) =
      LockRemoveConfirmedEvent;

  /// The person pressed Cancel while the files are being opened.
  const factory LockRemoveEvent.cancelled() = LockRemoveCancelledEvent;

  /// Unsealing the files got as far as [progress].
  const factory LockRemoveEvent.progressed(SealProgress progress) =
      LockRemoveProgressedEvent;
}
