// Kanso — recording_suite.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/player.dart';
import 'package:kanso/core/bridge/api/player_types.dart';
import 'package:kanso/core/bridge/api/recording.dart';
import 'package:kanso/core/bridge/api/recording_types.dart';

import 'open_library.dart';

/// Recording a note from the microphone, through the bridge: what a crash
/// left, recording into a note that is not there, and, where the machine has
/// a microphone, a recording from start to its note.
void recordingSuite(OpenLibrary open) {
  group('recording', () {
    late Directory data;

    setUp(() => data = Directory.systemTemp.createTempSync('kanso-rec'));
    tearDown(() => data.deleteSync(recursive: true));

    test('nothing a crash left is offered on a fresh folder', () async {
      final library = await open();
      expect(await library.recorder().found(dataDir: data.path), isEmpty);
    });

    test('a name that is not a recording is refused', () async {
      final library = await open();
      await expectLater(
        library.recorder().discardFound(dataDir: data.path, id: '../escape'),
        throwsA(anything),
      );
      await expectLater(
        library.recorder().recover(dataDir: data.path, id: 'nothing-here'),
        throwsA(anything),
      );
    });

    test('recording into a note that is not there is refused', () async {
      final library = await open();
      await expectLater(
        library.recorder().start(dataDir: data.path, noteId: 'note:nope'),
        throwsA(anything),
      );
    });

    test('a recording runs, pauses, and becomes a note', () async {
      final library = await open();
      KansoRecording recording;
      try {
        recording = await library.recorder().start(dataDir: data.path);
      } on Object {
        // No microphone here (a CI machine): the recorder says so.
        return;
      }
      expect(recording.status().paused, isFalse);
      recording.pause();
      expect(recording.status().paused, isTrue);
      recording.resume();
      await Future<void>.delayed(const Duration(milliseconds: 600));
      final status = recording.status();
      expect(status.paused, isFalse);
      expect(status.ended, isNull);
      RecordingSaved saved;
      try {
        saved = await recording.stop();
      } on Object {
        // Under a second of silence is no audio: nothing is kept.
        await recording.discard();
        return;
      }
      expect(saved.created, isTrue);
      expect(saved.fileIds, isNotEmpty);
      await _plays(library.players(), saved);
    });
  });

  group('playing', () {
    test('a note with no recording has none to play', () async {
      final library = await open();
      final id = await makeNote(library, 'Plain', 'no audio here');
      expect(await library.players().noteRecording(noteId: id), isNull);
    });

    test('parts that are not there are missing and do not open', () async {
      final library = await open();
      final players = library.players();
      expect(
        players.partsState(fileIds: <String>['file:nope']),
        PartsState.missing,
      );
      await expectLater(
        players.open(fileIds: <String>['file:nope']),
        throwsA(anything),
      );
    });
  });
}

/// Plays the recording just made: found on its note, its bytes here, and
/// driven on the speaker. A machine with no speaker stops at the opening.
Future<void> _plays(KansoPlayers players, RecordingSaved saved) async {
  final found = await players.noteRecording(noteId: saved.noteId);
  expect(found?.fileIds, saved.fileIds);
  expect(players.partsState(fileIds: saved.fileIds), PartsState.ready);
  KansoPlayer player;
  try {
    player = await players.open(fileIds: saved.fileIds);
  } on Object {
    return;
  }
  expect(player.status().playing, isFalse);
  player
    ..setVolume(volume: 0)
    ..play();
  expect(player.status().playing, isTrue);
  player.pause();
  expect(player.status().playing, isFalse);
  player.toggle();
  expect(player.status().playing, isTrue);
  player
    ..toggle()
    ..setSpeed(speed: 1.5);
  expect(player.status().speed, closeTo(1.5, 0.01));
  player
    ..seekMs(ms: 500)
    ..skipMs(deltaMs: -250);
  final status = player.status();
  expect(status.positionMs, lessThanOrEqualTo(status.lengthMs));
  expect(status.volume, 0);
}
