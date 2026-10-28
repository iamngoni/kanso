// Kanso — unlock_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/lock/models/data/lock_target.dart';
import 'package:kanso/lock/models/data/unlock_status.dart';
import 'package:kanso/lock/state/unlock/unlock_bloc.dart';
import 'package:kanso_lock/kanso_lock.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';
import '../../helpers/seeded_appearance_bloc.dart';

const _note = LockTarget.note('note:1');
const _book = LockTarget.notebook('book:1', 'Journal');
const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late MockSessionRepository session;
  late MockDeviceLockRepository device;
  late MockPreferencesRepository preferences;
  late Preferences saved;
  var status = const LockStatus(
    hasLockPassword: true,
    unlocked: false,
    biometricsOn: false,
  );
  var kind = BiometricKind.faceId;

  setUpAll(() {
    registerFallbackValue((Preferences current) => current);
  });

  setUp(() {
    saved = const Preferences();
    status = const LockStatus(
      hasLockPassword: true,
      unlocked: false,
      biometricsOn: false,
    );
    kind = BiometricKind.faceId;
    session = MockSessionRepository();
    device = MockDeviceLockRepository();
    preferences = MockPreferencesRepository();
    when(() => session.lockStatus()).thenAnswer(
      (_) async => Right<ApplicationError, LockStatus>(status),
    );
    when(() => device.availability()).thenAnswer(
      (_) async => Right<ApplicationError, BiometricKind>(kind),
    );
    when(
      preferences.load,
    ).thenAnswer((_) async => Right<ApplicationError, Preferences>(saved));
    when(() => preferences.update(any())).thenAnswer((call) async {
      final change =
          call.positionalArguments.first as Preferences Function(Preferences);
      saved = change(saved);
      return Right<ApplicationError, Preferences>(saved);
    });
    when(() => session.unlockNote('note:1', 'right password')).thenAnswer(
      (_) async => const Right<ApplicationError, bool>(true),
    );
    when(() => session.unlockNote('note:1', 'wrong password')).thenAnswer(
      (_) async => const Right<ApplicationError, bool>(false),
    );
    when(() => session.unlockNotebook('book:1', 'right password')).thenAnswer(
      (_) async => const Right<ApplicationError, bool>(true),
    );
    when(() => session.libraryId()).thenAnswer(
      (_) async => const Right<ApplicationError, String>('library-1'),
    );
    when(() => session.beginBiometricUnlock()).thenAnswer(
      (_) async => const Right<ApplicationError, int?>(7),
    );
    when(() => session.notebookIsOpen(any())).thenAnswer(
      (_) async => const Right<ApplicationError, bool>(false),
    );
  });

  UnlockBloc build() => UnlockBloc(session, device, preferences);

  void outcome(BiometricOutcome value) {
    when(() => device.unlock(any(), any(), any())).thenAnswer(
      (_) async => Right<ApplicationError, BiometricOutcome>(value),
    );
  }

  group('UnlockBloc with the password', () {
    blocTest<UnlockBloc, UnlockState>(
      'opens a note, and offers the biometric the first time',
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'right password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.opened, offerBiometrics: true),
      ],
      verify: (_) => expect(saved.biometricOfferSeen, isTrue),
    );

    blocTest<UnlockBloc, UnlockState>(
      'does not offer it a second time',
      setUp: () => saved = const Preferences(biometricOfferSeen: true),
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'right password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.opened),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'does not offer it when they are on, or the phone has none',
      setUp: () {
        status = const LockStatus(
          hasLockPassword: true,
          unlocked: true,
          biometricsOn: true,
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'right password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.opened),
      ],
      verify: (_) => expect(saved.biometricOfferSeen, isFalse),
    );

    blocTest<UnlockBloc, UnlockState>(
      'does not offer it when the phone has no biometrics',
      setUp: () => kind = BiometricKind.none,
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'right password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.opened),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'a wrong password says so and offers nothing',
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'wrong password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.wrongPassword),
      ],
      verify: (_) => expect(saved.biometricOfferSeen, isFalse),
    );

    blocTest<UnlockBloc, UnlockState>(
      'a text that cannot be a lock password is a wrong one',
      setUp: () {
        when(() => session.unlockNote('note:1', 'short')).thenAnswer(
          (_) async => const Left<ApplicationError, bool>(
            ApplicationError.invalid(message: 'Too short.'),
          ),
        );
      },
      build: build,
      act: (bloc) =>
          bloc.add(const UnlockEvent.passwordSubmitted(_note, 'short')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.wrongPassword),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'a failure of the library is a failure, with its error',
      setUp: () {
        when(() => session.unlockNote('note:1', 'right password')).thenAnswer(
          (_) async => const Left<ApplicationError, bool>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'right password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.failed, error: _failure),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'opens a notebook with its own call',
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_book, 'right password'),
      ),
      verify: (_) {
        verify(
          () => session.unlockNotebook('book:1', 'right password'),
        ).called(1);
        verifyNever(() => session.unlockNote(any(), any()));
      },
    );

    blocTest<UnlockBloc, UnlockState>(
      'a locked note error from the library is shown as a failure',
      setUp: () {
        when(() => session.unlockNote('note:1', 'right password')).thenAnswer(
          (_) async => const Left<ApplicationError, bool>(
            ApplicationError.locked(message: 'The note is locked.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const UnlockEvent.passwordSubmitted(_note, 'right password'),
      ),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(
          status: UnlockStatus.failed,
          error: ApplicationError.locked(message: 'The note is locked.'),
        ),
      ],
    );
  });

  group('UnlockBloc with a biometric', () {
    for (final entry in <BiometricOutcome, UnlockStatus>{
      BiometricOutcome.done: UnlockStatus.opened,
      BiometricOutcome.cancelled: UnlockStatus.cancelled,
      BiometricOutcome.invalidated: UnlockStatus.invalidated,
      BiometricOutcome.unavailable: UnlockStatus.unavailable,
      BiometricOutcome.failed: UnlockStatus.failed,
    }.entries) {
      blocTest<UnlockBloc, UnlockState>(
        '${entry.key.name} ends as ${entry.value.name}',
        setUp: () => outcome(entry.key),
        build: build,
        act: (bloc) => bloc.add(const UnlockEvent.biometricRequested('Unlock')),
        expect: () => <UnlockState>[
          const UnlockState(status: UnlockStatus.working),
          UnlockState(status: entry.value),
        ],
        verify: (_) {
          verify(() => device.unlock(7, 'library-1', 'Unlock')).called(1);
        },
      );
    }

    blocTest<UnlockBloc, UnlockState>(
      'with biometrics off there is nothing to ask the phone',
      setUp: () {
        when(() => session.beginBiometricUnlock()).thenAnswer(
          (_) async => const Right<ApplicationError, int?>(null),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.biometricRequested('Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.unavailable),
      ],
      verify: (_) => verifyNever(() => device.unlock(any(), any(), any())),
    );

    blocTest<UnlockBloc, UnlockState>(
      'a ticket the library refuses is a failure',
      setUp: () {
        when(() => session.beginBiometricUnlock()).thenAnswer(
          (_) async => const Left<ApplicationError, int?>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.biometricRequested('Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.failed, error: _failure),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'a phone that errors is a failure',
      setUp: () {
        when(() => device.unlock(any(), any(), any())).thenAnswer(
          (_) async => const Left<ApplicationError, BiometricOutcome>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.biometricRequested('Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.failed, error: _failure),
      ],
    );
  });

  group('UnlockBloc at the gate of a notebook', () {
    blocTest<UnlockBloc, UnlockState>(
      'a notebook that is open asks nothing',
      setUp: () {
        when(() => session.notebookIsOpen('book:1')).thenAnswer(
          (_) async => const Right<ApplicationError, bool>(true),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.gateOpened(_book, 'Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.opened),
      ],
      verify: (_) => verifyNever(() => device.unlock(any(), any(), any())),
    );

    blocTest<UnlockBloc, UnlockState>(
      'with biometrics off the password is to be asked',
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.gateOpened(_book, 'Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.unavailable),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'with biometrics on, it tries them and opens',
      setUp: () {
        status = const LockStatus(
          hasLockPassword: true,
          unlocked: false,
          biometricsOn: true,
        );
        outcome(BiometricOutcome.done);
      },
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.gateOpened(_book, 'Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.opened),
      ],
    );

    blocTest<UnlockBloc, UnlockState>(
      'a cancelled check leaves the password to be asked',
      setUp: () {
        status = const LockStatus(
          hasLockPassword: true,
          unlocked: false,
          biometricsOn: true,
        );
        outcome(BiometricOutcome.cancelled);
      },
      build: build,
      act: (bloc) => bloc.add(const UnlockEvent.gateOpened(_book, 'Unlock')),
      expect: () => const <UnlockState>[
        UnlockState(status: UnlockStatus.working),
        UnlockState(status: UnlockStatus.cancelled),
      ],
    );
  });
}
