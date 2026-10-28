// Kanso — ai_settings_change.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ai_settings_change.freezed.dart';

/// What the person set in Settings › AI, to be saved. An empty address,
/// model or region means the provider's own.
@freezed
abstract class AiSettingsChange with _$AiSettingsChange {
  const factory AiSettingsChange({
    required bool enabled,
    required String provider,
    @Default('') String baseUrl,
    @Default('') String model,
    @Default('') String region,
  }) = _AiSettingsChange;
}
