// Kanso — change_password_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/models/data/lock_form_status.dart';
import 'package:kanso/lock/state/change_password/change_password_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

void main() {
  late MockSessionRepository session;

  setUp(() {
    session = MockSessionRepository();
    when(
      () => session.changePassword('old password', 'new password'),
    ).thenAnswer(
      (_) async => const Right<ApplicationError, int>(0),
    );
  });

  ChangePasswordBloc build() => ChangePasswordBloc(session);

  group('ChangePasswordBloc', () {
    blocTest<ChangePasswordBloc, ChangePasswordState>(
      'changes the password',
      build: build,
      act: (bloc) => bloc.add(
        const ChangePasswordEvent.submitted(
          'old password',
          'new password',
          'new password',
        ),
      ),
      expect: () => const <ChangePasswordState>[
        ChangePasswordState(status: LockFormStatus.working),
        ChangePasswordState(status: LockFormStatus.done),
      ],
    );

    blocTest<ChangePasswordBloc, ChangePasswordState>(
      'says how many items kept the old password',
      setUp: () {
        when(
          () => session.changePassword('old password', 'new password'),
        ).thenAnswer((_) async => const Right<ApplicationError, int>(3));
      },
      build: build,
      act: (bloc) => bloc.add(
        const ChangePasswordEvent.submitted(
          'old password',
          'new password',
          'new password',
        ),
      ),
      expect: () => const <ChangePasswordState>[
        ChangePasswordState(status: LockFormStatus.working),
        ChangePasswordState(status: LockFormStatus.done, kept: 3),
      ],
    );

    blocTest<ChangePasswordBloc, ChangePasswordState>(
      'new passwords that differ are a mismatch, and nothing changes',
      build: build,
      act: (bloc) => bloc.add(
        const ChangePasswordEvent.submitted(
          'old password',
          'new password',
          'new passwor',
        ),
      ),
      expect: () => const <ChangePasswordState>[
        ChangePasswordState(status: LockFormStatus.mismatch),
      ],
      verify: (_) => verifyNever(() => session.changePassword(any(), any())),
    );

    blocTest<ChangePasswordBloc, ChangePasswordState>(
      'a new password that is too short is rejected',
      build: build,
      act: (bloc) => bloc.add(
        const ChangePasswordEvent.submitted('old password', 'short', 'short'),
      ),
      expect: () => const <ChangePasswordState>[
        ChangePasswordState(status: LockFormStatus.tooShort),
      ],
      verify: (_) => verifyNever(() => session.changePassword(any(), any())),
    );

    blocTest<ChangePasswordBloc, ChangePasswordState>(
      'a wrong current password says so',
      setUp: () {
        when(
          () => session.changePassword('old password', 'new password'),
        ).thenAnswer(
          (_) async => const Left<ApplicationError, int>(
            ApplicationError.auth(message: "That isn't the lock password."),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const ChangePasswordEvent.submitted(
          'old password',
          'new password',
          'new password',
        ),
      ),
      expect: () => const <ChangePasswordState>[
        ChangePasswordState(status: LockFormStatus.working),
        ChangePasswordState(status: LockFormStatus.wrongPassword),
      ],
    );

    blocTest<ChangePasswordBloc, ChangePasswordState>(
      'another failure is a failure',
      setUp: () {
        when(
          () => session.changePassword('old password', 'new password'),
        ).thenAnswer(
          (_) async => const Left<ApplicationError, int>(
            ApplicationError.storage(message: 'Disk error.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const ChangePasswordEvent.submitted(
          'old password',
          'new password',
          'new password',
        ),
      ),
      expect: () => const <ChangePasswordState>[
        ChangePasswordState(status: LockFormStatus.working),
        ChangePasswordState(
          status: LockFormStatus.failed,
          error: ApplicationError.storage(message: 'Disk error.'),
        ),
      ],
    );
  });
}
