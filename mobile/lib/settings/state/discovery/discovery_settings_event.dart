// Kanso — discovery_settings_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'discovery_settings_bloc.dart';

@freezed
sealed class DiscoverySettingsEvent with _$DiscoverySettingsEvent {
  const factory DiscoverySettingsEvent.started() =
      DiscoverySettingsStartedEvent;
  const factory DiscoverySettingsEvent.relatedChanged({required bool enabled}) =
      RelatedSettingChangedEvent;
  const factory DiscoverySettingsEvent.relatedRetryRequested() =
      RelatedRetryRequestedEvent;
  const factory DiscoverySettingsEvent.ocrChanged({required bool enabled}) =
      OcrSettingChangedEvent;
  const factory DiscoverySettingsEvent.relatedStatusReceived(
    RelatedState status,
  ) = RelatedStatusReceivedEvent;
  const factory DiscoverySettingsEvent.ocrCountReceived(int count) =
      OcrCountReceivedEvent;
}
