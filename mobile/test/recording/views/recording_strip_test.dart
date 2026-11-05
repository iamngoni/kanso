// Kanso — recording_strip_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/recording_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/recording/models/repos/abs/recording_repository.dart';
import 'package:kanso/recording/state/recording/recording_bloc.dart';
import 'package:kanso/recording/views/pages/recording_page.dart';
import 'package:kanso/recording/views/widgets/recording_strip.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/golden.dart';
import '../../helpers/pump_routed.dart';
import '../recording_test_support.dart';

void main() {
  late MockRecordingRepository repository;
  late FakeRecordingSession session;
  late RecordingBloc bloc;

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
    bloc = RecordingBloc(repository, tick: const Duration(days: 1));
  });

  tearDown(() => bloc.close());

  /// Sends [event] and lets the bloc's work (real async) finish.
  Future<void> send(WidgetTester tester, RecordingEvent event) async {
    await tester.runAsync(() async {
      bloc.add(event);
      await pumpEventQueue();
    });
    await tester.pumpAndSettle();
  }

  testWidgets('the strip takes no room when nothing is recording', (
    tester,
  ) async {
    await pumpRouted(tester, Scaffold(body: RecordingStrip(bloc: bloc)));
    await tester.pump();
    expect(find.text('Stop'), findsNothing);
    expect(find.byType(RecordingStrip), findsOneWidget);
    expect(tester.getSize(find.byType(RecordingStrip)).height, 0);
  });

  testWidgets('a running recording shows its clock and the three actions', (
    tester,
  ) async {
    await pumpRouted(tester, Scaffold(body: RecordingStrip(bloc: bloc)));
    await send(tester, const RecordingEvent.started());
    session.current = const RecordingStatus(
      seconds: 42 * 60 + 17,
      level: 0.4,
      paused: false,
      warned: false,
    );
    await send(tester, const RecordingEvent.ticked());
    expect(find.text('00:42:17'), findsOneWidget);
    expect(find.bySemanticsLabel('Pause'), findsOneWidget);
    expect(find.bySemanticsLabel('Stop'), findsOneWidget);
    expect(find.bySemanticsLabel('Discard'), findsOneWidget);
    await tester.tap(find.bySemanticsLabel('Stop'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.stopped, isTrue);
  });

  testWidgets('Discard asks once before deleting', (tester) async {
    await pumpRouted(tester, Scaffold(body: RecordingStrip(bloc: bloc)));
    await send(tester, const RecordingEvent.started());
    await tester.tap(find.bySemanticsLabel('Discard'));
    await tester.pumpAndSettle();
    expect(find.text('Discard this recording?'), findsOneWidget);
    expect(session.discarded, isFalse);
    await tester.tap(find.text('Discard').last);
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(session.discarded, isTrue);
  });

  testWidgets('a crash shows the banner with Recover and Discard', (
    tester,
  ) async {
    when(() => repository.found()).thenAnswer(
      (_) async => const Right<ApplicationError, List<RecordingFound>>(
        <RecordingFound>[foundRecording],
      ),
    );
    when(() => repository.recover('crash-1')).thenAnswer(
      (_) async => const Right<ApplicationError, RecordingSaved>(savedNote),
    );
    await pumpRouted(tester, Scaffold(body: RecordingStrip(bloc: bloc)));
    await send(tester, const RecordingEvent.foundChecked());
    expect(
      find.text('Kanso closed while recording at 14:05 (52 min).'),
      findsOneWidget,
    );
    await tester.tap(find.text('Recover'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    verify(() => repository.recover('crash-1')).called(1);
  });

  testWidgets('the Record page starts a recording into the note it is given', (
    tester,
  ) async {
    await pumpRouted(
      tester,
      RecordingPage(noteId: 'note:a', bloc: bloc),
    );
    await tester.pump();
    expect(find.text('Into this note'), findsOneWidget);
    await tester.tap(find.text('Start recording'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    verify(() => repository.start(noteId: 'note:a')).called(1);
  });

  testWidgets('a locked note says so on the Record page', (tester) async {
    when(() => repository.start(noteId: any(named: 'noteId'))).thenAnswer(
      (_) async => const Left<ApplicationError, RecordingSession>(
        ApplicationError.locked(message: 'locked'),
      ),
    );
    await pumpRouted(
      tester,
      RecordingPage(noteId: 'note:a', bloc: bloc),
    );
    await tester.tap(find.text('Start recording'));
    await tester.runAsync(pumpEventQueue);
    await tester.pumpAndSettle();
    expect(
      find.text('That note is locked. Unlock it first, then record into it.'),
      findsOneWidget,
    );
  });

  for (final brightness in Brightness.values) {
    testWidgets('the recording bar, ${brightness.name}', (tester) async {
      await send(tester, const RecordingEvent.started());
      session.current = const RecordingStatus(
        seconds: 42 * 60 + 17,
        level: 0.4,
        paused: false,
        warned: false,
      );
      await send(tester, const RecordingEvent.ticked());
      await expectGoldenOnBothPlatforms(
        tester,
        Scaffold(body: RecordingStrip(bloc: bloc)),
        name: 'phone_recording_bar_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    testWidgets('the Record sheet, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        RecordingPage(noteId: 'note:a', bloc: bloc),
        name: 'phone_record_page_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
