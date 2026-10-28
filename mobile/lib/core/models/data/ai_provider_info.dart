// Kanso — ai_provider_info.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ai_provider_info.freezed.dart';

/// A place answers can come from, as the core describes it: what the
/// provider is called, what it is for, and what it needs to be asked.
@freezed
abstract class AiProviderInfo with _$AiProviderInfo {
  const factory AiProviderInfo({
    /// The provider's stable name, what settings and keys are kept under:
    /// `anthropic`, `ollama`.
    required String key,

    /// What the picker calls it.
    required String label,

    /// One plain line under [label] in the picker.
    required String description,

    /// The address used when none is typed.
    required String defaultBaseUrl,

    /// The model used when none is typed.
    required String defaultModel,

    /// Whether the provider cannot answer without a key.
    required bool needsKey,

    /// Whether the provider takes a key at all (a local model takes one only
    /// if its server asks).
    required bool acceptsKey,

    /// Whether the provider is reached through a region (Bedrock).
    @Default(false) bool usesRegion,

    /// The region used when none is typed.
    @Default('') String defaultRegion,
  }) = _AiProviderInfo;
}
