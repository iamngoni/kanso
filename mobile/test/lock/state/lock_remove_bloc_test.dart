// Kanso — lock_remove_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/models/data/lock_target.dart';
import 'package:kanso/lock/models/data/remove_status.dart';
import 'package:kanso/lock/state/lock_remove/lock_remove_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

const _note = LockTarget.note('note:1');
const _book = LockTarget.notebook('book:1', 'Journal');
const _inBook = ApplicationError.invalid(
  message:
      'The note is in a locked notebook, so it stays locked. Move it out '
      'first.',
);

void main() {
  late MockSessionRepository session;

  setUp(() {
    session = MockSessionRepository();
    when(() => session.removeNoteLock('note:1')).thenAnswer(
      (_) async =>
          const Right<ApplicationError, Saved>(Saved(id: 'note:1', rev: 'r3')),
    );
    when(() => session.removeNotebookLock('book:1')).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(
      () => session.lockProgress(),
    ).thenAnswer((_) => const Stream<SealProgress>.empty());
    when(() => session.takeCopyNotice()).thenAnswer(
      (_) async => const Right<ApplicationError, LockCopyNotice?>(null),
    );
    when(() => session.cancelLock()).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
  });

  LockRemoveBloc build() => LockRemoveBloc(session);

  group('LockRemoveBloc', () {
    blocTest<LockRemoveBloc, LockRemoveState>(
      'removes a note lock',
      build: build,
      act: (bloc) => bloc.add(const LockRemoveEvent.confirmed(_note)),
      expect: () => const <LockRemoveState>[
        LockRemoveState(status: RemoveStatus.working),
        LockRemoveState(status: RemoveStatus.done),
      ],
      verify: (_) => verify(() => session.removeNoteLock('note:1')).called(1),
    );

    blocTest<LockRemoveBloc, LockRemoveState>(
      'removes a notebook lock',
      build: build,
      act: (bloc) => bloc.add(const LockRemoveEvent.confirmed(_book)),
      expect: () => const <LockRemoveState>[
        LockRemoveState(status: RemoveStatus.working),
        LockRemoveState(status: RemoveStatus.done),
      ],
      verify: (_) =>
          verify(() => session.removeNotebookLock('book:1')).called(1),
    );

    blocTest<LockRemoveBloc, LockRemoveState>(
      'says the copies it made for the note, once',
      setUp: () {
        when(() => session.takeCopyNotice()).thenAnswer(
          (_) async => const Right<ApplicationError, LockCopyNotice?>(
            LockCopyNotice(removed: true, copied: 2),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockRemoveEvent.confirmed(_note)),
      expect: () => const <LockRemoveState>[
        LockRemoveState(status: RemoveStatus.working),
        LockRemoveState(
          status: RemoveStatus.done,
          copyNotice: LockCopyNotice(removed: true, copied: 2),
        ),
      ],
      verify: (_) => verify(() => session.takeCopyNotice()).called(1),
    );

    // Cancel is heard between two files and nothing is written before the
    // last one is open, so a stopped Remove Lock leaves the note locked.
    blocTest<LockRemoveBloc, LockRemoveState>(
      'Cancel stops it, and the note stays locked',
      setUp: () {
        final waiting = Completer<void>();
        when(() => session.cancelLock()).thenAnswer((_) async {
          waiting.complete();
          return const Right<ApplicationError, Unit>(unit);
        });
        when(() => session.removeNoteLock('note:1')).thenAnswer((_) async {
          await waiting.future;
          return const Left<ApplicationError, Saved>(
            ApplicationError.invalid(message: 'Locking was cancelled.'),
          );
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockRemoveEvent.confirmed(_note));
        await bloc.stream.firstWhere((s) => s.status == RemoveStatus.working);
        bloc.add(const LockRemoveEvent.cancelled());
      },
      expect: () => const <LockRemoveState>[
        LockRemoveState(status: RemoveStatus.working),
        LockRemoveState(status: RemoveStatus.working, cancelling: true),
        LockRemoveState(status: RemoveStatus.stopped),
      ],
      verify: (_) {
        verify(() => session.cancelLock()).called(1);
        verifyNever(() => session.takeCopyNotice());
      },
    );

    blocTest<LockRemoveBloc, LockRemoveState>(
      'Cancel with nothing running does nothing',
      build: build,
      act: (bloc) => bloc.add(const LockRemoveEvent.cancelled()),
      expect: () => const <LockRemoveState>[],
      verify: (_) => verifyNever(() => session.cancelLock()),
    );

    blocTest<LockRemoveBloc, LockRemoveState>(
      "keeps the library's own sentence when it refuses",
      setUp: () {
        when(() => session.removeNoteLock('note:1')).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(_inBook),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockRemoveEvent.confirmed(_note)),
      expect: () => const <LockRemoveState>[
        LockRemoveState(status: RemoveStatus.working),
        LockRemoveState(status: RemoveStatus.failed, error: _inBook),
      ],
    );

    blocTest<LockRemoveBloc, LockRemoveState>(
      'notes that closed meanwhile are a locked error',
      setUp: () {
        when(() => session.removeNotebookLock('book:1')).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.locked(message: 'The note is locked.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockRemoveEvent.confirmed(_book)),
      expect: () => const <LockRemoveState>[
        LockRemoveState(status: RemoveStatus.working),
        LockRemoveState(
          status: RemoveStatus.failed,
          error: ApplicationError.locked(message: 'The note is locked.'),
        ),
      ],
    );
  });
}
