// Kanso — lock_setup_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/models/data/lock_form_status.dart';
import 'package:kanso/lock/models/data/lock_mode.dart';
import 'package:kanso/lock/models/data/lock_target.dart';
import 'package:kanso/lock/state/lock_setup/lock_setup_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

const _note = LockTarget.note('note:1');
const _book = LockTarget.notebook('book:1', 'Journal');
const _saved = Saved(id: 'note:1', rev: 'r2');
const _failure = ApplicationError.storage(message: 'Disk error.');

LockSetupState _state(
  LockTarget target,
  LockMode mode,
  LockFormStatus status, {
  int count = 0,
  ApplicationError? error,
}) => LockSetupState(
  target: target,
  mode: mode,
  status: status,
  count: count,
  error: error,
);

void main() {
  late MockSessionRepository session;
  var status = const LockStatus(
    hasLockPassword: false,
    unlocked: false,
    biometricsOn: false,
  );

  setUp(() {
    session = MockSessionRepository();
    status = const LockStatus(
      hasLockPassword: false,
      unlocked: false,
      biometricsOn: false,
    );
    when(() => session.lockStatus()).thenAnswer(
      (_) async => Right<ApplicationError, LockStatus>(status),
    );
    // What it would seal could not be counted: the dialog has no line about
    // it, which is how the tests of the way through the sheet read.
    when(() => session.noteLockSummary(any())).thenAnswer(
      (_) async => const Left<ApplicationError, LockSummary>(_failure),
    );
    when(() => session.notebookLockSummary(any())).thenAnswer(
      (_) async => const Left<ApplicationError, LockSummary>(_failure),
    );
    when(
      () => session.lockProgress(),
    ).thenAnswer((_) => const Stream<SealProgress>.empty());
    when(() => session.cancelLock()).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(() => session.takeCopyNotice()).thenAnswer(
      (_) async => const Right<ApplicationError, LockCopyNotice?>(null),
    );
  });

  LockSetupBloc build() => LockSetupBloc(session);

  group('LockSetupBloc finds the way', () {
    blocTest<LockSetupBloc, LockSetupState>(
      'no lock password yet: set one',
      build: build,
      act: (bloc) => bloc.add(const LockSetupEvent.started(_note)),
      expect: () => <LockSetupState>[
        const LockSetupState(target: _note),
        _state(_note, LockMode.setPassword, LockFormStatus.ready),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'notes closed: ask for the password',
      setUp: () => status = const LockStatus(
        hasLockPassword: true,
        unlocked: false,
        biometricsOn: false,
      ),
      build: build,
      act: (bloc) => bloc.add(const LockSetupEvent.started(_book)),
      expect: () => <LockSetupState>[
        const LockSetupState(target: _book),
        _state(_book, LockMode.askPassword, LockFormStatus.ready),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'notes open: confirm',
      setUp: () => status = const LockStatus(
        hasLockPassword: true,
        unlocked: true,
        biometricsOn: false,
      ),
      build: build,
      act: (bloc) => bloc.add(const LockSetupEvent.started(_note)),
      expect: () => <LockSetupState>[
        const LockSetupState(target: _note),
        _state(_note, LockMode.confirm, LockFormStatus.ready),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'a status that cannot be read is a failure',
      setUp: () {
        when(() => session.lockStatus()).thenAnswer(
          (_) async => const Left<ApplicationError, LockStatus>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockSetupEvent.started(_note)),
      expect: () => <LockSetupState>[
        const LockSetupState(target: _note),
        const LockSetupState(
          target: _note,
          status: LockFormStatus.failed,
          error: _failure,
        ),
      ],
    );
  });

  group('LockSetupBloc sets the lock password', () {
    blocTest<LockSetupBloc, LockSetupState>(
      'two matching passwords lock the note',
      setUp: () {
        when(() => session.lockNote('note:1', 'long enough')).thenAnswer(
          (_) async => const Right<ApplicationError, Saved>(_saved),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere(
          (s) => s.status == LockFormStatus.ready,
        );
        bloc.add(const LockSetupEvent.submitted('long enough', 'long enough'));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.setPassword, LockFormStatus.working),
        _state(_note, LockMode.setPassword, LockFormStatus.done, count: 1),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'passwords that differ are a mismatch and nothing is locked',
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.submitted('long enough', 'long enougH'));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.setPassword, LockFormStatus.mismatch),
      ],
      verify: (_) => verifyNever(() => session.lockNote(any(), any())),
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'a short password is too short and nothing is locked',
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_book));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.submitted('short', 'short'));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_book, LockMode.setPassword, LockFormStatus.tooShort),
      ],
      verify: (_) => verifyNever(
        () => session.lockNotebook(any(), password: any(named: 'password')),
      ),
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'a notebook reports how many notes were sealed',
      setUp: () {
        when(
          () => session.lockNotebook('book:1', password: 'long enough'),
        ).thenAnswer((_) async => const Right<ApplicationError, int>(4));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_book));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.submitted('long enough', 'long enough'));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_book, LockMode.setPassword, LockFormStatus.working),
        _state(_book, LockMode.setPassword, LockFormStatus.done, count: 4),
      ],
    );
  });

  group('LockSetupBloc says what it copied', () {
    setUp(() {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: true,
        biometricsOn: false,
      );
    });

    blocTest<LockSetupBloc, LockSetupState>(
      'a lock that sealed copies carries the notice to the sheet',
      setUp: () {
        when(() => session.lockNoteWithSession('note:1')).thenAnswer(
          (_) async => const Right<ApplicationError, Saved>(_saved),
        );
        when(() => session.takeCopyNotice()).thenAnswer(
          (_) async => const Right<ApplicationError, LockCopyNotice?>(
            LockCopyNotice(removed: false, copied: 3),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.confirmed());
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.confirm, LockFormStatus.working),
        _state(_note, LockMode.confirm, LockFormStatus.done, count: 1).copyWith(
          copyNotice: const LockCopyNotice(removed: false, copied: 3),
        ),
      ],
      verify: (_) => verify(() => session.takeCopyNotice()).called(1),
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'a lock that failed asks for no notice',
      setUp: () {
        when(() => session.lockNoteWithSession('note:1')).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(_failure),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.confirmed());
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.confirm, LockFormStatus.working),
        _state(
          _note,
          LockMode.confirm,
          LockFormStatus.failed,
          error: _failure,
        ),
      ],
      verify: (_) => verifyNever(() => session.takeCopyNotice()),
    );
  });

  group('LockSetupBloc asks for the password', () {
    setUp(() {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: false,
        biometricsOn: false,
      );
    });

    blocTest<LockSetupBloc, LockSetupState>(
      'a wrong password says so and stays',
      setUp: () {
        when(() => session.lockNote('note:1', 'not the one')).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(
            ApplicationError.auth(message: "That isn't the lock password."),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.submitted('not the one', ''));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.askPassword, LockFormStatus.working),
        _state(_note, LockMode.askPassword, LockFormStatus.wrongPassword),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'the right password locks the note',
      setUp: () {
        when(() => session.lockNote('note:1', 'the one')).thenAnswer(
          (_) async => const Right<ApplicationError, Saved>(_saved),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.submitted('the one', ''));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.askPassword, LockFormStatus.working),
        _state(_note, LockMode.askPassword, LockFormStatus.done, count: 1),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'another failure is shown as one, with its error',
      setUp: () {
        when(() => session.lockNote('note:1', 'the one')).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(_failure),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.submitted('the one', ''));
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.askPassword, LockFormStatus.working),
        _state(
          _note,
          LockMode.askPassword,
          LockFormStatus.failed,
          error: _failure,
        ),
      ],
    );
  });

  group('LockSetupBloc confirms', () {
    setUp(() {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: true,
        biometricsOn: false,
      );
    });

    blocTest<LockSetupBloc, LockSetupState>(
      'locks the note with the open session',
      setUp: () {
        when(() => session.lockNoteWithSession('note:1')).thenAnswer(
          (_) async => const Right<ApplicationError, Saved>(_saved),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.confirmed());
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.confirm, LockFormStatus.working),
        _state(_note, LockMode.confirm, LockFormStatus.done, count: 1),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'locks a notebook with no password',
      setUp: () {
        when(() => session.lockNotebook('book:1')).thenAnswer(
          (_) async => const Right<ApplicationError, int>(2),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_book));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.confirmed());
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_book, LockMode.confirm, LockFormStatus.working),
        _state(_book, LockMode.confirm, LockFormStatus.done, count: 2),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'a session that closed meanwhile asks for the password after all',
      setUp: () {
        when(() => session.lockNoteWithSession('note:1')).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(
            ApplicationError.locked(message: 'The note is locked.'),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.confirmed());
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.confirm, LockFormStatus.working),
        _state(_note, LockMode.askPassword, LockFormStatus.ready),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'another failure is a failure',
      setUp: () {
        when(() => session.lockNoteWithSession('note:1')).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(_failure),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LockSetupEvent.started(_note));
        await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
        bloc.add(const LockSetupEvent.confirmed());
      },
      skip: 2,
      expect: () => <LockSetupState>[
        _state(_note, LockMode.confirm, LockFormStatus.working),
        _state(
          _note,
          LockMode.confirm,
          LockFormStatus.failed,
          error: _failure,
        ),
      ],
    );
  });

  group('LockSetupBloc counts and follows the files', () {
    const summary = LockSummary(files: 12, bytes: 84000000, shared: 3);
    const first = SealProgress(
      phase: SealPhase.sealing,
      done: 5,
      total: 12,
      bytesDone: 31000000,
      bytesTotal: 84000000,
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'the dialog is told what a note would seal',
      setUp: () {
        when(() => session.noteLockSummary('note:1')).thenAnswer(
          (_) async => const Right<ApplicationError, LockSummary>(summary),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockSetupEvent.started(_note)),
      expect: () => <LockSetupState>[
        const LockSetupState(target: _note),
        _state(_note, LockMode.setPassword, LockFormStatus.ready),
        _state(
          _note,
          LockMode.setPassword,
          LockFormStatus.ready,
        ).copyWith(summary: summary),
      ],
    );

    blocTest<LockSetupBloc, LockSetupState>(
      'the dialog is told what a notebook would seal',
      setUp: () {
        when(() => session.notebookLockSummary('book:1')).thenAnswer(
          (_) async => const Right<ApplicationError, LockSummary>(summary),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockSetupEvent.started(_book)),
      verify: (bloc) => expect(bloc.state.summary, summary),
    );

    test('progress reaches the state while the lock runs', () async {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: true,
        biometricsOn: false,
      );
      final progress = StreamController<SealProgress>();
      final finish = Completer<Either<ApplicationError, Saved>>();
      when(
        () => session.lockProgress(),
      ).thenAnswer((_) => progress.stream);
      when(
        () => session.lockNoteWithSession('note:1'),
      ).thenAnswer((_) => finish.future);
      final bloc = build();
      addTearDown(bloc.close);
      bloc.add(const LockSetupEvent.started(_note));
      await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
      bloc.add(const LockSetupEvent.confirmed());
      await bloc.stream.firstWhere((s) => s.status == LockFormStatus.working);
      progress.add(first);
      final seen = await bloc.stream.firstWhere((s) => s.progress != null);
      expect(seen.progress, first);
      finish.complete(const Right<ApplicationError, Saved>(_saved));
      final done = await bloc.stream.firstWhere(
        (s) => s.status == LockFormStatus.done,
      );
      expect(done.count, 1);
      await progress.close();
    });

    test('Cancel stops the lock and the sheet reports it stopped', () async {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: true,
        biometricsOn: false,
      );
      final finish = Completer<Either<ApplicationError, Saved>>();
      when(
        () => session.lockNoteWithSession('note:1'),
      ).thenAnswer((_) => finish.future);
      final bloc = build();
      addTearDown(bloc.close);
      bloc.add(const LockSetupEvent.started(_note));
      await bloc.stream.firstWhere((s) => s.status == LockFormStatus.ready);
      bloc.add(const LockSetupEvent.confirmed());
      await bloc.stream.firstWhere((s) => s.status == LockFormStatus.working);

      bloc.add(const LockSetupEvent.cancelled());
      await bloc.stream.firstWhere((s) => s.cancelling);
      verify(() => session.cancelLock()).called(1);
      // The library stops between two files and says so as an error.
      finish.complete(
        const Left<ApplicationError, Saved>(
          ApplicationError.invalid(message: 'Locking was cancelled.'),
        ),
      );
      final stopped = await bloc.stream.firstWhere(
        (s) => s.status == LockFormStatus.stopped,
      );
      expect(stopped.cancelling, isTrue);
    });

    test('Cancel with nothing running does nothing', () async {
      final bloc = build();
      addTearDown(bloc.close);
      bloc.add(const LockSetupEvent.cancelled());
      await Future<void>.delayed(Duration.zero);
      verifyNever(() => session.cancelLock());
    });
  });
}
