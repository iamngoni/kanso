// Kanso — rust_player_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/player.dart';
import '../../../../core/bridge/api/player_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../../../services/recording_audio_session.dart';
import '../abs/player_repository.dart';

/// [PlayerRepository] over the Rust core's library.
@LazySingleton(as: PlayerRepository)
class RustPlayerRepository implements PlayerRepository {
  RustPlayerRepository(this._library, this._audio);

  final KansoLibrary _library;
  final RecordingAudioSession _audio;

  @override
  Future<Either<ApplicationError, NoteRecording?>> noteRecording(
    String noteId,
  ) => attempt(() => _library.players().noteRecording(noteId: noteId));

  @override
  PartsState partsState(List<String> fileIds) =>
      _library.players().partsState(fileIds: fileIds);

  @override
  Future<Either<ApplicationError, PlaybackSession>> open(
    List<String> fileIds,
  ) => attempt<PlaybackSession>(() async {
    // The phone's side first: iOS wants the playback category before the
    // speaker is opened.
    await _audio.playback();
    final player = await _library.players().open(fileIds: fileIds);
    return _RustPlaybackSession(player);
  });
}

class _RustPlaybackSession implements PlaybackSession {
  _RustPlaybackSession(this._player);

  final KansoPlayer _player;

  @override
  PlayerStatus get status => _player.status();

  @override
  void play() => _player.play();

  @override
  void pause() => _player.pause();

  @override
  void toggle() => _player.toggle();

  @override
  void seekMs(int ms) => _player.seekMs(ms: ms);

  @override
  void skipMs(int deltaMs) => _player.skipMs(deltaMs: deltaMs);

  @override
  void setSpeed(double speed) => _player.setSpeed(speed: speed);

  @override
  void setVolume(double volume) => _player.setVolume(volume: volume);

  @override
  void dispose() => _player.dispose();
}
