// Kanso — voice_suite.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/bridge/api/voice.dart';
import 'package:kanso/core/bridge/api/voice_capture_types.dart';
import 'package:path/path.dart' as p;

import 'open_library.dart';

/// Voice input, through the bridge: the spoken commands and their setting.
void voiceSuite(OpenLibrary open) {
  group('voice input', () {
    test('spoken punctuation and a UTF-16 caret', () async {
      final said = dictated(
        before: '',
        transcript: 'Buy milk. Comma. Eggs.',
        commands: true,
      );
      expect(said?.text, 'Buy milk, eggs.');
      expect(said?.caretUtf16, 'Buy milk, eggs.'.length);

      // An emoji is two UTF-16 units, as Dart counts them.
      final emoji = dictated(
        before: '',
        transcript: '  hello 😀  ',
        commands: false,
      );
      expect(emoji?.text, 'hello 😀');
      expect(emoji?.caretUtf16, 'hello 😀'.length);
      expect(
        dictated(before: '', transcript: '   ', commands: false),
        isNull,
      );
    });

    test('the setting is on until turned off, and is kept', () async {
      final library = await open();
      expect((await library.voiceSettings()).spokenCommands, isTrue);
      await library.setVoiceSettings(
        settings: const VoiceSettings(
          spokenCommands: false,
          tidy: VoiceTidy.ask,
          meetings: true,
          keepAudio: false,
          allowServerRecognition: false,
        ),
      );
      expect((await library.voiceSettings()).spokenCommands, isFalse);
    });
  });

  group('voice capture', () {
    const id = 'cd34cd34cd34cd34cd34cd34cd34cd34';

    test('a spoken note lands in the Inbox as a voice capture', () async {
      final library = await open();
      final voice = library.voiceCapture();
      final saved = await voice.save(
        input: const VoiceCaptureInput(
          id: id,
          startedMs: 1000,
          durationMs: 12000,
          transcript: 'Remember to call the dentist tomorrow morning.',
          onDevice: true,
        ),
      );
      expect(saved.noteId, 'note:cap-$id');
      expect(saved.existed, isFalse);
      expect(saved.meeting, isFalse);
      final note = await library.note(id: 'note:cap-$id');
      expect(note.body, contains('call the dentist'));
      final info = await voice.noteInfo(id: 'note:cap-$id');
      expect(info?.durationMs, 12000);
      expect(info?.tidy, 'none');

      // The same capture again is the same note.
      final again = await voice.save(
        input: const VoiceCaptureInput(
          id: id,
          startedMs: 1000,
          durationMs: 12000,
          transcript: 'Remember to call the dentist tomorrow morning.',
          onDevice: true,
        ),
      );
      expect(again.existed, isTrue);
    });

    test('nothing heard saves nothing, and a plain note has no info', () async {
      final library = await open();
      final voice = library.voiceCapture();
      final saved = await voice.save(
        input: const VoiceCaptureInput(
          id: id,
          startedMs: 1,
          durationMs: 500,
          transcript: '   ',
          onDevice: true,
        ),
      );
      expect(saved.noteId, isNull);
      final plain = await library.create(
        note: const NewNote(title: 'Plain', body: 'text'),
      );
      expect(await voice.noteInfo(id: plain.id), isNull);
    });

    test('tidying needs AI, which a new library has off', () async {
      final library = await open();
      final voice = library.voiceCapture();
      await voice.save(
        input: const VoiceCaptureInput(
          id: id,
          startedMs: 1,
          durationMs: 4000,
          transcript: 'Buy milk and eggs.',
          onDevice: true,
        ),
      );
      await expectLater(
        voice.tidyNote(id: 'note:cap-$id', run: library.ai().newRun()),
        throwsA(anything),
      );
    });

    test('recordings left outside the app are listed and saved', () async {
      final library = await open();
      final voice = library.voiceCapture();
      final folder = await Directory.systemTemp.createTemp('kanso_voice_');
      addTearDown(() => folder.delete(recursive: true));
      final inbox = p.join(folder.path, 'inbox');
      expect(await voice.drops(inbox: inbox), isEmpty);

      Directory drop(String name) {
        final dir = Directory(p.join(inbox, name))..createSync(recursive: true);
        File(p.join(dir.path, 'rec.m4a')).writeAsBytesSync(
          <int>[0, 0, 0, 24, 102, 116, 121, 112, 77, 52, 65, 32],
        );
        File(p.join(dir.path, 'drop.json')).writeAsStringSync(
          jsonEncode(<String, Object?>{
            'version': 1,
            'kind': 'voice',
            'createdAt': 1,
            'started_ms': 1000,
            'duration_ms': 7000,
            'audio': 'rec.m4a',
          }),
        );
        return dir;
      }

      final first = drop(id);
      final second = drop('ef56ef56ef56ef56ef56ef56ef56ef56');
      final third = drop('12ab12ab12ab12ab12ab12ab12ab12ab');
      final listed = await voice.drops(inbox: inbox);
      expect(listed, hasLength(3));

      final saved = await voice.saveDrop(
        inbox: inbox,
        id: id,
        transcript: 'Pick up the parcel.',
        onDevice: true,
      );
      expect(saved.noteId, 'note:cap-$id');
      expect(first.existsSync(), isFalse);

      final audioOnly = await voice.saveDropAudioOnly(
        inbox: inbox,
        id: 'ef56ef56ef56ef56ef56ef56ef56ef56',
      );
      expect(audioOnly.noteId, isNotNull);
      expect(second.existsSync(), isFalse);

      await voice.discardDrop(
        inbox: inbox,
        id: '12ab12ab12ab12ab12ab12ab12ab12ab',
      );
      expect(third.existsSync(), isFalse);
    });
  });
}
