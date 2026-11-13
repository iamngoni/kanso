// Kanso — capture_task_form_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_first_row.dart';
import 'package:kanso/capture/models/data/media_source.dart';
import 'package:kanso/capture/state/capture_task/capture_task_cubit.dart';
import 'package:kanso/capture/views/widgets/capture_sheet.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

class _Host extends StatelessWidget {
  const _Host({required this.cubit, required this.chosen});

  final CaptureTaskCubit cubit;
  final List<MediaSource> chosen;

  @override
  Widget build(BuildContext context) => Scaffold(
    body: Center(
      child: TextButton(
        onPressed: () => CaptureSheet.show(
          context,
          first: CaptureFirstRow(
            icon: KansoIcons.plus,
            label: 'New note',
            onTap: () {},
          ),
          onSource: chosen.add,
          tasks: cubit,
        ),
        child: const Text('Open'),
      ),
    ),
  );
}

void main() {
  late MockCaptureRepository capture;
  late SeededPreferencesRepository preferences;
  late List<MediaSource> chosen;

  setUpAll(() {
    registerFallbackValue(CaptureTarget.dailyNote);
    registerFallbackValue(DateTime(2026));
  });

  setUp(() {
    capture = MockCaptureRepository();
    preferences = SeededPreferencesRepository(const Preferences());
    chosen = <MediaSource>[];
    when(
      () =>
          capture.taskHint(any(), any(), monthFirst: any(named: 'monthFirst')),
    ).thenReturn(null);
    when(
      () => capture.taskHint(
        'film Pixel review fri 3pm every week',
        any(),
        monthFirst: any(named: 'monthFirst'),
      ),
    ).thenReturn('Fri 23 Oct, 15:00 · every week');
    when(
      () => capture.captureTask(
        text: any(named: 'text'),
        target: any(named: 'target'),
        now: any(named: 'now'),
        monthFirst: any(named: 'monthFirst'),
      ),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, TaskCaptured>(
        TaskCaptured(noteId: 'note:daily', line: '- [ ] film'),
      ),
    );
  });

  Future<CaptureTaskCubit> open(WidgetTester tester) async {
    final cubit = CaptureTaskCubit(capture, preferences);
    await pumpKanso(tester, _Host(cubit: cubit, chosen: chosen));
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    return cubit;
  }

  Future<void> goToTask(WidgetTester tester) async {
    await tester.tap(find.text('Task'));
    await tester.pumpAndSettle();
  }

  group('the capture sheet with Task mode', () {
    testWidgets(
      'opens in Note mode with the sources and a Note | Task switch',
      (
        tester,
      ) async {
        await open(tester);
        expect(find.text('Note'), findsOneWidget);
        expect(find.text('Task'), findsOneWidget);
        expect(find.text('Take photo'), findsOneWidget);
        expect(find.text('Add task'), findsNothing);
      },
    );

    testWidgets('Task mode is a field, the place it goes and Add task', (
      tester,
    ) async {
      await open(tester);
      await goToTask(tester);
      expect(find.text('Take photo'), findsNothing);
      expect(
        find.text('What needs doing? Try “call Sam fri 3pm”'),
        findsOneWidget,
      );
      expect(find.text('Goes to today’s daily note'), findsOneWidget);
      expect(find.text('Add task'), findsOneWidget);
    });

    testWidgets('shows how a typed date and repeat read, like the desktop', (
      tester,
    ) async {
      await open(tester);
      await goToTask(tester);
      await tester.enterText(
        find.byType(EditableText),
        'film Pixel review fri 3pm every week',
      );
      await tester.pumpAndSettle();
      expect(find.text('Fri 23 Oct, 15:00 · every week'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'film Pixel review');
      await tester.pumpAndSettle();
      expect(find.text('Fri 23 Oct, 15:00 · every week'), findsNothing);
    });

    testWidgets('Add task writes the task to the daily note and empties the '
        'field for the next', (tester) async {
      await open(tester);
      await goToTask(tester);
      await tester.enterText(find.byType(EditableText), 'film Pixel review');
      await tester.pumpAndSettle();
      await tester.tap(find.text('Add task'));
      await tester.pumpAndSettle();
      verify(
        () => capture.captureTask(
          text: 'film Pixel review',
          target: CaptureTarget.dailyNote,
          now: any(named: 'now'),
          monthFirst: false,
        ),
      ).called(1);
      expect(find.text('Added to today’s daily note'), findsOneWidget);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        isEmpty,
      );
    });

    testWidgets('goes to the Inbox when that is the setting', (tester) async {
      preferences.current = const Preferences(
        captureTaskTarget: CaptureTarget.inbox,
      );
      await open(tester);
      await goToTask(tester);
      expect(find.text('Goes to the Inbox'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'buy tape');
      await tester.pumpAndSettle();
      await tester.tap(find.text('Add task'));
      await tester.pumpAndSettle();
      verify(
        () => capture.captureTask(
          text: 'buy tape',
          target: CaptureTarget.inbox,
          now: any(named: 'now'),
          monthFirst: false,
        ),
      ).called(1);
      expect(find.text('Added to the Inbox'), findsOneWidget);
    });

    testWidgets('Add task waits for words', (tester) async {
      await open(tester);
      await goToTask(tester);
      await tester.tap(find.text('Add task'));
      await tester.pumpAndSettle();
      verifyNever(
        () => capture.captureTask(
          text: any(named: 'text'),
          target: any(named: 'target'),
          now: any(named: 'now'),
          monthFirst: any(named: 'monthFirst'),
        ),
      );
    });

    testWidgets('a locked place says so and keeps the text', (tester) async {
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
      await open(tester);
      await goToTask(tester);
      await tester.enterText(find.byType(EditableText), 'secret');
      await tester.pumpAndSettle();
      await tester.tap(find.text('Add task'));
      await tester.pumpAndSettle();
      expect(find.textContaining('That place is locked'), findsOneWidget);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        'secret',
      );
    });

    testWidgets('remembers Task mode: the sheet opens in it next time', (
      tester,
    ) async {
      await open(tester);
      await goToTask(tester);
      expect(preferences.current.captureAsTask, isTrue);
      // Closed and opened again, with a cubit that reads the saved choice.
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();
      await open(tester);
      expect(find.text('Add task'), findsOneWidget);
      expect(find.text('Take photo'), findsNothing);
      // And back to Note mode, kept too.
      await tester.tap(find.text('Note'));
      await tester.pumpAndSettle();
      expect(find.text('Take photo'), findsOneWidget);
      expect(preferences.current.captureAsTask, isFalse);
    });

    testWidgets('Note mode still reports a chosen source', (tester) async {
      await open(tester);
      await tester.tap(find.text('Choose photos'));
      await tester.pumpAndSettle();
      expect(chosen, <MediaSource>[MediaSource.photos]);
    });
  });
}
