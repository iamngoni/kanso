// Kanso — rust_voice_capture_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/voice_capture_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/models/data/library_folder.dart';
import '../../../../core/services/external_changes.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/voice_capture_repository.dart';

/// [VoiceCaptureRepository] over the Rust core's library.
@LazySingleton(as: VoiceCaptureRepository)
class RustVoiceCaptureRepository implements VoiceCaptureRepository {
  RustVoiceCaptureRepository(this._library, this._folder, this._external);

  final KansoLibrary _library;
  final LibraryFolder _folder;
  final ExternalChanges _external;

  @override
  Future<Either<ApplicationError, VoiceCaptureSaved>> save(
    VoiceCaptureInput input,
  ) => attempt(() => _library.voiceCapture().save(input: input));

  @override
  Future<Either<ApplicationError, List<VoiceDrop>>> drops() => attempt(
    () => _library.voiceCapture().drops(inbox: _folder.inboxPath),
  );

  @override
  Future<Either<ApplicationError, VoiceCaptureSaved>> saveDrop(
    String id,
    String transcript, {
    required bool onDevice,
  }) => attempt(() async {
    final saved = await _library.voiceCapture().saveDrop(
      inbox: _folder.inboxPath,
      id: id,
      transcript: transcript,
      onDevice: onDevice,
    );
    _external.announce();
    return saved;
  });

  @override
  Future<Either<ApplicationError, VoiceCaptureSaved>> saveDropAudioOnly(
    String id,
  ) => attempt(() async {
    final saved = await _library.voiceCapture().saveDropAudioOnly(
      inbox: _folder.inboxPath,
      id: id,
    );
    _external.announce();
    return saved;
  });

  @override
  Future<Either<ApplicationError, Unit>> discardDrop(String id) =>
      attempt(() async {
        await _library.voiceCapture().discardDrop(
          inbox: _folder.inboxPath,
          id: id,
        );
        return unit;
      });

  @override
  Future<Either<ApplicationError, VoiceNoteInfo?>> noteInfo(String id) =>
      attempt(() => _library.voiceCapture().noteInfo(id: id));

  @override
  Future<Either<ApplicationError, TidyOutcome>> tidy(String id) => attempt(() {
    final run = _library.ai().newRun();
    return _library.voiceCapture().tidyNote(id: id, run: run);
  });
}
