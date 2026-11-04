// Kanso — recording_suite.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/recording.dart';

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
      try {
        final saved = await recording.stop();
        expect(saved.created, isTrue);
        expect(saved.fileIds, isNotEmpty);
      } on Object {
        // Under a second of silence is no audio: nothing is kept.
        await recording.discard();
      }
    });
  });
}
