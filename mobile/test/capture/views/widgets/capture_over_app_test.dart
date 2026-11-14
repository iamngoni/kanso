// Kanso — capture_over_app_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:dartz/dartz.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_mode.dart';
import 'package:kanso/capture/models/repos/abs/capture_window_repository.dart';
import 'package:kanso/capture/state/capture_over_app/capture_over_app_cubit.dart';
import 'package:kanso/capture/state/capture_task/capture_task_cubit.dart';
import 'package:kanso/capture/views/widgets/capture_over_app.dart';
import 'package:kanso/capture/views/widgets/capture_over_app_host.dart';
import 'package:kanso/capture/views/widgets/capture_task_form.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fake_media_picker_repository.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

/// The activity, as the app over another app sees it: the mode its link
/// asks for, and what it was asked to do.
class _FakeWindow implements CaptureWindowRepository {
  _FakeWindow(this.answer);

  final CaptureMode answer;
  int finished = 0;
  final List<String> opened = <String>[];

  @override
  Future<CaptureMode> mode() async => answer;

  @override
  Future<void> finish() async => finished++;

  @override
  Future<void> openInKanso(String link) async => opened.add(link);
}

void main() {
  late MockCaptureRepository capture;
  late SeededPreferencesRepository preferences;
  late FakeMediaPickerRepository picker;

  setUpAll(() {
    registerFallbackValue(CaptureTarget.dailyNote);
    registerFallbackValue(DateTime(2026));
    registerFallbackValue(const Share(files: <ShareFile>[]));
  });

  setUp(() {
    capture = MockCaptureRepository();
    preferences = SeededPreferencesRepository(const Preferences());
    picker = FakeMediaPickerRepository();
    when(
      () =>
          capture.taskHint(any(), any(), monthFirst: any(named: 'monthFirst')),
    ).thenReturn(null);
    when(
      () => capture.captureTask(
        text: any(named: 'text'),
        target: any(named: 'target'),
        now: any(named: 'now'),
        monthFirst: any(named: 'monthFirst'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, TaskCaptured>(
        TaskCaptured(noteId: 'note:daily', line: '- [ ] call Sam'),
      ),
    );
    when(() => capture.capture(any())).thenAnswer(
      (_) async => const Right<ApplicationError, CaptureOutcome>(
        CaptureOutcome(noteId: 'note:cap-1', problems: <String>[]),
      ),
    );
  });

  tearDown(
    () => TestWidgetsFlutterBinding.instance.handleAppLifecycleStateChanged(
      AppLifecycleState.resumed,
    ),
  );

  /// Runs the app of `captureMain` with [window], its window shown unless
  /// [shown] is false.
  Future<void> pump(
    WidgetTester tester,
    _FakeWindow window, {
    bool shown = true,
  }) async {
    final appearance = SeededAppearanceBloc(const AppearanceState());
    final media = CaptureOverAppCubit(capture, picker);
    addTearDown(appearance.close);
    addTearDown(media.close);
    tester.binding.handleAppLifecycleStateChanged(
      shown ? AppLifecycleState.resumed : AppLifecycleState.inactive,
    );
    await tester.pumpWidget(
      CaptureOverApp(
        window: window,
        appearance: appearance,
        tasks: () => CaptureTaskCubit(capture, preferences),
        media: media,
      ),
    );
    await tester.pumpAndSettle();
  }

  group('the capture sheet over another app', () {
    testWidgets('waits for its window, then opens in the mode asked', (
      tester,
    ) async {
      final window = _FakeWindow(CaptureMode.task);
      await pump(tester, window, shown: false);
      // An engine started ahead: nothing until the window is on screen.
      expect(find.byType(CaptureTaskForm), findsNothing);

      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pumpAndSettle();
      expect(find.byType(CaptureTaskForm), findsOneWidget);
      expect(find.text('Add task'), findsOneWidget);
      expect(window.finished, 0);
    });

    testWidgets('a task written says where it went, then the window goes', (
      tester,
    ) async {
      final window = _FakeWindow(CaptureMode.task);
      await pump(tester, window);
      await tester.enterText(find.byType(EditableText), 'call Sam fri 3pm');
      await tester.pump();
      await tester.tap(find.text('Add task'));
      await tester.pump();
      verify(
        () => capture.captureTask(
          text: 'call Sam fri 3pm',
          target: CaptureTarget.dailyNote,
          now: any(named: 'now'),
          monthFirst: any(named: 'monthFirst'),
        ),
      ).called(1);
      expect(find.text('Added to today’s daily note'), findsOneWidget);
      expect(window.finished, 0);

      await tester.pump(CaptureOverAppHost.savedPause);
      await tester.pumpAndSettle();
      expect(find.byType(CaptureTaskForm), findsNothing);
      expect(window.finished, 1);
      expect(window.opened, isEmpty);
    });

    testWidgets('a locked place is refused as in the app, and the sheet '
        'stays', (tester) async {
      when(
        () => capture.captureTask(
          text: any(named: 'text'),
          target: any(named: 'target'),
          now: any(named: 'now'),
          monthFirst: any(named: 'monthFirst'),
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, TaskCaptured>(
          ApplicationError.locked(message: 'locked'),
        ),
      );
      final window = _FakeWindow(CaptureMode.task);
      await pump(tester, window);
      await tester.enterText(find.byType(EditableText), 'call Sam');
      await tester.pump();
      await tester.tap(find.text('Add task'));
      await tester.pump(CaptureOverAppHost.failurePause);
      await tester.pumpAndSettle();
      expect(
        find.text(
          'That place is locked, so the task wasn’t added. Unlock it, or '
          'change where tasks go in Settings.',
        ),
        findsOneWidget,
      );
      expect(find.byType(CaptureTaskForm), findsOneWidget);
      expect(window.finished, 0);
    });

    testWidgets('dismissed, the window goes', (tester) async {
      final window = _FakeWindow(CaptureMode.task);
      await pump(tester, window);
      Navigator.of(tester.element(find.byType(CaptureTaskForm))).pop();
      await tester.pumpAndSettle();
      expect(window.finished, 1);
    });

    testWidgets('Note mode offers a new note, which opens in Kanso', (
      tester,
    ) async {
      final window = _FakeWindow(CaptureMode.note);
      await pump(tester, window);
      expect(find.byType(CaptureTaskForm), findsNothing);
      expect(find.text('Take photo'), findsOneWidget);

      await tester.tap(find.text('New note'));
      await tester.pumpAndSettle();
      expect(window.opened, <String>['kanso://open/new']);
      expect(window.finished, 0);
    });

    testWidgets('photos picked are saved to the Inbox, said, then the '
        'window goes', (tester) async {
      picker.answer = Right<ApplicationError, List<ShareFile>>(<ShareFile>[
        ShareFile(
          name: 'photo.jpg',
          contentType: 'image/jpeg',
          bytes: Uint8List.fromList(<int>[1]),
        ),
      ]);
      final window = _FakeWindow(CaptureMode.note);
      await pump(tester, window);
      await tester.tap(find.text('Choose photos'));
      await tester.pump();
      await tester.pump();
      verify(() => capture.capture(any())).called(1);
      expect(find.text('Saved to the Inbox'), findsOneWidget);
      // Closing the sheet to pick did not close the window.
      expect(window.finished, 0);

      await tester.pump(CaptureOverAppHost.noticePause);
      expect(window.finished, 1);
      // The notice's own timer runs out.
      await tester.pumpAndSettle(const Duration(seconds: 3));
    });

    testWidgets('a picker closed with nothing lets the window go', (
      tester,
    ) async {
      final window = _FakeWindow(CaptureMode.note);
      await pump(tester, window);
      await tester.tap(find.text('Take photo'));
      await tester.pumpAndSettle();
      verifyNever(() => capture.capture(any()));
      expect(window.finished, 1);
    });
  });
}
