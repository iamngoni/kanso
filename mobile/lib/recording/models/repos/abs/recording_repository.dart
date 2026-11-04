// Kanso — recording_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/recording_types.dart';
import '../../../../core/models/data/application_error.dart';

/// A recording in progress (issue #38): the Rust core's recorder, seen from
/// the app.
abstract interface class RecordingSession {
  /// The clock, level and whether it ended by itself.
  RecordingStatus get status;

  void pause();

  void resume();

  /// Stops, attaches the audio and puts it in its note.
  Future<Either<ApplicationError, RecordingSaved>> stop();

  /// Stops and deletes the recording.
  Future<Either<ApplicationError, Unit>> discard();
}

/// Recording a note from the microphone, and what a crash left.
abstract interface class RecordingRepository {
  /// Starts recording into a new note, or into [noteId]. A locked note, a
  /// denied microphone and a missing one are errors.
  Future<Either<ApplicationError, RecordingSession>> start({String? noteId});

  /// The recordings a crash left.
  Future<Either<ApplicationError, List<RecordingFound>>> found();

  /// Finishes a left-behind recording and attaches it to its note.
  Future<Either<ApplicationError, RecordingSaved>> recover(String id);

  /// Deletes a left-behind recording.
  Future<Either<ApplicationError, Unit>> discardFound(String id);
}
