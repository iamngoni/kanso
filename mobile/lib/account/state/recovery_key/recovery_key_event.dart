// Kanso — recovery_key_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'recovery_key_bloc.dart';

@freezed
sealed class RecoveryKeyEvent with _$RecoveryKeyEvent {
  /// The screen opened with the [key] the flow that made it holds, or, with
  /// none, to show the one the library kept.
  const factory RecoveryKeyEvent.started({RecoveryKeyText? key}) =
      RecoveryKeyStartedEvent;

  /// The box "I've saved my recovery key somewhere safe" was changed.
  const factory RecoveryKeyEvent.confirmedChanged({required bool confirmed}) =
      RecoveryKeyConfirmedChangedEvent;

  /// Done was pressed.
  const factory RecoveryKeyEvent.done() = RecoveryKeyDoneEvent;
}
