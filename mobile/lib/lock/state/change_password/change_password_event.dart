// Kanso — change_password_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'change_password_bloc.dart';

@freezed
sealed class ChangePasswordEvent with _$ChangePasswordEvent {
  /// The person typed the [current] lock password and the [next] one twice.
  const factory ChangePasswordEvent.submitted(
    String current,
    String next,
    String again,
  ) = ChangePasswordSubmittedEvent;
}
