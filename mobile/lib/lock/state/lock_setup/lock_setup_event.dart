// Kanso — lock_setup_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_setup_bloc.dart';

@freezed
sealed class LockSetupEvent with _$LockSetupEvent {
  /// The sheet that locks [target] opened: find out which way it goes.
  const factory LockSetupEvent.started(LockTarget target) =
      LockSetupStartedEvent;

  /// The person typed [password] (and [again], when it is a new one).
  const factory LockSetupEvent.submitted(String password, String again) =
      LockSetupSubmittedEvent;

  /// The person confirmed locking with the notes open.
  const factory LockSetupEvent.confirmed() = LockSetupConfirmedEvent;
}
