// Kanso — daily_note_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/state/daily_note/daily_note_bloc.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

void main() {
  late MockCaptureRepository capture;
  final morning = DateTime(2026, 10, 26, 8, 5);
  final night = DateTime(2026, 10, 26, 23, 59, 59);

  setUp(() {
    capture = MockCaptureRepository();
    registerFallbackValue(morning);
  });

  group('DailyNoteBloc', () {
    test('starts loading, with no note', () {
      final state = DailyNoteBloc(capture).state;
      expect(state.status, LoadStatus.loading);
      expect(state.noteId, isNull);
    });

    blocTest<DailyNoteBloc, DailyNoteState>(
      "finds today's note, asking with the phone's date and time",
      setUp: () => when(() => capture.todayNote(any())).thenAnswer(
        (_) async => const Right<ApplicationError, String>(
          'note:daily-2026-10-26',
        ),
      ),
      build: () => DailyNoteBloc(capture),
      act: (bloc) => bloc.add(DailyNoteEvent.requested(morning)),
      expect: () => const <DailyNoteState>[
        DailyNoteState(),
        DailyNoteState(
          status: LoadStatus.ready,
          noteId: 'note:daily-2026-10-26',
        ),
      ],
      verify: (_) => verify(() => capture.todayNote(morning)).called(1),
    );

    blocTest<DailyNoteBloc, DailyNoteState>(
      'a failure is an error, and trying again asks again with the new time',
      setUp: () {
        final answers = <Either<ApplicationError, String>>[
          const Left<ApplicationError, String>(
            ApplicationError.storage(message: 'Busy.'),
          ),
          const Right<ApplicationError, String>('note:daily-2026-10-26'),
        ];
        when(
          () => capture.todayNote(any()),
        ).thenAnswer((_) async => answers.removeAt(0));
      },
      build: () => DailyNoteBloc(capture),
      act: (bloc) async {
        bloc.add(DailyNoteEvent.requested(morning));
        await Future<void>.delayed(Duration.zero);
        bloc.add(DailyNoteEvent.requested(night));
      },
      wait: const Duration(milliseconds: 5),
      expect: () => const <DailyNoteState>[
        DailyNoteState(),
        DailyNoteState(
          status: LoadStatus.failed,
          error: ApplicationError.storage(message: 'Busy.'),
        ),
        // Trying again says it is loading until the answer comes.
        DailyNoteState(),
        DailyNoteState(
          status: LoadStatus.ready,
          noteId: 'note:daily-2026-10-26',
        ),
      ],
      verify: (_) {
        verify(() => capture.todayNote(morning)).called(1);
        verify(() => capture.todayNote(night)).called(1);
      },
    );
  });
}
