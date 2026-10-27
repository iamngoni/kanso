// Kanso — recovery_key_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/state/recovery_key/recovery_key_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/mock_repositories.dart';

void main() {
  late MockAccountRepository account;

  setUp(() => account = MockAccountRepository());

  group('RecoveryKeyBloc', () {
    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'shows the key the flow gave it, and asks the library for nothing',
      build: () => RecoveryKeyBloc(account),
      act: (bloc) =>
          bloc.add(const RecoveryKeyEvent.started(key: sampleRecoveryKey)),
      expect: () => const <RecoveryKeyState>[
        RecoveryKeyState(status: LoadStatus.ready, key: sampleRecoveryKey),
      ],
      verify: (_) => verifyNever(account.pendingRecoveryKey),
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'with none, shows the key the library kept',
      build: () {
        when(account.pendingRecoveryKey).thenAnswer(
          (_) async => const Right<ApplicationError, RecoveryKeyText?>(
            sampleRecoveryKey,
          ),
        );
        return RecoveryKeyBloc(account);
      },
      act: (bloc) => bloc.add(const RecoveryKeyEvent.started()),
      expect: () => const <RecoveryKeyState>[
        RecoveryKeyState(),
        RecoveryKeyState(status: LoadStatus.ready, key: sampleRecoveryKey),
      ],
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'with none kept, there is nothing to save',
      build: () {
        when(account.pendingRecoveryKey).thenAnswer(
          (_) async => const Right<ApplicationError, RecoveryKeyText?>(null),
        );
        return RecoveryKeyBloc(account);
      },
      act: (bloc) => bloc.add(const RecoveryKeyEvent.started()),
      expect: () => const <RecoveryKeyState>[
        RecoveryKeyState(),
        RecoveryKeyState(status: LoadStatus.ready),
      ],
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'a key that cannot be read is failed, and tried again on request',
      build: () {
        var calls = 0;
        when(account.pendingRecoveryKey).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, RecoveryKeyText?>(
                  ApplicationError.storage(message: 'Disk error.'),
                )
              : const Right<ApplicationError, RecoveryKeyText?>(
                  sampleRecoveryKey,
                );
        });
        return RecoveryKeyBloc(account);
      },
      act: (bloc) async {
        bloc.add(const RecoveryKeyEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == LoadStatus.failed,
        );
        bloc.add(const RecoveryKeyEvent.started());
      },
      expect: () => const <RecoveryKeyState>[
        RecoveryKeyState(),
        RecoveryKeyState(
          status: LoadStatus.failed,
          error: ApplicationError.storage(message: 'Disk error.'),
        ),
        RecoveryKeyState(),
        RecoveryKeyState(status: LoadStatus.ready, key: sampleRecoveryKey),
      ],
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'Done does nothing until the box is checked',
      build: () => RecoveryKeyBloc(account),
      seed: () => const RecoveryKeyState(
        status: LoadStatus.ready,
        key: sampleRecoveryKey,
      ),
      act: (bloc) => bloc.add(const RecoveryKeyEvent.done()),
      expect: () => const <RecoveryKeyState>[],
      verify: (_) => verifyNever(account.recoveryKeySaved),
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'with the box checked Done says the key is saved',
      build: () {
        when(account.recoveryKeySaved).thenAnswer(
          (_) async => const Right<ApplicationError, Unit>(unit),
        );
        return RecoveryKeyBloc(account);
      },
      seed: () => const RecoveryKeyState(
        status: LoadStatus.ready,
        key: sampleRecoveryKey,
      ),
      act: (bloc) => bloc
        ..add(const RecoveryKeyEvent.confirmedChanged(confirmed: true))
        ..add(const RecoveryKeyEvent.done()),
      expect: () => const <RecoveryKeyState>[
        RecoveryKeyState(
          status: LoadStatus.ready,
          key: sampleRecoveryKey,
          confirmed: true,
        ),
        RecoveryKeyState(
          status: LoadStatus.ready,
          key: sampleRecoveryKey,
          confirmed: true,
          saving: true,
        ),
        RecoveryKeyState(
          status: LoadStatus.ready,
          key: sampleRecoveryKey,
          confirmed: true,
          done: true,
        ),
      ],
      verify: (_) => verify(account.recoveryKeySaved).called(1),
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'a save that fails keeps the screen and says why',
      build: () {
        when(account.recoveryKeySaved).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(
            ApplicationError.storage(message: 'Disk error.'),
          ),
        );
        return RecoveryKeyBloc(account);
      },
      seed: () => const RecoveryKeyState(
        status: LoadStatus.ready,
        key: sampleRecoveryKey,
        confirmed: true,
      ),
      act: (bloc) => bloc.add(const RecoveryKeyEvent.done()),
      verify: (bloc) {
        expect(bloc.state.done, isFalse);
        expect(bloc.state.saving, isFalse);
        expect(bloc.state.error, isA<StorageError>());
      },
    );

    blocTest<RecoveryKeyBloc, RecoveryKeyState>(
      'with nothing to save Done closes at once',
      build: () {
        when(account.recoveryKeySaved).thenAnswer(
          (_) async => const Right<ApplicationError, Unit>(unit),
        );
        return RecoveryKeyBloc(account);
      },
      seed: () => const RecoveryKeyState(status: LoadStatus.ready),
      act: (bloc) => bloc.add(const RecoveryKeyEvent.done()),
      verify: (bloc) => expect(bloc.state.done, isTrue),
    );
  });
}
