// Kanso — account_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'account_bloc.dart';

@freezed
sealed class AccountEvent with _$AccountEvent {
  /// The app started: follow the account.
  const factory AccountEvent.started() = AccountStartedEvent;
}
