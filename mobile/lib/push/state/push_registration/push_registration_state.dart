// Kanso — push_registration_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'push_registration_bloc.dart';

@freezed
abstract class PushRegistrationState with _$PushRegistrationState {
  const factory PushRegistrationState({
    /// How far the registration has got.
    @Default(PushRegistrationPhase.starting) PushRegistrationPhase phase,

    /// Why the last step failed; null when it worked. Nothing shows it: a
    /// push that can't be registered is retried, not reported.
    ApplicationError? error,
  }) = _PushRegistrationState;
}
