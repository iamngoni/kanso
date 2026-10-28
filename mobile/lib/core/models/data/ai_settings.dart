// Kanso — ai_settings.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ai_settings.freezed.dart';

/// Where AI stands now: whether it is on, where answers come from and what
/// is already saved. The API key is never in it, only whether one is saved.
@freezed
abstract class AiSettings with _$AiSettings {
  const factory AiSettings({
    /// Whether the AI features are on. They are off until the person turns
    /// them on.
    @Default(false) bool enabled,

    /// The provider's key (see `AiProviderInfo.key`).
    @Default('') String provider,

    /// The address typed, empty to use the provider's default.
    @Default('') String baseUrl,

    /// The model typed, empty to use the provider's default.
    @Default('') String model,

    /// The region typed, for the providers that use one.
    @Default('') String region,

    /// Whether a key is saved for [provider]. The key itself is never read
    /// back.
    @Default(false) bool hasKey,

    /// The server a question goes to, as a name: `api.anthropic.com`.
    @Default('') String host,

    /// Whether [host] is somewhere other than this device or its own
    /// network (not a loopback address).
    @Default(false) bool isRemote,

    /// The plain line about where a question goes and what is never sent,
    /// in the words of the device ("this phone", "this tablet").
    @Default('') String privacy,
  }) = _AiSettings;
}
