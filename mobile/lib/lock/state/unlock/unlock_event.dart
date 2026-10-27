// Kanso — unlock_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'unlock_bloc.dart';

@freezed
sealed class UnlockEvent with _$UnlockEvent {
  /// The list of the notebook [target] is shown: see whether it is open,
  /// and try a biometric when they are on. [reason] is the prompt's text.
  const factory UnlockEvent.gateOpened(LockTarget target, String reason) =
      UnlockGateOpenedEvent;

  /// The person chose to unlock with a biometric. [reason] is the prompt's
  /// text.
  const factory UnlockEvent.biometricRequested(String reason) =
      UnlockBiometricRequestedEvent;

  /// The person typed [password] for [target].
  const factory UnlockEvent.passwordSubmitted(
    LockTarget target,
    String password,
  ) = UnlockPasswordSubmittedEvent;
}
