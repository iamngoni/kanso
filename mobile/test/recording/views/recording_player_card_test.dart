// Kanso — recording_player_card_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/player_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/recording/models/repos/abs/player_repository.dart';
import 'package:kanso/recording/state/player/player_bloc.dart';
import 'package:kanso/recording/views/widgets/recording_player_card.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/golden.dart';
import '../../helpers/pump_routed.dart';
import '../player_test_support.dart';

void main() {
  late FakePlaybackSession session;
  late MockPlayerRepository repository;
  late PlayerBloc bloc;

  setUpAll(() => registerFallbackValue(<String>[]));

  setUp(() {
    session = FakePlaybackSession(lengthMs: 52 * 60 * 1000);
    repository = repositoryWith(
      recording: const NoteRecording(
        fileIds: <String>['file:rec-abc-1'],
        state: PartsState.ready,
        lengthMs: 52 * 60 * 1000,
      ),
      session: session,
    );
    bloc = PlayerBloc(repository, tick: const Duration(days: 1));
  });

  tearDown(() => bloc.close());

  /// Sends [events] and lets the bloc's work (real async) finish.
  Future<void> send(WidgetTester tester, List<PlayerEvent> events) async {
    await tester.runAsync(() async {
      events.forEach(bloc.add);
      await pumpEventQueue();
    });
    await tester.pumpAndSettle();
  }

  Widget card({VoidCallback? onSyncNow}) => Scaffold(
    body: Padding(
      padding: const EdgeInsets.all(16),
      child: RecordingPlayerCard(bloc: bloc, onSyncNow: onSyncNow),
    ),
  );

  testWidgets('a note without a recording has no card and takes no room', (
    tester,
  ) async {
    final none = repositoryWith(recording: null);
    final quiet = PlayerBloc(none, tick: const Duration(days: 1));
    addTearDown(quiet.close);
    await pumpRouted(
      tester,
      Scaffold(body: RecordingPlayerCard(bloc: quiet)),
    );
    await tester.runAsync(() async {
      quiet.add(const PlayerEvent.started('note:a'));
      await pumpEventQueue();
    });
    await tester.pump();
    expect(tester.getSize(find.byType(RecordingPlayerCard)).height, 0);
    expect(find.bySemanticsLabel('Play'), findsNothing);
  });

  testWidgets('the card shows play, the times and the speed', (tester) async {
    await pumpRouted(tester, card());
    await send(tester, <PlayerEvent>[const PlayerEvent.started('note:a')]);
    expect(find.bySemanticsLabel('Play'), findsOneWidget);
    expect(find.text('00:00'), findsOneWidget);
    expect(find.text('52:00'), findsOneWidget);
    expect(find.text('1×'), findsOneWidget);
    expect(find.bySemanticsLabel('Mute'), findsOneWidget);
    expect(find.bySemanticsLabel('Back 5 seconds'), findsOneWidget);
    expect(find.bySemanticsLabel('Forward 5 seconds'), findsOneWidget);
    expect(
      find.bySemanticsLabel(RegExp('Position in the recording')),
      findsOneWidget,
    );
  });

  testWidgets('Play plays and the button turns into Pause', (tester) async {
    await pumpRouted(tester, card());
    await send(tester, <PlayerEvent>[const PlayerEvent.started('note:a')]);
    await tester.tap(find.bySemanticsLabel('Play'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.playing, isTrue);
    expect(find.bySemanticsLabel('Pause'), findsOneWidget);
  });

  testWidgets('the skip buttons and the speed button reach the player', (
    tester,
  ) async {
    await pumpRouted(tester, card());
    await send(tester, <PlayerEvent>[
      const PlayerEvent.started('note:a'),
      const PlayerEvent.seeked(60000),
    ]);
    await tester.tap(find.bySemanticsLabel('Forward 5 seconds'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.positionMs, 65000);
    expect(find.text('01:05'), findsOneWidget);
    await tester.tap(find.bySemanticsLabel('Back 5 seconds'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.positionMs, 60000);
    await tester.tap(find.text('1×'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.speed, 1.25);
    expect(find.text('1.25×'), findsOneWidget);
  });

  testWidgets('releasing the scrubber seeks; dragging alone does not', (
    tester,
  ) async {
    await pumpRouted(tester, card());
    await send(tester, <PlayerEvent>[const PlayerEvent.started('note:a')]);
    final slider = find.byType(Slider).first;
    final gesture = await tester.startGesture(tester.getCenter(slider));
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    expect(session.current.positionMs, 0, reason: 'seeks on release');
    await gesture.up();
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.positionMs, greaterThan(0));
    expect(session.current.positionMs, lessThanOrEqualTo(52 * 60 * 1000));
  });

  testWidgets('the mute button silences and restores the sound', (
    tester,
  ) async {
    await pumpRouted(tester, card());
    await send(tester, <PlayerEvent>[
      const PlayerEvent.started('note:a'),
      const PlayerEvent.toggled(),
    ]);
    await tester.tap(find.bySemanticsLabel('Mute'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.volume, 0);
    expect(find.bySemanticsLabel('Unmute'), findsOneWidget);
    await tester.tap(find.bySemanticsLabel('Unmute'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.current.volume, 1);
  });

  testWidgets('audio that has not arrived says so and offers Sync Now', (
    tester,
  ) async {
    final waiting = repositoryWith(
      recording: const NoteRecording(
        fileIds: <String>['file:rec-abc-1'],
        state: PartsState.notHere,
      ),
    );
    final later = PlayerBloc(waiting, tick: const Duration(days: 1));
    addTearDown(later.close);
    var synced = 0;
    await pumpRouted(
      tester,
      Scaffold(
        body: RecordingPlayerCard(bloc: later, onSyncNow: () => synced++),
      ),
    );
    await tester.runAsync(() async {
      later.add(const PlayerEvent.started('note:a'));
      await pumpEventQueue();
    });
    await tester.pumpAndSettle();
    expect(find.text('Audio not on this device yet'), findsOneWidget);
    expect(find.bySemanticsLabel('Play'), findsNothing);
    await tester.tap(find.text('Sync Now'));
    expect(synced, 1);
    await tester.runAsync(later.close);
  });

  testWidgets('a speaker that fails says so and offers Try Again', (
    tester,
  ) async {
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
    await pumpRouted(
      tester,
      Scaffold(body: RecordingPlayerCard(bloc: failing)),
    );
    await tester.runAsync(() async {
      failing
        ..add(const PlayerEvent.started('note:a'))
        ..add(const PlayerEvent.toggled());
      await pumpEventQueue();
    });
    await tester.pumpAndSettle();
    expect(find.text("Couldn't play the recording"), findsOneWidget);
    expect(find.text('Try Again'), findsOneWidget);
  });

  for (final brightness in Brightness.values) {
    testWidgets('the card, paused part way, ${brightness.name}', (
      tester,
    ) async {
      await send(tester, <PlayerEvent>[
        const PlayerEvent.started('note:a'),
        const PlayerEvent.seeked(754000),
      ]);
      await expectGoldenOnBothPlatforms(
        tester,
        card(),
        name: 'phone_player_paused_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    testWidgets('the card, playing fast and quiet, ${brightness.name}', (
      tester,
    ) async {
      await send(tester, <PlayerEvent>[
        const PlayerEvent.started('note:a'),
        const PlayerEvent.speedCycled(),
        const PlayerEvent.speedCycled(),
        const PlayerEvent.volumeChanged(0.4),
        const PlayerEvent.momentTapped('file:rec-abc-1', 1500),
      ]);
      await expectGoldenOnBothPlatforms(
        tester,
        card(),
        name: 'phone_player_playing_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    testWidgets('the card, audio not here yet, ${brightness.name}', (
      tester,
    ) async {
      final waiting = repositoryWith(
        recording: const NoteRecording(
          fileIds: <String>['file:rec-abc-1'],
          state: PartsState.notHere,
        ),
      );
      final later = PlayerBloc(waiting, tick: const Duration(days: 1));
      addTearDown(later.close);
      await tester.runAsync(() async {
        later.add(const PlayerEvent.started('note:a'));
        await pumpEventQueue();
      });
      await expectGoldenOnBothPlatforms(
        tester,
        Scaffold(
          body: Padding(
            padding: const EdgeInsets.all(16),
            child: RecordingPlayerCard(bloc: later, onSyncNow: () {}),
          ),
        ),
        name: 'phone_player_not_here_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
      await tester.runAsync(later.close);
    });
  }
}
