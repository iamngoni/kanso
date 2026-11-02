// Kanso — voice_drops_bloc_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/voice_capture_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/dictation/models/data/file_transcript.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_repository.dart';
import 'package:kanso/voice_capture/state/voice_drops/voice_drops_bloc.dart';
import 'package:kanso/voice_settings/models/data/voice_settings.dart';
import 'package:kanso/voice_settings/models/repos/abs/voice_settings_repository.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

class MockDictation extends Mock implements DictationRepository {}

class MockVoiceSettings extends Mock implements VoiceSettingsRepository {}

const VoiceDrop _drop = VoiceDrop(
  id: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
  audioPath: '/inbox/a/rec.m4a',
  startedMs: 1,
  durationMs: 5000,
);

const VoiceCaptureSaved _saved = VoiceCaptureSaved(
  noteId: 'note:cap-1',
  problems: <String>[],
  existed: false,
  meeting: false,
  tidyNow: false,
);

void main() {
  late MockVoiceCaptureRepository voice;
  late MockDictation dictation;
  late MockVoiceSettings settings;

  setUp(() {
    voice = MockVoiceCaptureRepository();
    dictation = MockDictation();
    settings = MockVoiceSettings();
    when(() => settings.settings()).thenAnswer(
      (_) async =>
          const Right<ApplicationError, VoiceSettings>(VoiceSettings()),
    );
    when(() => voice.drops()).thenAnswer(
      (_) async =>
          const Right<ApplicationError, List<VoiceDrop>>(<VoiceDrop>[_drop]),
    );
  });

  void heard(FileTranscript transcript) =>
      when(
        () => dictation.transcribeFile(
          any(),
          allowServer: any(named: 'allowServer'),
          localeTag: any(named: 'localeTag'),
        ),
      ).thenAnswer(
        (_) async => Right<ApplicationError, FileTranscript>(transcript),
      );

  VoiceDropsBloc build() => VoiceDropsBloc(voice, dictation, settings);

  blocTest<VoiceDropsBloc, VoiceDropsState>(
    'a transcribed recording becomes a note',
    setUp: () {
      heard(const FileTranscript(text: 'Buy milk', onDevice: true));
      when(
        () => voice.saveDrop(any(), any(), onDevice: any(named: 'onDevice')),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, VoiceCaptureSaved>(_saved),
      );
    },
    build: build,
    act: (bloc) => bloc.add(const VoiceDropsEvent.passRequested()),
    verify: (bloc) {
      expect(bloc.state.saved, <String>['note:cap-1']);
      expect(bloc.state.waiting, 0);
      verify(
        () => voice.saveDrop(_drop.id, 'Buy milk', onDevice: true),
      ).called(1);
    },
  );

  blocTest<VoiceDropsBloc, VoiceDropsState>(
    'a phone that cannot recognize a file keeps the audio as a note',
    setUp: () {
      heard(
        const FileTranscript(
          text: '',
          onDevice: false,
          problem: FileTranscriptProblem.unsupported,
        ),
      );
      when(() => voice.saveDropAudioOnly(any())).thenAnswer(
        (_) async => const Right<ApplicationError, VoiceCaptureSaved>(_saved),
      );
    },
    build: build,
    act: (bloc) => bloc.add(const VoiceDropsEvent.passRequested()),
    verify: (bloc) {
      expect(bloc.state.saved, <String>['note:cap-1']);
      verify(() => voice.saveDropAudioOnly(_drop.id)).called(1);
      verifyNever(
        () => voice.saveDrop(any(), any(), onDevice: any(named: 'onDevice')),
      );
    },
  );

  blocTest<VoiceDropsBloc, VoiceDropsState>(
    'a recording only a server could read waits, untouched',
    setUp: () => heard(
      const FileTranscript(
        text: '',
        onDevice: false,
        problem: FileTranscriptProblem.notOnDevice,
      ),
    ),
    build: build,
    act: (bloc) => bloc.add(const VoiceDropsEvent.passRequested()),
    verify: (bloc) {
      expect(bloc.state.waiting, 1);
      expect(bloc.state.saved, isEmpty);
      verifyNever(() => voice.discardDrop(any()));
      verifyNever(() => voice.saveDropAudioOnly(any()));
    },
  );

  blocTest<VoiceDropsBloc, VoiceDropsState>(
    'an unreadable file is discarded and reported',
    setUp: () {
      heard(
        const FileTranscript(
          text: '',
          onDevice: false,
          problem: FileTranscriptProblem.unreadable,
        ),
      );
      when(() => voice.discardDrop(any())).thenAnswer(
        (_) async => const Right<ApplicationError, Unit>(unit),
      );
    },
    build: build,
    act: (bloc) => bloc.add(const VoiceDropsEvent.passRequested()),
    verify: (bloc) {
      expect(bloc.state.problems, hasLength(1));
      verify(() => voice.discardDrop(_drop.id)).called(1);
    },
  );

  blocTest<VoiceDropsBloc, VoiceDropsState>(
    'nothing waiting does nothing',
    setUp: () => when(() => voice.drops()).thenAnswer(
      (_) async =>
          const Right<ApplicationError, List<VoiceDrop>>(<VoiceDrop>[]),
    ),
    build: build,
    act: (bloc) => bloc.add(const VoiceDropsEvent.passRequested()),
    expect: () => <VoiceDropsState>[],
  );
}
