// Kanso — ai_settings_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'ai_settings_bloc.dart';

/// The address, model and region as typed.
@freezed
abstract class AiFields with _$AiFields {
  const factory AiFields({
    @Default('') String address,
    @Default('') String model,
    @Default('') String region,
  }) = _AiFields;
}

@freezed
sealed class AiSettingsEvent with _$AiSettingsEvent {
  /// The page opened: read the settings and the providers.
  const factory AiSettingsEvent.started() = AiSettingsStartedEvent;

  /// The person turned AI on or off.
  const factory AiSettingsEvent.enabledChanged({required bool enabled}) =
      AiSettingsEnabledChangedEvent;

  /// The person chose the provider with this key.
  const factory AiSettingsEvent.providerChosen(String provider) =
      AiSettingsProviderChosenEvent;

  /// The person typed in the address, model or region: save them once they
  /// pause.
  const factory AiSettingsEvent.fieldsChanged(AiFields fields) =
      AiSettingsFieldsChangedEvent;

  /// The person saved a key for the chosen provider.
  const factory AiSettingsEvent.keySaved(AiKey key) = AiSettingsKeySavedEvent;

  /// The person removed the saved key.
  const factory AiSettingsEvent.keyRemoved() = AiSettingsKeyRemovedEvent;

  /// The person pressed Test: save what is typed, then ask the model.
  const factory AiSettingsEvent.testRequested(AiFields fields) =
      AiSettingsTestRequestedEvent;
}
