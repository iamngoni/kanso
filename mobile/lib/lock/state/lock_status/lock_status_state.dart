// Kanso — lock_status_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'lock_status_bloc.dart';

@freezed
abstract class LockStatusState with _$LockStatusState {
  const factory LockStatusState({
    /// Whether the status was read.
    @Default(false) bool loaded,

    /// Whether any note or notebook is locked, here or on another device.
    @Default(false) bool hasLockPassword,

    /// Whether the lock password has opened the locked notes.
    @Default(false) bool unlocked,

    /// Whether biometrics are turned on for the locked notes on this phone.
    @Default(false) bool biometricsOn,

    /// What biometric unlock the phone offers.
    @Default(BiometricKind.none) BiometricKind kind,
  }) = _LockStatusState;

  const LockStatusState._();

  /// The same, as the library reports it.
  LockStatus get status => LockStatus(
    hasLockPassword: hasLockPassword,
    unlocked: unlocked,
    biometricsOn: biometricsOn,
  );

  /// Whether a biometric can open the notes now.
  bool get biometricsUsable => biometricsOn && biometricsAvailable(kind);

  /// Whether a biometric could be turned on.
  bool get biometricsOfferable => !biometricsOn && biometricsAvailable(kind);
}
