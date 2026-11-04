// Kanso — rust_recording_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/recording.dart';
import '../../../../core/bridge/api/recording_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/models/data/library_folder.dart';
import '../../../../core/services/external_changes.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../../../services/recording_audio_session.dart';
import '../abs/recording_repository.dart';

/// [RecordingRepository] over the Rust core's library.
@LazySingleton(as: RecordingRepository)
class RustRecordingRepository implements RecordingRepository {
  RustRecordingRepository(
    this._library,
    this._folder,
    this._external,
    this._audio,
  );

  final KansoLibrary _library;
  final LibraryFolder _folder;
  final ExternalChanges _external;
  final RecordingAudioSession _audio;

  @override
  Future<Either<ApplicationError, RecordingSession>> start({
    String? noteId,
  }) async {
    // The phone asks for the microphone and, on iOS, sets the audio
    // session to record: the Rust recorder can do neither.
    if (!await _audio.prepare()) {
      return const Left(
        ApplicationError.invalid(message: 'The microphone is not allowed.'),
      );
    }
    final started = await attempt<RecordingSession>(() async {
      final recording = await _library.recorder().start(
        dataDir: _folder.path,
        noteId: noteId,
      );
      return _RustRecordingSession(recording, _external, _audio);
    });
    if (started.isLeft()) {
      await _audio.release();
    }
    return started;
  }

  @override
  Future<Either<ApplicationError, List<RecordingFound>>> found() =>
      attempt(() => _library.recorder().found(dataDir: _folder.path));

  @override
  Future<Either<ApplicationError, RecordingSaved>> recover(String id) =>
      attempt(() async {
        final saved = await _library.recorder().recover(
          dataDir: _folder.path,
          id: id,
        );
        _external.announce();
        return saved;
      });

  @override
  Future<Either<ApplicationError, Unit>> discardFound(String id) =>
      attempt(() async {
        await _library.recorder().discardFound(dataDir: _folder.path, id: id);
        return unit;
      });
}

class _RustRecordingSession implements RecordingSession {
  _RustRecordingSession(this._recording, this._external, this._audio);

  final KansoRecording _recording;
  final ExternalChanges _external;
  final RecordingAudioSession _audio;

  @override
  RecordingStatus get status => _recording.status();

  @override
  void pause() => _recording.pause();

  @override
  void resume() => _recording.resume();

  @override
  Future<Either<ApplicationError, RecordingSaved>> stop() async {
    final result = await attempt(() async {
      final saved = await _recording.stop();
      _external.announce();
      return saved;
    });
    await _audio.release();
    return result;
  }

  @override
  Future<Either<ApplicationError, Unit>> discard() async {
    final result = await attempt(() async {
      await _recording.discard();
      return unit;
    });
    await _audio.release();
    return result;
  }
}
