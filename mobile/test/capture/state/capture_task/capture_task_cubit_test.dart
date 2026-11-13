// Kanso — capture_task_cubit_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_mode.dart';
import 'package:kanso/capture/state/capture_task/capture_task_cubit.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

final DateTime _now = DateTime(2026, 10, 20, 10);

void main() {
  late MockCaptureRepository capture;
  late SeededPreferencesRepository preferences;

  setUpAll(() {
    registerFallbackValue(CaptureTarget.dailyNote);
    registerFallbackValue(_now);
  });

  setUp(() {
    capture = MockCaptureRepository();
    preferences = SeededPreferencesRepository(const Preferences());
    when(
      () =>
          capture.taskHint(any(), any(), monthFirst: any(named: 'monthFirst')),
    ).thenReturn(null);
  });

  CaptureTaskCubit make() {
    final cubit = CaptureTaskCubit(capture, preferences);
    addTearDown(cubit.close);
    return cubit;
  }

  Right<ApplicationError, TaskCaptured> captured() =>
      const Right<ApplicationError, TaskCaptured>(
        TaskCaptured(noteId: 'note:daily-2026-10-20', line: '- [ ] x'),
      );

  void stubCapture(Either<ApplicationError, TaskCaptured> result) => when(
    () => capture.captureTask(
      text: any(named: 'text'),
      target: any(named: 'target'),
      now: any(named: 'now'),
      monthFirst: any(named: 'monthFirst'),
    ),
  ).thenAnswer((_) async => result);

  group('CaptureTaskCubit', () {
    test('starts in Note mode, going to the daily note', () async {
      final cubit = make();
      await cubit.started();
      expect(cubit.state.loaded, isTrue);
      expect(cubit.state.mode, CaptureMode.note);
      expect(cubit.state.target, CaptureTarget.dailyNote);
    });

    test('opens in the mode and place that were saved', () async {
      preferences.current = const Preferences(
        captureAsTask: true,
        captureTaskTarget: CaptureTarget.inbox,
      );
      final cubit = make();
      await cubit.started();
      expect(cubit.state.mode, CaptureMode.task);
      expect(cubit.state.target, CaptureTarget.inbox);
    });

    test('remembers the mode it was left in', () async {
      final cubit = make();
      await cubit.modeChosen(CaptureMode.task);
      expect(cubit.state.mode, CaptureMode.task);
      expect(preferences.current.captureAsTask, isTrue);
      await cubit.modeChosen(CaptureMode.note);
      expect(preferences.current.captureAsTask, isFalse);
    });

    test('remembers where tasks go', () async {
      final cubit = make();
      await cubit.targetChosen(CaptureTarget.inbox);
      expect(cubit.state.target, CaptureTarget.inbox);
      expect(preferences.current.captureTaskTarget, CaptureTarget.inbox);
    });

    test('reads the date and repeat of the typed text', () {
      when(
        () => capture.taskHint(
          'film fri 3pm every week',
          any(),
          monthFirst: false,
        ),
      ).thenReturn('Fri 23 Oct, 15:00 · every week');
      final cubit = make()
        ..textChanged('film fri 3pm every week', monthFirst: false, now: _now);
      expect(cubit.state.hint, 'Fri 23 Oct, 15:00 · every week');
      cubit.textChanged('film', monthFirst: false, now: _now);
      expect(cubit.state.hint, isNull);
    });

    test('saves a task where it was told, and is ready for the next', () async {
      stubCapture(captured());
      final cubit = make();
      await cubit.targetChosen(CaptureTarget.inbox);
      final written = await cubit.submitted(
        'film fri 3pm',
        monthFirst: true,
        now: _now,
      );
      expect(written, isTrue);
      expect(cubit.state.status, CaptureTaskStatus.saved);
      expect(cubit.state.hint, isNull);
      verify(
        () => capture.captureTask(
          text: 'film fri 3pm',
          target: CaptureTarget.inbox,
          now: _now,
          monthFirst: true,
        ),
      ).called(1);
      // Typing again clears what was said of the last one.
      cubit.textChanged('x', monthFirst: false, now: _now);
      expect(cubit.state.status, CaptureTaskStatus.idle);
    });

    for (final (error, status) in <(ApplicationError, CaptureTaskStatus)>[
      (
        const ApplicationError.locked(message: 'locked'),
        CaptureTaskStatus.locked,
      ),
      (
        const ApplicationError.invalid(message: 'none'),
        CaptureTaskStatus.nothing,
      ),
      (
        const ApplicationError.storage(message: 'disk'),
        CaptureTaskStatus.failed,
      ),
    ]) {
      test('${status.name}: says what went wrong and writes nothing', () async {
        stubCapture(Left<ApplicationError, TaskCaptured>(error));
        final cubit = make();
        final written = await cubit.submitted(
          'x',
          monthFirst: false,
          now: _now,
        );
        expect(written, isFalse);
        expect(cubit.state.status, status);
      });
    }
  });
}
