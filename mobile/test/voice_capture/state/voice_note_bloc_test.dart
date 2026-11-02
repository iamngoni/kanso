// Kanso — voice_note_bloc_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/voice_capture_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/dictation/models/data/voice_note_status.dart';
import 'package:kanso/dictation/state/voice_note/voice_note_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

class _FakeInput extends Fake implements VoiceCaptureInput {}

VoiceCaptureSaved _saved({String? id = 'note:cap-1', bool tidyNow = false}) =>
    VoiceCaptureSaved(
      noteId: id,
      problems: const <String>[],
      existed: false,
      meeting: false,
      tidyNow: tidyNow,
    );

void main() {
  late MockVoiceCaptureRepository voice;
  var clock = DateTime.utc(2026, 11, 2, 9);

  setUpAll(() => registerFallbackValue(_FakeInput()));

  setUp(() {
    voice = MockVoiceCaptureRepository();
    clock = DateTime.utc(2026, 11, 2, 9);
  });

  VoiceNoteBloc build() => VoiceNoteBloc(voice)..now = () => clock;

  test('the length is counted from when listening started', () async {
    when(() => voice.save(any())).thenAnswer(
      (_) async => Right<ApplicationError, VoiceCaptureSaved>(_saved()),
    );
    final bloc = build()..add(const VoiceNoteEvent.listeningStarted());
    await Future<void>.delayed(Duration.zero);
    clock = clock.add(const Duration(seconds: 42));
    bloc.add(const VoiceNoteEvent.saveRequested(' Hello there '));
    await bloc.stream.firstWhere((s) => s.status == VoiceNoteStatus.saved);
    final input =
        verify(() => voice.save(captureAny())).captured.single
            as VoiceCaptureInput;
    expect(input.durationMs, 42000);
    expect(input.transcript, 'Hello there');
    expect(bloc.state.noteId, 'note:cap-1');
    await bloc.close();
  });

  test('nothing heard saves nothing', () async {
    final bloc = build()..add(const VoiceNoteEvent.saveRequested('  '));
    await Future<void>.delayed(Duration.zero);
    verifyNever(() => voice.save(any()));
    expect(bloc.state.status, VoiceNoteStatus.idle);
    await bloc.close();
  });

  test('a save that fails keeps the words to try again', () async {
    when(() => voice.save(any())).thenAnswer(
      (_) async => const Left<ApplicationError, VoiceCaptureSaved>(
        ApplicationError.storage(message: 'Disk full.'),
      ),
    );
    final bloc = build()..add(const VoiceNoteEvent.saveRequested('Keep'));
    final state = await bloc.stream.firstWhere(
      (s) => s.status == VoiceNoteStatus.failed,
    );
    expect(state.error, isNotNull);
    await bloc.close();
  });

  test('Always tidies the note it saved, in the background', () async {
    when(() => voice.save(any())).thenAnswer(
      (_) async =>
          Right<ApplicationError, VoiceCaptureSaved>(_saved(tidyNow: true)),
    );
    when(() => voice.tidy(any())).thenAnswer(
      (_) async => const Right<ApplicationError, TidyOutcome>(
        TidyOutcome(insertedAbove: false, dropped: 0),
      ),
    );
    final bloc = build()
      ..add(const VoiceNoteEvent.saveRequested('Plan the week'));
    await bloc.stream.firstWhere((s) => s.status == VoiceNoteStatus.saved);
    verify(() => voice.tidy('note:cap-1')).called(1);
    await bloc.close();
  });
}
