// Kanso — recording_test_support.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/recording_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/recording/models/repos/abs/recording_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockRecordingRepository extends Mock implements RecordingRepository {}

/// A recording with no microphone: the test sets what the recorder reports.
class FakeRecordingSession implements RecordingSession {
  RecordingStatus current = const RecordingStatus(
    seconds: 0,
    level: 0,
    paused: false,
    warned: false,
  );
  bool paused = false;
  bool stopped = false;
  bool discarded = false;
  Either<ApplicationError, RecordingSaved> stopResult =
      const Right<ApplicationError, RecordingSaved>(savedNote);

  @override
  RecordingStatus get status => current;

  @override
  void pause() => paused = true;

  @override
  void resume() => paused = false;

  @override
  Future<Either<ApplicationError, RecordingSaved>> stop() async {
    stopped = true;
    return stopResult;
  }

  @override
  Future<Either<ApplicationError, Unit>> discard() async {
    discarded = true;
    return const Right<ApplicationError, Unit>(unit);
  }
}

const RecordingSaved savedNote = RecordingSaved(
  noteId: 'note:rec-abc',
  fileIds: <String>['file:rec-abc-1'],
  durationSecs: 3120,
  created: true,
);

const RecordingFound foundRecording = RecordingFound(
  id: 'crash-1',
  startedMs: 1,
  seconds: 3120,
  text: 'Kanso closed while recording at 14:05 (52 min).',
);
