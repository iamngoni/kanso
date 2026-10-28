// Kanso — undo_notice_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/undo_notice.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

UndoNotice _notice(int ticket, String message, int serial) =>
    UndoNotice(ticket: ticket, message: message, serial: serial);

void main() {
  late MockLibraryRepository library;
  late MockHapticsRepository haptics;

  setUp(() {
    library = MockLibraryRepository();
    haptics = MockHapticsRepository();
    when(() => library.undo(any())).thenAnswer(
      (_) async => const Right<ApplicationError, int>(2),
    );
    when(() => haptics.impact()).thenAnswer((_) async {});
  });

  UndoNoticeBloc build() => UndoNoticeBloc(library, haptics);

  group('UndoNoticeBloc', () {
    test('starts with nothing to show', () {
      expect(build().state, const UndoNoticeState());
    });

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'shows the notice, and takes it away when its time is up',
      build: build,
      act: (bloc) async {
        bloc.add(const UndoNoticeEvent.shown(3, 'Pinned 1 note.'));
        await Future<void>.delayed(Duration.zero);
        bloc.add(const UndoNoticeEvent.timedOut(1));
      },
      expect: () => <UndoNoticeState>[
        UndoNoticeState(notice: _notice(3, 'Pinned 1 note.', 1)),
        const UndoNoticeState(),
      ],
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'a later notice takes the place of the one before, and the first '
      "one's time being up does not take the second away",
      build: build,
      act: (bloc) async {
        bloc.add(const UndoNoticeEvent.shown(1, 'First.'));
        await Future<void>.delayed(Duration.zero);
        bloc.add(const UndoNoticeEvent.shown(2, 'Second.'));
        await Future<void>.delayed(Duration.zero);
        bloc.add(const UndoNoticeEvent.timedOut(1));
      },
      expect: () => <UndoNoticeState>[
        UndoNoticeState(notice: _notice(1, 'First.', 1)),
        UndoNoticeState(notice: _notice(2, 'Second.', 2)),
      ],
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'a timeout that belongs to an earlier notice does nothing',
      build: build,
      seed: () => UndoNoticeState(notice: _notice(2, 'Second.', 2)),
      act: (bloc) => bloc.add(const UndoNoticeEvent.timedOut(1)),
      expect: () => <UndoNoticeState>[],
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'Undo takes the change back with its ticket, ticks the hand and '
      'removes the notice',
      build: build,
      seed: () => UndoNoticeState(notice: _notice(9, 'Pinned 2 notes.', 1)),
      act: (bloc) => bloc.add(const UndoNoticeEvent.undoPressed()),
      expect: () => <UndoNoticeState>[
        UndoNoticeState(
          notice: _notice(9, 'Pinned 2 notes.', 1),
          undoing: true,
        ),
        const UndoNoticeState(),
      ],
      verify: (_) {
        verify(() => library.undo(9)).called(1);
        verify(() => haptics.impact()).called(1);
      },
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'Undo pressed twice takes the change back once',
      build: build,
      seed: () => UndoNoticeState(notice: _notice(9, 'Pinned 2 notes.', 1)),
      act: (bloc) => bloc
        ..add(const UndoNoticeEvent.undoPressed())
        ..add(const UndoNoticeEvent.undoPressed()),
      verify: (_) => verify(() => library.undo(9)).called(1),
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'Undo with no notice does nothing',
      build: build,
      act: (bloc) => bloc.add(const UndoNoticeEvent.undoPressed()),
      expect: () => <UndoNoticeState>[],
      verify: (_) => verifyNever(() => library.undo(any())),
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'a failed undo says so, until it is dismissed',
      setUp: () => when(() => library.undo(any())).thenAnswer(
        (_) async => const Left<ApplicationError, int>(_failure),
      ),
      build: build,
      seed: () => UndoNoticeState(notice: _notice(4, 'Moved 1 note.', 1)),
      act: (bloc) async {
        bloc.add(const UndoNoticeEvent.undoPressed());
        await Future<void>.delayed(const Duration(milliseconds: 10));
        bloc.add(const UndoNoticeEvent.errorDismissed());
      },
      expect: () => <UndoNoticeState>[
        UndoNoticeState(
          notice: _notice(4, 'Moved 1 note.', 1),
          undoing: true,
        ),
        const UndoNoticeState(error: _failure),
        const UndoNoticeState(),
      ],
    );

    blocTest<UndoNoticeBloc, UndoNoticeState>(
      'dismissing removes the notice',
      build: build,
      seed: () => UndoNoticeState(notice: _notice(4, 'Moved 1 note.', 1)),
      act: (bloc) => bloc.add(const UndoNoticeEvent.dismissed()),
      expect: () => <UndoNoticeState>[const UndoNoticeState()],
    );
  });
}
