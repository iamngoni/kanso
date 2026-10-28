// Kanso — discovery_settings_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'discovery_settings_bloc.dart';

@freezed
abstract class DiscoverySettingsState with _$DiscoverySettingsState {
  const factory DiscoverySettingsState({
    @Default(false) bool relatedEnabled,
    RelatedState? related,
    @Default(false) bool relatedBusy,
    @Default(false) bool ocrEnabled,
    @Default(0) int ocrWaiting,
  }) = _DiscoverySettingsState;
}
