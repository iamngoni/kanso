// Kanso — player_bloc_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/player_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/recording/models/repos/abs/player_repository.dart';
import 'package:kanso/recording/state/player/player_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../player_test_support.dart';

void main() {
  late FakePlaybackSession session;
  late MockPlayerRepository repository;
  late PlayerBloc bloc;

  /// Sends [event] and lets the bloc's work finish.
  Future<void> send(PlayerEvent event) async {
    bloc.add(event);
    await pumpEventQueue();
  }

  setUpAll(() => registerFallbackValue(<String>[]));

  setUp(() {
    session = FakePlaybackSession();
    repository = repositoryWith(session: session);
    bloc = PlayerBloc(repository, tick: const Duration(days: 1));
  });

  tearDown(() => bloc.close());

  test('a note with no recording has no card', () async {
    final none = repositoryWith(recording: null);
    final quiet = PlayerBloc(none, tick: const Duration(days: 1));
    addTearDown(quiet.close);
    quiet.add(const PlayerEvent.started('note:a'));
    await pumpEventQueue();
    expect(quiet.state.phase, PlayerPhase.none);
    expect(quiet.state.shown, isFalse);
  });

  test(
    'a note with a recording shows its card and its length, unopened',
    () async {
      await send(const PlayerEvent.started('note:a'));
      expect(bloc.state.phase, PlayerPhase.idle);
      expect(bloc.state.fileIds, <String>['file:rec-abc-1']);
      expect(bloc.state.lengthMs, 300000);
      // The speaker is not taken until Play.
      verifyNever(() => repository.open(any()));
    },
  );

  test('the first Play opens the speaker and plays, the next pauses', () async {
    await send(const PlayerEvent.started('note:a'));
    await send(const PlayerEvent.toggled());
    verify(() => repository.open(<String>['file:rec-abc-1'])).called(1);
    expect(bloc.state.phase, PlayerPhase.ready);
    expect(bloc.state.playing, isTrue);
    await send(const PlayerEvent.toggled());
    verifyNever(() => repository.open(<String>['other']));
    expect(bloc.state.playing, isFalse);
    expect(session.disposed, isFalse);
  });

  test('a tap on a timestamp seeks, plays and asks for the card', () async {
    await send(const PlayerEvent.started('note:a'));
    await send(const PlayerEvent.momentTapped('file:rec-abc-1', 125));
    expect(bloc.state.positionMs, 125000);
    expect(bloc.state.playing, isTrue);
    expect(bloc.state.reveals, 1);
    await send(const PlayerEvent.momentTapped('file:rec-abc-1', 7));
    expect(bloc.state.positionMs, 7000);
    expect(bloc.state.reveals, 2);
    // A moment of another recording is not followed.
    await send(const PlayerEvent.momentTapped('file:elsewhere', 3));
    expect(bloc.state.positionMs, 7000);
    expect(bloc.state.reveals, 2);
  });

  test('a moment past the end holds at the end', () async {
    await send(const PlayerEvent.started('note:a'));
    await send(const PlayerEvent.momentTapped('file:rec-abc-1', 99999));
    expect(bloc.state.positionMs, bloc.state.lengthMs);
  });

  test(
    'seeking and skipping move the position; speed and volume stick',
    () async {
      await send(const PlayerEvent.started('note:a'));
      await send(const PlayerEvent.seeked(60000));
      expect(bloc.state.positionMs, 60000);
      await send(const PlayerEvent.skipped(5000));
      expect(bloc.state.positionMs, 65000);
      await send(const PlayerEvent.skipped(-70000));
      expect(bloc.state.positionMs, 0);
      for (final expected in <double>[1.25, 1.5, 2, 0.75, 1]) {
        await send(const PlayerEvent.speedCycled());
        expect(bloc.state.speed, expected);
        expect(session.current.speed, expected);
      }
      await send(const PlayerEvent.volumeChanged(0.4));
      expect(session.current.volume, 0.4);
      await send(const PlayerEvent.muteToggled());
      expect(bloc.state.volume, 0);
      await send(const PlayerEvent.muteToggled());
      expect(bloc.state.volume, 0.4);
    },
  );

  test(
    'bytes that have not arrived say so, and the card turns into the player',
    () async {
      final waiting = repositoryWith(
        recording: const NoteRecording(
          fileIds: <String>['file:rec-abc-1'],
          state: PartsState.notHere,
        ),
        session: session,
      );
      final later = PlayerBloc(waiting, tick: const Duration(milliseconds: 5));
      addTearDown(later.close);
      later.add(const PlayerEvent.started('note:a'));
      await pumpEventQueue();
      expect(later.state.phase, PlayerPhase.notHere);
      // Playing is refused until they are here, and nothing is opened.
      later.add(const PlayerEvent.toggled());
      await pumpEventQueue();
      verifyNever(() => waiting.open(any()));
      // Sync brings them.
      when(() => waiting.partsState(any())).thenReturn(PartsState.ready);
      when(() => waiting.noteRecording(any())).thenAnswer(
        (_) async =>
            const Right<ApplicationError, NoteRecording?>(heldRecording),
      );
      for (var i = 0; i < 80 && later.state.phase == PlayerPhase.notHere; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }
      expect(later.state.phase, PlayerPhase.idle);
    },
  );

  test('a recording whose file is gone says so', () async {
    final gone = repositoryWith(
      recording: const NoteRecording(
        fileIds: <String>['file:ghost'],
        state: PartsState.missing,
      ),
    );
    final lost = PlayerBloc(gone, tick: const Duration(days: 1));
    addTearDown(lost.close);
    lost.add(const PlayerEvent.started('note:a'));
    await pumpEventQueue();
    expect(lost.state.phase, PlayerPhase.missing);
  });

  test('a speaker that will not open fails, and Try Again opens it', () async {
    final broken = MockPlayerRepository();
    when(() => broken.noteRecording(any())).thenAnswer(
      (_) async => const Right<ApplicationError, NoteRecording?>(heldRecording),
    );
    when(() => broken.partsState(any())).thenReturn(PartsState.ready);
    when(() => broken.open(any())).thenAnswer(
      (_) async => const Left<ApplicationError, PlaybackSession>(
        ApplicationError.invalid(message: 'There is no speaker.'),
      ),
    );
    final failing = PlayerBloc(broken, tick: const Duration(days: 1));
    addTearDown(failing.close);
    failing
      ..add(const PlayerEvent.started('note:a'))
      ..add(const PlayerEvent.toggled());
    await pumpEventQueue();
    expect(failing.state.phase, PlayerPhase.failed);
    expect(failing.state.message, 'There is no speaker.');
    when(() => broken.open(any())).thenAnswer(
      (_) async => Right<ApplicationError, PlaybackSession>(session),
    );
    failing.add(const PlayerEvent.retried());
    await pumpEventQueue();
    expect(failing.state.phase, PlayerPhase.ready);
    expect(failing.state.playing, isTrue);
  });

  test('closing the bloc lets the speaker go', () async {
    await send(const PlayerEvent.started('note:a'));
    await send(const PlayerEvent.toggled());
    await bloc.close();
    expect(session.disposed, isTrue);
  });

  test('playback that ends by itself reads as stopped at the end', () async {
    final ticking = PlayerBloc(
      repository,
      tick: const Duration(milliseconds: 5),
    );
    addTearDown(ticking.close);
    ticking
      ..add(const PlayerEvent.started('note:a'))
      ..add(const PlayerEvent.toggled());
    await pumpEventQueue();
    expect(ticking.state.playing, isTrue);
    session.current = PlayerStatus(
      positionMs: session.current.lengthMs,
      lengthMs: session.current.lengthMs,
      playing: false,
      ended: true,
      speed: 1,
      volume: 1,
    );
    for (var i = 0; i < 80 && ticking.state.playing; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    expect(ticking.state.playing, isFalse);
    expect(ticking.state.positionMs, ticking.state.lengthMs);
  });
}
