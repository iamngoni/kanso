// Kanso — biometrics_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'biometrics_bloc.dart';

@freezed
abstract class BiometricsState with _$BiometricsState {
  const factory BiometricsState({
    @Default(BiometricsStatus.idle) BiometricsStatus status,
    ApplicationError? error,
  }) = _BiometricsState;
}
