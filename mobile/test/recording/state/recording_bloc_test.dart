// Kanso — recording_bloc_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// Events are sent one at a time on purpose: the order is what is tested.
// ignore_for_file: cascade_invocations

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/recording_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/recording/models/repos/abs/recording_repository.dart';
import 'package:kanso/recording/state/recording/recording_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../recording_test_support.dart';

void main() {
  late MockRecordingRepository repository;
  late FakeRecordingSession session;

  setUp(() {
    repository = MockRecordingRepository();
    session = FakeRecordingSession();
    when(() => repository.start(noteId: any(named: 'noteId'))).thenAnswer(
      (_) async => Right<ApplicationError, RecordingSession>(session),
    );
    when(() => repository.found()).thenAnswer(
      (_) async => const Right<ApplicationError, List<RecordingFound>>(
        <RecordingFound>[],
      ),
    );
  });

  RecordingBloc build() =>
      RecordingBloc(repository, tick: const Duration(days: 1));

  test('start runs the recorder, and the tick reads its clock', () async {
    final bloc = build();
    bloc.add(const RecordingEvent.started());
    await pumpEventQueue();
    expect(bloc.state.phase, RecordingPhase.recording);
    session.current = const RecordingStatus(
      seconds: 42,
      level: 0.5,
      paused: false,
      warned: false,
    );
    bloc.add(const RecordingEvent.ticked());
    await pumpEventQueue();
    expect(bloc.state.seconds, 42);
    expect(bloc.state.level, 0.5);
    await bloc.close();
  });

  test('a second start is refused while one runs', () async {
    final bloc = build();
    bloc.add(const RecordingEvent.started());
    bloc.add(const RecordingEvent.started());
    await pumpEventQueue();
    expect(bloc.state.problem, RecordingProblem.busy);
    expect(bloc.state.phase, RecordingPhase.recording);
    verify(() => repository.start()).called(1);
    await bloc.close();
  });

  test('pause and resume reach the recorder', () async {
    final bloc = build();
    bloc.add(const RecordingEvent.started());
    bloc.add(const RecordingEvent.paused());
    await pumpEventQueue();
    expect(session.paused, isTrue);
    expect(bloc.state.paused, isTrue);
    bloc.add(const RecordingEvent.resumed());
    await pumpEventQueue();
    expect(session.paused, isFalse);
    expect(bloc.state.paused, isFalse);
    await bloc.close();
  });

  test('stop keeps the recording and reports its note', () async {
    final bloc = build();
    bloc.add(const RecordingEvent.started(noteId: 'note:a'));
    bloc.add(const RecordingEvent.stopped());
    await pumpEventQueue();
    expect(session.stopped, isTrue);
    expect(bloc.state.phase, RecordingPhase.idle);
    expect(bloc.state.saved, savedNote);
    await bloc.close();
  });

  test('a stop that fails keeps the audio and checks for recovery', () async {
    session.stopResult = const Left<ApplicationError, RecordingSaved>(
      ApplicationError.storage(message: 'disk'),
    );
    final bloc = build();
    bloc.add(const RecordingEvent.started());
    bloc.add(const RecordingEvent.stopped());
    await pumpEventQueue();
    expect(bloc.state.problem, RecordingProblem.saveFailed);
    expect(bloc.state.phase, RecordingPhase.idle);
    verify(() => repository.found()).called(1);
    await bloc.close();
  });

  test('a recording the cap ended is stopped and kept', () async {
    final bloc = build();
    bloc.add(const RecordingEvent.started());
    await pumpEventQueue();
    session.current = const RecordingStatus(
      seconds: 21600,
      level: 0,
      paused: false,
      warned: true,
      ended: RecordingEnd.cap,
    );
    bloc.add(const RecordingEvent.ticked());
    await pumpEventQueue();
    expect(session.stopped, isTrue);
    expect(bloc.state.ended, RecordingEnd.cap);
    expect(bloc.state.saved, savedNote);
    await bloc.close();
  });

  test('discard deletes it', () async {
    final bloc = build();
    bloc.add(const RecordingEvent.started());
    bloc.add(const RecordingEvent.discarded());
    await pumpEventQueue();
    expect(session.discarded, isTrue);
    expect(bloc.state.phase, RecordingPhase.idle);
    expect(bloc.state.saved, isNull);
    await bloc.close();
  });

  test('a locked note and a denied microphone say why', () async {
    when(() => repository.start(noteId: any(named: 'noteId'))).thenAnswer(
      (_) async => const Left<ApplicationError, RecordingSession>(
        ApplicationError.locked(message: 'locked'),
      ),
    );
    final bloc = build();
    bloc.add(const RecordingEvent.started(noteId: 'note:a'));
    await pumpEventQueue();
    expect(bloc.state.problem, RecordingProblem.locked);
    expect(bloc.state.phase, RecordingPhase.idle);
    when(() => repository.start(noteId: any(named: 'noteId'))).thenAnswer(
      (_) async => const Left<ApplicationError, RecordingSession>(
        ApplicationError.unexpected(message: 'denied'),
      ),
    );
    bloc.add(const RecordingEvent.started());
    await pumpEventQueue();
    expect(bloc.state.problem, RecordingProblem.cannotStart);
    await bloc.close();
  });

  test('recover and discard act on what a crash left', () async {
    when(() => repository.found()).thenAnswer(
      (_) async => const Right<ApplicationError, List<RecordingFound>>(
        <RecordingFound>[foundRecording],
      ),
    );
    when(() => repository.recover('crash-1')).thenAnswer(
      (_) async => const Right<ApplicationError, RecordingSaved>(savedNote),
    );
    when(() => repository.discardFound('crash-1')).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    final bloc = build();
    bloc.add(const RecordingEvent.foundChecked());
    await pumpEventQueue();
    expect(bloc.state.found, <RecordingFound>[foundRecording]);
    bloc.add(const RecordingEvent.recovered('crash-1'));
    await pumpEventQueue();
    expect(bloc.state.found, isEmpty);
    expect(bloc.state.saved, savedNote);
    bloc.add(const RecordingEvent.foundChecked());
    await pumpEventQueue();
    bloc.add(const RecordingEvent.foundDiscarded('crash-1'));
    await pumpEventQueue();
    verify(() => repository.discardFound('crash-1')).called(1);
    expect(bloc.state.found, isEmpty);
    await bloc.close();
  });
}
