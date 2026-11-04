// Kanso — player_test_support.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/player_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/recording/models/repos/abs/player_repository.dart';
import 'package:mocktail/mocktail.dart';

class MockPlayerRepository extends Mock implements PlayerRepository {}

/// A recording with no speaker: it keeps the position the test asks for,
/// and moves it only when told to.
class FakePlaybackSession implements PlaybackSession {
  FakePlaybackSession({int lengthMs = 5 * 60 * 1000})
    : current = PlayerStatus(
        positionMs: 0,
        lengthMs: lengthMs,
        playing: false,
        ended: false,
        speed: 1,
        volume: 1,
      );

  PlayerStatus current;
  bool disposed = false;
  int plays = 0;

  void _set({int? positionMs, bool? playing, double? speed, double? volume}) {
    current = PlayerStatus(
      positionMs: positionMs ?? current.positionMs,
      lengthMs: current.lengthMs,
      playing: playing ?? current.playing,
      ended: false,
      speed: speed ?? current.speed,
      volume: volume ?? current.volume,
    );
  }

  @override
  PlayerStatus get status => current;

  @override
  void play() {
    plays++;
    _set(playing: true);
  }

  @override
  void pause() => _set(playing: false);

  @override
  void toggle() => current.playing ? pause() : play();

  @override
  void seekMs(int ms) => _set(positionMs: ms.clamp(0, current.lengthMs));

  @override
  void skipMs(int deltaMs) => _set(
    positionMs: (current.positionMs + deltaMs).clamp(0, current.lengthMs),
  );

  @override
  void setSpeed(double speed) => _set(speed: speed);

  @override
  void setVolume(double volume) => _set(volume: volume);

  @override
  void dispose() => disposed = true;
}

/// The recording a note holds: one part, here, 5 minutes.
const NoteRecording heldRecording = NoteRecording(
  fileIds: <String>['file:rec-abc-1'],
  state: PartsState.ready,
  lengthMs: 5 * 60 * 1000,
);

/// A repository that finds [recording] in every note and opens [session].
MockPlayerRepository repositoryWith({
  NoteRecording? recording = heldRecording,
  FakePlaybackSession? session,
}) {
  final repository = MockPlayerRepository();
  when(() => repository.noteRecording(any())).thenAnswer(
    (_) async => Right<ApplicationError, NoteRecording?>(recording),
  );
  when(() => repository.partsState(any())).thenReturn(
    recording?.state ?? PartsState.missing,
  );
  if (session != null) {
    when(() => repository.open(any())).thenAnswer(
      (_) async => Right<ApplicationError, PlaybackSession>(session),
    );
  }
  return repository;
}
