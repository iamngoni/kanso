// Kanso — biometrics_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'biometrics_bloc.dart';

@freezed
sealed class BiometricsEvent with _$BiometricsEvent {
  /// The person turned biometric unlock on; [reason] is the prompt's text.
  const factory BiometricsEvent.turnedOn(String reason) =
      BiometricsTurnedOnEvent;

  /// The person turned biometric unlock off.
  const factory BiometricsEvent.turnedOff() = BiometricsTurnedOffEvent;
}
