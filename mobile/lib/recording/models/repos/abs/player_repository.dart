// Kanso — player_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/player_types.dart';
import '../../../../core/models/data/application_error.dart';

/// A recording on the speaker (issue #38): the Rust core's player, seen from
/// the app. Dropping it with [dispose] stops the sound.
abstract interface class PlaybackSession {
  /// Where playback stands: read about ten times a second while it plays.
  PlayerStatus get status;

  void play();

  void pause();

  void toggle();

  /// Moves to [ms] into the recording.
  void seekMs(int ms);

  /// Moves by [deltaMs] (negative goes back).
  void skipMs(int deltaMs);

  /// 0.5× to 4×.
  void setSpeed(double speed);

  /// 0 to 1.
  void setVolume(double volume);

  /// Stops the sound and lets the speaker go.
  void dispose();
}

/// Finding the recording a note holds, and opening it.
abstract interface class PlayerRepository {
  /// The recording the note holds, with whether its bytes are here and its
  /// length; null when it holds none or is locked.
  Future<Either<ApplicationError, NoteRecording?>> noteRecording(String noteId);

  /// Where the bytes of the parts stand on this device.
  PartsState partsState(List<String> fileIds);

  /// Opens the parts on the speaker, paused at the start.
  Future<Either<ApplicationError, PlaybackSession>> open(List<String> fileIds);
}
