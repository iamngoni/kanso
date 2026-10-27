// Kanso — lock_status_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/widgets.dart' show AppLifecycleState;
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso_lock/kanso_lock.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/mock_repositories.dart';

void main() {
  late MockSessionRepository session;
  late MockDeviceLockRepository device;
  late StreamController<LockEvent> events;
  var status = const LockStatus(
    hasLockPassword: true,
    unlocked: false,
    biometricsOn: true,
  );

  setUp(() {
    status = const LockStatus(
      hasLockPassword: true,
      unlocked: false,
      biometricsOn: true,
    );
    events = StreamController<LockEvent>.broadcast();
    addTearDown(events.close);
    session = MockSessionRepository();
    device = MockDeviceLockRepository();
    when(() => session.lockStatus()).thenAnswer(
      (_) async => Right<ApplicationError, LockStatus>(status),
    );
    when(() => session.lockEvents()).thenAnswer((_) => events.stream);
    when(() => session.lock()).thenAnswer((_) async {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: false,
        biometricsOn: true,
      );
      return const Right<ApplicationError, Unit>(unit);
    });
    when(() => session.lockForBackground()).thenAnswer((_) async {
      status = const LockStatus(
        hasLockPassword: true,
        unlocked: false,
        biometricsOn: true,
      );
      return const Right<ApplicationError, Unit>(unit);
    });
    when(() => device.availability()).thenAnswer(
      (_) async =>
          const Right<ApplicationError, BiometricKind>(BiometricKind.faceId),
    );
  });

  LockStatusBloc build() => LockStatusBloc(session, device);

  const closed = LockStatusState(
    loaded: true,
    hasLockPassword: true,
    biometricsOn: true,
    kind: BiometricKind.faceId,
  );
  final open = closed.copyWith(unlocked: true);

  group('LockStatusBloc', () {
    test('starts not loaded, with nothing open', () {
      final state = build().state;
      expect(state.loaded, isFalse);
      expect(state.unlocked, isFalse);
      expect(state.biometricsUsable, isFalse);
    });

    blocTest<LockStatusBloc, LockStatusState>(
      'started reads the status and what the phone offers',
      build: build,
      act: (bloc) => bloc.add(const LockStatusEvent.started()),
      expect: () => <LockStatusState>[closed],
      verify: (bloc) {
        expect(bloc.state.biometricsUsable, isTrue);
        expect(bloc.state.biometricsOfferable, isFalse);
      },
    );

    blocTest<LockStatusBloc, LockStatusState>(
      'a lock event reads the status again',
      build: build,
      act: (bloc) async {
        bloc.add(const LockStatusEvent.started());
        await Future<void>.delayed(Duration.zero);
        status = const LockStatus(
          hasLockPassword: true,
          unlocked: true,
          biometricsOn: true,
        );
        events.add(const LockEvent.unlocked());
      },
      wait: const Duration(milliseconds: 50),
      expect: () => <LockStatusState>[closed, open],
    );

    blocTest<LockStatusBloc, LockStatusState>(
      'lock now closes the notes',
      seed: () => open,
      build: build,
      act: (bloc) => bloc.add(const LockStatusEvent.lockedNow()),
      expect: () => <LockStatusState>[closed],
      verify: (_) => verify(() => session.lock()).called(1),
    );

    blocTest<LockStatusBloc, LockStatusState>(
      'hidden closes the notes for the background',
      seed: () => open,
      build: build,
      act: (bloc) => bloc.add(
        const LockStatusEvent.lifecycleChanged(AppLifecycleState.hidden),
      ),
      expect: () => <LockStatusState>[closed],
      verify: (_) => verify(() => session.lockForBackground()).called(1),
    );

    for (final lifecycle in <AppLifecycleState>[
      AppLifecycleState.inactive,
      AppLifecycleState.resumed,
      AppLifecycleState.paused,
    ]) {
      blocTest<LockStatusBloc, LockStatusState>(
        '${lifecycle.name} leaves the notes alone',
        seed: () => open,
        build: build,
        act: (bloc) => bloc.add(LockStatusEvent.lifecycleChanged(lifecycle)),
        expect: () => const <LockStatusState>[],
        verify: (_) => verifyNever(() => session.lockForBackground()),
      );
    }

    blocTest<LockStatusBloc, LockStatusState>(
      'hidden with the notes closed does nothing',
      seed: () => closed,
      build: build,
      act: (bloc) => bloc.add(
        const LockStatusEvent.lifecycleChanged(AppLifecycleState.hidden),
      ),
      expect: () => const <LockStatusState>[],
      verify: (_) => verifyNever(() => session.lockForBackground()),
    );

    blocTest<LockStatusBloc, LockStatusState>(
      'a status that cannot be read keeps what it knew',
      seed: () => open,
      setUp: () {
        when(() => session.lockStatus()).thenAnswer(
          (_) async => const Left<ApplicationError, LockStatus>(
            ApplicationError.storage(message: 'Disk error.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockStatusEvent.refreshed()),
      // The same state: nothing is emitted for no change.
      expect: () => const <LockStatusState>[],
    );

    blocTest<LockStatusBloc, LockStatusState>(
      'a phone with no biometrics offers none',
      setUp: () {
        when(() => device.availability()).thenAnswer(
          (_) async =>
              const Right<ApplicationError, BiometricKind>(BiometricKind.none),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LockStatusEvent.started()),
      verify: (bloc) {
        expect(bloc.state.biometricsUsable, isFalse);
        expect(bloc.state.biometricsOfferable, isFalse);
      },
    );
  });
}
