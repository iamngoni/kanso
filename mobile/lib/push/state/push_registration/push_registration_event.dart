// Kanso — push_registration_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'push_registration_bloc.dart';

@freezed
sealed class PushRegistrationEvent with _$PushRegistrationEvent {
  /// The app started: ask the OS for a token and follow the ones it issues.
  const factory PushRegistrationEvent.started() = PushRegistrationStartedEvent;

  /// The OS issued [token].
  const factory PushRegistrationEvent.tokenReceived(PushToken token) =
      PushRegistrationTokenReceivedEvent;

  /// The app came back to the front.
  const factory PushRegistrationEvent.resumed() = PushRegistrationResumedEvent;
}
