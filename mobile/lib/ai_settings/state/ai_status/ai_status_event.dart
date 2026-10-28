// Kanso — ai_status_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'ai_status_bloc.dart';

@freezed
sealed class AiStatusEvent with _$AiStatusEvent {
  /// The app opened: read the settings, and follow them.
  const factory AiStatusEvent.started() = AiStatusStartedEvent;

  /// The settings changed while the app was open.
  const factory AiStatusEvent.settingsChanged(AiSettings settings) =
      AiStatusSettingsChangedEvent;
}
