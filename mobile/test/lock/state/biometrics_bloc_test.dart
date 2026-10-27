// Kanso — biometrics_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/models/data/biometrics_status.dart';
import 'package:kanso/lock/state/biometrics/biometrics_bloc.dart';
import 'package:kanso_lock/kanso_lock.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late MockSessionRepository session;
  late MockDeviceLockRepository device;
  var unlocked = true;

  setUp(() {
    unlocked = true;
    session = MockSessionRepository();
    device = MockDeviceLockRepository();
    when(() => session.lockStatus()).thenAnswer(
      (_) async => Right<ApplicationError, LockStatus>(
        LockStatus(
          hasLockPassword: true,
          unlocked: unlocked,
          biometricsOn: false,
        ),
      ),
    );
    when(() => session.libraryId()).thenAnswer(
      (_) async => const Right<ApplicationError, String>('library-1'),
    );
    when(() => session.beginBiometricEnrollment()).thenAnswer(
      (_) async => const Right<ApplicationError, int>(9),
    );
    when(() => session.turnOffBiometrics()).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(() => device.forget(any())).thenAnswer(
      (_) async => const Right<ApplicationError, Unit>(unit),
    );
    when(() => device.enroll(any(), any(), any())).thenAnswer(
      (_) async => const Right<ApplicationError, BiometricOutcome>(
        BiometricOutcome.done,
      ),
    );
  });

  BiometricsBloc build() => BiometricsBloc(session, device);

  group('BiometricsBloc turning on', () {
    blocTest<BiometricsBloc, BiometricsState>(
      'enrols with the ticket and the library id',
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOn('Use it')),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.enabled),
      ],
      verify: (_) =>
          verify(() => device.enroll(9, 'library-1', 'Use it')).called(1),
    );

    blocTest<BiometricsBloc, BiometricsState>(
      'with the notes closed it asks nothing of the phone',
      setUp: () => unlocked = false,
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOn('Use it')),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.notesClosed),
      ],
      verify: (_) {
        verifyNever(() => session.beginBiometricEnrollment());
        verifyNever(() => device.enroll(any(), any(), any()));
      },
    );

    blocTest<BiometricsBloc, BiometricsState>(
      'notes that closed meanwhile are the same as closed',
      setUp: () {
        when(() => session.beginBiometricEnrollment()).thenAnswer(
          (_) async => const Left<ApplicationError, int>(
            ApplicationError.locked(message: 'The note is locked.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOn('Use it')),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.notesClosed),
      ],
    );

    blocTest<BiometricsBloc, BiometricsState>(
      'a dismissed prompt is cancelled',
      setUp: () {
        when(() => device.enroll(any(), any(), any())).thenAnswer(
          (_) async => const Right<ApplicationError, BiometricOutcome>(
            BiometricOutcome.cancelled,
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOn('Use it')),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.cancelled),
      ],
    );

    for (final failing in <BiometricOutcome>[
      BiometricOutcome.invalidated,
      BiometricOutcome.unavailable,
      BiometricOutcome.failed,
    ]) {
      blocTest<BiometricsBloc, BiometricsState>(
        '${failing.name} is a failure',
        setUp: () {
          when(() => device.enroll(any(), any(), any())).thenAnswer(
            (_) async => Right<ApplicationError, BiometricOutcome>(failing),
          );
        },
        build: build,
        act: (bloc) => bloc.add(const BiometricsEvent.turnedOn('Use it')),
        expect: () => const <BiometricsState>[
          BiometricsState(status: BiometricsStatus.working),
          BiometricsState(status: BiometricsStatus.failed),
        ],
      );
    }

    blocTest<BiometricsBloc, BiometricsState>(
      'another failure of the library is a failure, with its error',
      setUp: () {
        when(() => session.beginBiometricEnrollment()).thenAnswer(
          (_) async => const Left<ApplicationError, int>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOn('Use it')),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.failed, error: _failure),
      ],
    );
  });

  group('BiometricsBloc turning off', () {
    blocTest<BiometricsBloc, BiometricsState>(
      'forgets the keys in the core and then on the phone',
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOff()),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.disabled),
      ],
      verify: (_) {
        verifyInOrder(<void Function()>[
          () => session.turnOffBiometrics(),
          () => device.forget('library-1'),
        ]);
      },
    );

    blocTest<BiometricsBloc, BiometricsState>(
      'a failure of the core leaves the phone alone',
      setUp: () {
        when(() => session.turnOffBiometrics()).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const BiometricsEvent.turnedOff()),
      expect: () => const <BiometricsState>[
        BiometricsState(status: BiometricsStatus.working),
        BiometricsState(status: BiometricsStatus.failed, error: _failure),
      ],
      verify: (_) => verifyNever(() => device.forget(any())),
    );
  });
}
