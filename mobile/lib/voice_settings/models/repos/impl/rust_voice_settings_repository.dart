// Kanso — rust_voice_settings_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/voice.dart' as bridge;
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../../data/voice_settings.dart';
import '../abs/voice_settings_repository.dart';

/// The voice settings kept by the Rust core, with the library's other
/// settings.
@LazySingleton(as: VoiceSettingsRepository)
class RustVoiceSettingsRepository implements VoiceSettingsRepository {
  RustVoiceSettingsRepository(this.library);

  final KansoLibrary library;

  @override
  Future<Either<ApplicationError, VoiceSettings>> settings() =>
      attempt(() async {
        final read = await library.voiceSettings();
        return VoiceSettings(spokenCommands: read.spokenCommands);
      });

  @override
  Future<Either<ApplicationError, VoiceSettings>> save(
    VoiceSettings settings,
  ) => attempt(() async {
    await library.setVoiceSettings(
      settings: bridge.VoiceSettings(spokenCommands: settings.spokenCommands),
    );
    return settings;
  });
}
