// Kanso — memory_voice_settings_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/voice_settings.dart';
import '../abs/voice_settings_repository.dart';

/// The settings kept in memory, for tests; the app's are the Rust core's
/// (`RustVoiceSettingsRepository`).
class MemoryVoiceSettingsRepository implements VoiceSettingsRepository {
  VoiceSettings current = const VoiceSettings();

  @override
  Future<Either<ApplicationError, VoiceSettings>> settings() async =>
      Right<ApplicationError, VoiceSettings>(current);

  @override
  Future<Either<ApplicationError, VoiceSettings>> save(
    VoiceSettings settings,
  ) async {
    current = settings;
    return Right<ApplicationError, VoiceSettings>(current);
  }
}
