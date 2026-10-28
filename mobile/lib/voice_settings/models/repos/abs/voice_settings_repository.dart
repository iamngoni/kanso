// Kanso — voice_settings_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/voice_settings.dart';

/// The person's choices for voice input, kept with the library's settings by
/// the Rust core (`voiceSettings` / `setVoiceSettings`).
abstract interface class VoiceSettingsRepository {
  Future<Either<ApplicationError, VoiceSettings>> settings();

  /// Keeps [settings]; the settings now.
  Future<Either<ApplicationError, VoiceSettings>> save(VoiceSettings settings);
}
