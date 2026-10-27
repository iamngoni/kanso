// Kanso — push_registration_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/push/models/data/push_registration_phase.dart';
import 'package:kanso/push/models/data/push_token.dart';
import 'package:kanso/push/state/push_registration/push_registration_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const _token = PushToken(
  platform: PushPlatformKind.fcm,
  token: 'fcm-token-1',
  environment: PushEnvironmentKind.production,
);

const _newer = PushToken(
  platform: PushPlatformKind.fcm,
  token: 'fcm-token-2',
  environment: PushEnvironmentKind.production,
);

const _network = ApplicationError.network(message: 'No connection.');

Either<ApplicationError, Unit> _ok() =>
    const Right<ApplicationError, Unit>(unit);

void main() {
  late MockPushRepository push;
  late StreamController<PushToken> tokens;

  setUpAll(() => registerFallbackValue(_token));

  setUp(() {
    push = MockPushRepository();
    tokens = StreamController<PushToken>.broadcast();
    addTearDown(tokens.close);
    when(
      () => push.unavailableReason(),
    ).thenAnswer((_) async => const Right<ApplicationError, String?>(null));
    when(() => push.tokens()).thenAnswer((_) => tokens.stream);
    when(() => push.register()).thenAnswer((_) async => _ok());
    when(() => push.setToken(any())).thenAnswer((_) async => _ok());
    when(() => push.refresh()).thenAnswer((_) async => _ok());
  });

  PushRegistrationBloc build() => PushRegistrationBloc(push);

  group('PushRegistrationBloc', () {
    test('starts waiting to find out', () {
      expect(build().state.phase, PushRegistrationPhase.starting);
    });

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'started asks the OS once and has the library retry',
      build: build,
      act: (bloc) => bloc.add(const PushRegistrationEvent.started()),
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(phase: PushRegistrationPhase.waiting),
      ],
      verify: (_) {
        verify(() => push.register()).called(1);
        verify(() => push.refresh()).called(1);
      },
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'started where push cannot work asks for nothing',
      setUp: () {
        when(() => push.unavailableReason()).thenAnswer(
          (_) async => const Right<ApplicationError, String?>('No Firebase.'),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const PushRegistrationEvent.started()),
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(phase: PushRegistrationPhase.unavailable),
      ],
      verify: (_) {
        verifyNever(() => push.register());
        verifyNever(() => push.refresh());
      },
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'started where the plugin cannot be reached is unavailable, with why',
      setUp: () {
        when(() => push.unavailableReason()).thenAnswer(
          (_) async => const Left<ApplicationError, String?>(_network),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const PushRegistrationEvent.started()),
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(
          phase: PushRegistrationPhase.unavailable,
          error: _network,
        ),
      ],
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'each token the OS issues goes to the library, a new one too',
      build: build,
      act: (bloc) async {
        bloc.add(const PushRegistrationEvent.started());
        await pumpEventQueue();
        tokens
          ..add(_token)
          ..add(_newer);
        await pumpEventQueue();
      },
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(phase: PushRegistrationPhase.waiting),
        const PushRegistrationState(phase: PushRegistrationPhase.registered),
      ],
      verify: (_) {
        verifyInOrder(<void Function()>[
          () => push.setToken(_token),
          () => push.setToken(_newer),
        ]);
      },
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'a token the library cannot register is kept as an error, and the '
      'next one clears it',
      setUp: () {
        when(() => push.setToken(_token)).thenAnswer(
          (_) async => const Left<ApplicationError, Unit>(_network),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const PushRegistrationEvent.started());
        await pumpEventQueue();
        tokens.add(_token);
        await pumpEventQueue();
        tokens.add(_newer);
        await pumpEventQueue();
      },
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(phase: PushRegistrationPhase.waiting),
        const PushRegistrationState(
          phase: PushRegistrationPhase.waiting,
          error: _network,
        ),
        const PushRegistrationState(phase: PushRegistrationPhase.registered),
      ],
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'a failure to ask the OS is kept, not thrown',
      setUp: () {
        when(
          () => push.register(),
        ).thenAnswer((_) async => const Left<ApplicationError, Unit>(_network));
      },
      build: build,
      act: (bloc) => bloc.add(const PushRegistrationEvent.started()),
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(error: _network),
      ],
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'coming back to the front has the library retry',
      build: build,
      act: (bloc) async {
        bloc.add(const PushRegistrationEvent.started());
        await pumpEventQueue();
        bloc.add(const PushRegistrationEvent.resumed());
      },
      verify: (_) => verify(() => push.refresh()).called(2),
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'a retry that fails is kept as an error',
      build: build,
      act: (bloc) async {
        bloc.add(const PushRegistrationEvent.started());
        await pumpEventQueue();
        when(
          () => push.refresh(),
        ).thenAnswer((_) async => const Left<ApplicationError, Unit>(_network));
        bloc.add(const PushRegistrationEvent.resumed());
      },
      expect: () => <PushRegistrationState>[
        const PushRegistrationState(phase: PushRegistrationPhase.waiting),
        const PushRegistrationState(
          phase: PushRegistrationPhase.waiting,
          error: _network,
        ),
      ],
    );

    blocTest<PushRegistrationBloc, PushRegistrationState>(
      'coming back to the front where push cannot work does nothing',
      setUp: () {
        when(() => push.unavailableReason()).thenAnswer(
          (_) async => const Right<ApplicationError, String?>('No Firebase.'),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const PushRegistrationEvent.started());
        await pumpEventQueue();
        bloc.add(const PushRegistrationEvent.resumed());
      },
      verify: (_) => verifyNever(() => push.refresh()),
    );
  });
}
