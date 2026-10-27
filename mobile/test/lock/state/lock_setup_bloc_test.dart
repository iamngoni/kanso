// Kanso — lock_setup_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

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
}
