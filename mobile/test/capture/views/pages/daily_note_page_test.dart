// Kanso — daily_note_page_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/state/daily_note/daily_note_bloc.dart';
import 'package:kanso/capture/views/pages/daily_note_page.dart';
import 'package:kanso/capture/views/widgets/daily_note_failed_pane.dart';
import 'package:kanso/capture/views/widgets/daily_note_loading_pane.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/injection.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';

void main() {
  late MockCaptureRepository capture;

  setUpAll(() => registerFallbackValue(DateTime(2026)));

  setUp(() {
    capture = MockCaptureRepository();
    getIt.registerFactory<DailyNoteBloc>(() => DailyNoteBloc(capture));
    addTearDown(() => unawaited(getIt.reset()));
  });

  group('DailyNotePage', () {
    testWidgets('says it is opening the note while it waits', (tester) async {
      final answer = Completer<Either<ApplicationError, String>>();
      when(() => capture.todayNote(any())).thenAnswer((_) => answer.future);
      final router = await withClock(
        Clock.fixed(DateTime(2026, 10, 26, 8, 5)),
        () => pumpRouted(tester, const DailyNotePage()),
      );
      expect(find.byType(DailyNoteLoadingPane), findsOneWidget);
      expect(find.text('Opening today’s note'), findsOneWidget);
      expect(whereIs(router), '/start');
      verify(() => capture.todayNote(DateTime(2026, 10, 26, 8, 5))).called(1);
    });

    testWidgets('goes to the note in All Notes once it is found', (
      tester,
    ) async {
      when(() => capture.todayNote(any())).thenAnswer(
        (_) async => const Right<ApplicationError, String>(
          'note:daily-2026-10-26',
        ),
      );
      final router = await pumpRouted(tester, const DailyNotePage());
      expect(whereIs(router), '/notes/note/note%3Adaily-2026-10-26');
    });

    testWidgets('a failure says so, and trying again asks again', (
      tester,
    ) async {
      final answers = <Either<ApplicationError, String>>[
        const Left<ApplicationError, String>(
          ApplicationError.storage(message: 'Busy.'),
        ),
        const Right<ApplicationError, String>('note:daily-2026-10-26'),
      ];
      when(
        () => capture.todayNote(any()),
      ).thenAnswer((_) async => answers.removeAt(0));
      final router = await pumpRouted(tester, const DailyNotePage());
      expect(find.byType(DailyNoteFailedPane), findsOneWidget);
      expect(find.text('Today’s note didn’t open'), findsOneWidget);
      // Nothing of the technical message is shown.
      expect(find.textContaining('Busy'), findsNothing);
      expect(whereIs(router), '/start');

      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(whereIs(router), '/notes/note/note%3Adaily-2026-10-26');
      verify(() => capture.todayNote(any())).called(2);
    });

    testWidgets('a failure can leave for All Notes', (tester) async {
      when(() => capture.todayNote(any())).thenAnswer(
        (_) async => const Left<ApplicationError, String>(
          ApplicationError.internal(message: 'Bug.'),
        ),
      );
      final router = await pumpRouted(tester, const DailyNotePage());
      await tester.tap(find.text('All Notes'));
      await tester.pumpAndSettle();
      expect(whereIs(router), '/notes');
    });
  });
}
